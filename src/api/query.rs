use rglua::lua::{
    LuaState, lua_isnoneornil, lua_istable, lua_pushnil, lua_setmetatable, luaL_checkstring,
    luaL_getmetatable, luaL_optstring, luaL_typename,
};

use crate::{
    LogLevel, MT_NEO4J_QUERY, log, mapping::lua_table_to_boltmap, userdata::write_userdata,
};

#[lua_function]
pub fn new_query(l: LuaState) -> i32 {
    // Argument 1: query string
    let query_str = rstr!(luaL_checkstring(l, 1));

    // Argument 2: param table
    if !lua_isnoneornil(l, 2) && !lua_istable(l, 2) {
        let value = luaL_optstring(l, 2, cstr!("unknown"));
        let type_of = luaL_typename(l, 2);
        log(
            LogLevel::Error,
            &format!(
                "Query argument 2 must be a table of parameters: {} (type: {})",
                rstr!(value),
                rstr!(type_of)
            ),
        );
        lua_pushnil(l);
        return 1;
    }

    let mut query = neo4rs::query(query_str);

    if lua_istable(l, 2) {
        let table = match lua_table_to_boltmap(l, 2) {
            Ok(tbl) => tbl,
            Err(err) => {
                log(
                    LogLevel::Error,
                    &format!("Query parameters could not be mapped: {}", err),
                );
                lua_pushnil(l);
                return 1;
            }
        };

        for (k, v) in table.value {
            log(LogLevel::Debug, format!("{}={}", k, v));
            query = query.param(&k.value, v)
        }
    }

    write_userdata(l, query);
    luaL_getmetatable(l, MT_NEO4J_QUERY);
    lua_setmetatable(l, -2);

    1
}
