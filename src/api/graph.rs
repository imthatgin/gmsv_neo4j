use neo4rs::{Graph, query};
use rglua::lua::{
    LuaState, lua_getfield, lua_isnumber, lua_isstring, lua_istable, lua_pop, lua_pushnil,
    lua_setmetatable, lua_tointeger, lua_tostring, luaL_checkstring, luaL_getmetatable,
};

use crate::{
    LogLevel, MT_NEO4J_GRAPH, MT_NEO4J_TXN, log,
    neo_client::{self, THREAD_WORKER},
    userdata::{read_userdata, write_userdata, write_userdata_owned},
};

#[lua_function]
pub fn new_graph(l: LuaState) -> i32 {
    // Required args: uri, user, password
    let uri = rstr!(luaL_checkstring(l, 1));
    let user = rstr!(luaL_checkstring(l, 2));
    let password = rstr!(luaL_checkstring(l, 3));

    let mut config = neo4rs::ConfigBuilder::new()
        .uri(uri)
        .user(user)
        .password(password);

    if lua_istable(l, 4) {
        // db
        lua_getfield(l, 4, cstr!("db"));
        if lua_isstring(l, -1) != 0 {
            let db_str = rstr!(lua_tostring(l, -1));
            config = config.db(db_str);
        }
        lua_pop(l, 1);

        // fetch_size
        lua_getfield(l, 4, cstr!("fetch_size"));
        if lua_isnumber(l, -1) != 0 {
            let fetch_size = lua_tointeger(l, -1) as usize;
            config = config.fetch_size(fetch_size);
        }
        lua_pop(l, 1);

        // max_connections
        lua_getfield(l, 4, cstr!("max_connections"));
        if lua_isnumber(l, -1) != 0 {
            let max_connections = lua_tointeger(l, -1) as usize;
            config = config.max_connections(max_connections);
        }
        lua_pop(l, 1);
    }

    let built_config = match config.build() {
        Ok(config) => config,
        Err(err) => {
            log(
                LogLevel::Error,
                &format!("Failed to create Neo4j configuration: {}", err),
            );
            lua_pushnil(l);
            return 1;
        }
    };

    // Create a graph instance
    let graph = match neo_client::open_graph(built_config) {
        Ok(client) => client,
        Err(err) => {
            log(
                LogLevel::Error,
                &format!("Failed to connect to Neo4j: {}", err),
            );
            lua_pushnil(l);
            return 1;
        }
    };

    let connection_test_result =
        THREAD_WORKER.block_on(async { graph.run(query("MATCH (n) RETURN n")).await });

    if connection_test_result.is_err() {
        log(
            LogLevel::Error,
            &format!(
                "Connection test failed: {}",
                connection_test_result.unwrap_err()
            ),
        );
        lua_pushnil(l);
        return 1;
    }

    log(LogLevel::Info, "Successfully connected to Neo4j");

    write_userdata(l, graph);
    luaL_getmetatable(l, MT_NEO4J_GRAPH);
    lua_setmetatable(l, -2);

    1
}

/// Called in Lua to start a transaction on the default database.
#[lua_function]
pub fn start_txn(l: LuaState) -> i32 {
    let graph_result = read_userdata::<Graph>(l);
    if graph_result.is_err() {
        log(LogLevel::Error, "Failed to initialize transaction.");
        return 0;
    }

    let graph = graph_result.unwrap();

    THREAD_WORKER.block_on(async {
        let txn_result = graph.start_txn().await.map_err(|e| e.to_string());
        if txn_result.is_err() {
            log(LogLevel::Error, "Failed to initialize transaction.");
            return 0;
        }

        let txn = txn_result.unwrap();

        log(LogLevel::Debug, "Created txn");

        write_userdata_owned(l, txn);
        luaL_getmetatable(l, MT_NEO4J_TXN);
        lua_setmetatable(l, -2);

        1
    })
}

/// Called to start a transaction on a specific database.
#[lua_function]
pub fn start_txn_on(l: LuaState) -> i32 {
    let graph_result = read_userdata::<Graph>(l);
    if graph_result.is_err() {
        log(LogLevel::Error, "Failed to initialize transaction.");
        return 0;
    }

    let graph = graph_result.unwrap();

    let database_name = rstr!(luaL_checkstring(l, 2));

    THREAD_WORKER.block_on(async {
        let txn_result = graph
            .start_txn_on(database_name)
            .await
            .map_err(|e| e.to_string());
        if txn_result.is_err() {
            log(LogLevel::Error, "Failed to initialize transaction.");
            return 0;
        }

        let txn = txn_result.unwrap();

        log(LogLevel::Debug, "Created txn");

        write_userdata_owned(l, txn);
        luaL_getmetatable(l, MT_NEO4J_TXN);
        lua_setmetatable(l, -2);

        1
    })
}
