use anyhow::{Error, anyhow};
use gmod::{
    LUA_TBOOLEAN, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE,
    lua::{self},
    push_to_lua::PushToLua,
};

use neo4rs::{BoltBoolean, BoltFloat, BoltInteger, BoltList, BoltMap, BoltString, BoltType};

unsafe fn get_table_key(l: &lua::State, key_type: i32) -> anyhow::Result<String> {
    if key_type == LUA_TSTRING {
        let key = l.check_string(-2)?;
        return Ok(key);
    }

    if key_type == LUA_TNUMBER {
        let key = l.to_number(-2);
        return Ok(key.to_string());
    }

    Err(Error::msg(format!(
        "Could not retrieve table key due to unsupported key type: {}",
        key_type
    )))
}

fn is_sequential_table(l: &lua::State, index: i32) -> bool {
    let len = l.len(index);
    let mut count = 0;

    unsafe {
        l.push_nil();
        while l.next(index) != 0 {
            let key_type = l.lua_type(-2);
            if key_type != LUA_TNUMBER {
                l.pop_n(2);
                return false;
            }
            let key = l.to_number(-2) as i32;
            if key < 1 || key > len as i32 {
                l.pop_n(2);
                return false;
            }
            count += 1;
            l.pop_n(1);
        }
    }
    count == len as i64
}

pub fn lua_table_to_boltmap(l: lua::State, index: i32) -> anyhow::Result<BoltMap> {
    if !l.is_table(index) {
        return Err(anyhow!("Expected a table"));
    }

    let mut map = BoltMap::new();
    unsafe {
        l.push_nil();
        while l.next(index) != 0 {
            let key_type = l.lua_type(-2);

            let key = get_table_key(&l, key_type)?;
            let key = BoltString::from(key);

            let value_type = l.lua_type(-1);
            match value_type {
                LUA_TSTRING => {
                    let value = l.check_string(-1)?;
                    map.put(key, BoltType::String(BoltString::from(value)));
                }
                LUA_TNUMBER => {
                    let value = l.to_number(-1);
                    if value.fract() == 0.0 {
                        map.put(key, BoltType::Integer(BoltInteger::from(value as i64)));
                    } else {
                        map.put(key, BoltType::Float(BoltFloat::new(value)));
                    }
                }
                LUA_TTABLE => {
                    if is_sequential_table(&l, l.get_top()) {
                        let nested = lua_table_to_boltlist(l, l.get_top())?;
                        map.put(key, BoltType::List(nested));
                    } else {
                        let nested_table = lua_table_to_boltmap(l, l.get_top())?;
                        map.put(key, BoltType::Map(nested_table));
                    }
                }
                LUA_TBOOLEAN => {
                    let bool = l.check_boolean(-1)?;
                    map.put(key, BoltType::Boolean(BoltBoolean::new(bool)));
                }
                _ => {
                    l.pop_n(1);
                    return Err(Error::msg(format!("Unsupported table value type")));
                }
            }
            l.pop_n(1);
        }
    }
    Ok(map)
}

pub fn lua_table_to_boltlist(l: lua::State, index: i32) -> anyhow::Result<BoltList> {
    let mut list = BoltList::new();
    for i in 1..=l.len(index) as i32 {
        l.raw_geti(index, i);
        let value_type = l.lua_type(-1);

        let bolt_value = match value_type {
            LUA_TSTRING => {
                let value = l.check_string(-1)?;
                BoltType::String(BoltString::from(value))
            }
            LUA_TNUMBER => {
                let value = l.to_number(-1);
                if value.fract() == 0.0 {
                    BoltType::Integer(BoltInteger::from(value as i64))
                } else {
                    BoltType::Float(BoltFloat::new(value))
                }
            }
            LUA_TTABLE => {
                l.pop();
                return Err(Error::msg("Nested tables in lists are not supported"));
            }
            LUA_TBOOLEAN => {
                let bool = l.check_boolean(-1)?;
                BoltType::Boolean(BoltBoolean::new(bool))
            }
            _ => {
                l.pop();
                return Err(Error::msg("Unsupported table value type"));
            }
        };

        list.push(bolt_value);
        l.pop();
    }

    Ok(list)
}

pub fn boltmap_to_lua_table(l: lua::State, map: &BoltMap) -> anyhow::Result<()> {
    l.new_table();
    for (key, value) in map.value.iter() {
        l.push_string(&key.value);
        map_type_to_lua(l, value)?;
        l.raw_set_table(-3);
    }

    Ok(())
}

pub fn boltlist_to_lua_table(l: lua::State, list: &BoltList) -> anyhow::Result<()> {
    l.new_table();
    for (idx, entry) in list.iter().enumerate() {
        let key: isize = idx.try_into()?;
        l.raw_push_number((key + 1) as f64);
        map_type_to_lua(l, entry)?;
        l.raw_set_table(-3);
    }

    Ok(())
}

pub fn map_type_to_lua(l: lua::State, item: &BoltType) -> anyhow::Result<()> {
    match item {
        BoltType::Integer(v) => v.value.push_to_lua(&l),
        BoltType::Float(v) => v.value.push_to_lua(&l),
        BoltType::Node(v) => {
            boltmap_to_lua_table(l, &v.properties)?;
            return Ok(());
        }
        BoltType::List(v) => {
            boltlist_to_lua_table(l, v)?;
            return Ok(());
        }
        BoltType::Relation(v) => {
            boltmap_to_lua_table(l, &v.properties)?;
            return Ok(());
        }
        BoltType::String(v) => l.push_string(&v.value),
        BoltType::Boolean(v) => l.push_boolean(v.value),
        BoltType::Map(v) => {
            boltmap_to_lua_table(l, v)?;
            return Ok(());
        }
        _ => l.push_nil(),
    }

    Ok(())
}
