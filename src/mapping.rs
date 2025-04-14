use std::ffi::{CStr, CString};

use neo4rs::{BoltBoolean, BoltFloat, BoltInteger, BoltList, BoltMap, BoltString, BoltType};
use rglua::lua::{
    LuaState, TBOOLEAN, TNUMBER, TSTRING, TTABLE, lua_gettop, lua_istable, lua_newtable, lua_next,
    lua_pop, lua_pushboolean, lua_pushinteger, lua_pushnil, lua_pushnumber, lua_pushstring,
    lua_settable, lua_toboolean, lua_tonumber, lua_tostring, lua_type,
};

unsafe fn get_table_key(l: LuaState, key_type: i32) -> Result<String, String> {
    if key_type == TSTRING {
        let key_ptr = lua_tostring(l, -2);
        if key_ptr.is_null() {
            lua_pop(l, 1);
            return Err("Invalid key type".to_string());
        }
        return Ok(unsafe { CStr::from_ptr(key_ptr).to_str().unwrap().to_string() });
    }

    if key_type == TNUMBER {
        let key = lua_tonumber(l, -2);
        return Ok(key.to_string());
    }

    Err("Table key must be a string or number".to_string())
}

pub fn lua_table_to_boltmap(l: LuaState, index: i32) -> Result<BoltMap, String> {
    #[allow(unused_unsafe)]
    if !unsafe { lua_istable(l, index) } {
        return Err("Expected a table".to_string());
    }

    let mut doc = BoltMap::new();
    unsafe {
        lua_pushnil(l);
        while lua_next(l, index) != 0 {
            let key_type = lua_type(l, -2);

            let key_result = get_table_key(l, key_type);
            if key_result.is_err() {
                lua_pop(l, 1);
                return Err(key_result.unwrap_err());
            }

            let key = BoltString::from(key_result.expect("Couldn't find key"));

            let value_type = lua_type(l, -1);
            match value_type {
                TSTRING => {
                    let value_ptr = lua_tostring(l, -1);
                    if value_ptr.is_null() {
                        lua_pop(l, 1);
                        return Err("Invalid value type".to_string());
                    }
                    let value = CStr::from_ptr(value_ptr)
                        .to_str()
                        .expect("Cannot convert ptr to BoltString")
                        .to_string();
                    doc.put(key, BoltType::String(BoltString::from(value)));
                }
                TNUMBER => {
                    let value = lua_tonumber(l, -1);
                    if value.fract() == 0.0 {
                        doc.put(key, BoltType::Integer(BoltInteger::from(value as i64)));
                    } else {
                        doc.put(key, BoltType::Float(BoltFloat::new(value)));
                    }
                }
                TTABLE => {
                    let nested_doc = lua_table_to_boltmap(l, lua_gettop(l))?;
                    doc.put(key, BoltType::Map(nested_doc));
                }
                TBOOLEAN => {
                    let bool = lua_toboolean(l, -1) != 0;
                    doc.put(key, BoltType::Boolean(BoltBoolean::new(bool)));
                }
                _ => {
                    lua_pop(l, 1);
                    return Err("Unsupported value type".to_string());
                }
            }
            lua_pop(l, 1);
        }
    }
    Ok(doc)
}

pub fn boltmap_to_lua_table(l: LuaState, map: BoltMap) {
    #[allow(unused_unsafe)]
    unsafe {
        lua_newtable(l);
        for (key, value) in map.value.iter() {
            let key = CString::new(key.to_string()).unwrap();
            lua_pushstring(l, key.as_ptr());
            map_type_to_lua(l, value.clone());
        }
    }
}

pub fn boltlist_to_lua_table(l: LuaState, list: BoltList) {
    #[allow(unused_unsafe)]
    unsafe {
        lua_newtable(l);

        for (idx, entry) in list.iter().enumerate() {
            let key: isize = idx.try_into().expect("Cannot convert i32 into isize");
            lua_pushinteger(l, key + 1);
            map_type_to_lua(l, entry.clone());
        }
        lua_settable(l, -3);
    }
}

pub fn map_type_to_lua(l: LuaState, item: BoltType) {
    match item {
        BoltType::Integer(v) => lua_pushinteger(l, v.value as isize),
        BoltType::Float(v) => lua_pushnumber(l, v.value),
        BoltType::Node(v) => {
            boltmap_to_lua_table(l, v.properties.clone());
            lua_settable(l, -3);
            return;
        }
        BoltType::List(v) => {
            boltlist_to_lua_table(l, v.clone());
            return;
        }
        BoltType::Relation(v) => {
            boltmap_to_lua_table(l, v.properties.clone());
            lua_settable(l, -3);
            return;
        }
        BoltType::String(v) => {
            let cstr =
                CString::new(v.value.clone()).expect("Failed to build CString from BoltString");
            lua_pushstring(l, cstr.as_ptr())
        }
        BoltType::Boolean(v) => lua_pushboolean(l, (v.value) as i32),
        BoltType::Map(v) => {
            boltmap_to_lua_table(l, v.clone());
            lua_settable(l, -3);
            return;
        }
        _ => lua_pushnil(l),
    }
    lua_settable(l, -3);
}
