use rglua::lua::{
    LuaState, lua_pushcfunction, lua_pushvalue, lua_setfield, luaL_newmetatable, luaL_register,
};

use crate::{
    MT_NEO4J_GRAPH, MT_NEO4J_TXN,
    api::{self},
};

/// Adds the module functions to Garry's Mod's Lua state.
pub fn init_luastate(l: LuaState) {
    // Graph -> :Tx, :TxOn
    luaL_newmetatable(l, MT_NEO4J_GRAPH);
    lua_pushvalue(l, -1);
    lua_setfield(l, -2, cstr!("__index"));
    lua_pushcfunction(l, api::graph::start_txn);
    lua_setfield(l, -2, cstr!("Tx"));
    lua_pushcfunction(l, api::graph::start_txn_on);
    lua_setfield(l, -2, cstr!("TxOn"));

    // Tx -> :Execute
    luaL_newmetatable(l, MT_NEO4J_TXN);
    lua_pushvalue(l, -1);
    lua_setfield(l, -2, cstr!("__index"));
    lua_pushcfunction(l, api::tx::execute);
    lua_setfield(l, -2, cstr!("Execute"));
    lua_pushcfunction(l, api::tx::commit);
    lua_setfield(l, -2, cstr!("Commit"));
    lua_pushcfunction(l, api::tx::close);
    lua_setfield(l, -2, cstr!("Close"));

    // neo4j -> Graph, Query
    let lib = reg![
        "Graph" => api::graph::new_graph,
        "Query" => api::query::new_query
    ];
    luaL_register(l, cstr!("neo4j"), lib.as_ptr());
}
