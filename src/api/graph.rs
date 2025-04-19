use std::sync::Arc;

use anyhow::Error;
use gmod::rstruct::RStruct;
use gmod::{lua, lua_function, register_lua_rstruct};
use neo4rs::{BoltMap, Config, Graph, Query};
use tokio::sync::Mutex;

use crate::api::query::LuaNeoQuery;
use crate::api::result::dispatch_callback;
use crate::api::transaction::LuaNeoTxn;
use crate::{THREAD_WORKER, runtime};

pub struct LuaNeoGraph {
    pub graph: Graph,
}

impl LuaNeoGraph {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        THREAD_WORKER.block_on(async {
            let graph = Graph::connect(config).await?;
            Ok(Self { graph })
        })
    }
}

register_lua_rstruct!(LuaNeoGraph, c"Neo4jGraph", &[
    (c"Execute", execute),
    (c"ExecuteOn", execute_on),
    (c"Tx", new_txn),
    (c"TxOn", new_txn_on)
]);

#[lua_function]
pub fn new_graph(l: lua::State) -> anyhow::Result<i32> {
    let uri = l.check_string(1)?;
    let user = l.check_string(2)?;
    let password = l.check_string(3)?;

    let mut config = neo4rs::ConfigBuilder::new()
        .uri(uri)
        .user(user)
        .password(password);

    // Catch bad parameter sets
    if !l.is_none_or_nil(4) && !l.is_table(4) {
        return Err(Error::msg("Last argument must be a table of options"));
    }

    // Parse the rest of the options
    if l.is_table(4) {
        l.get_field(4, c"db");
        if l.is_string(-1) {
            let db_str = l.get_string_unchecked(-1);
            config = config.db(db_str)
        }
        l.pop_n(1);

        l.get_field(4, c"fetch_size");
        if l.is_number(-1) {
            let fetch_size = l.to_number(-1) as usize;
            config = config.fetch_size(fetch_size)
        }
        l.pop_n(1);

        l.get_field(4, c"max_connections");
        if l.is_number(-1) {
            let max_connections = l.to_number(-1) as usize;
            config = config.max_connections(max_connections)
        }
        l.pop_n(1);
    }

    let config = config.build()?;
    l.push_struct::<LuaNeoGraph>(LuaNeoGraph::new(config)?);

    Ok(1)
}

#[lua_function]
pub fn new_txn(l: lua::State) -> anyhow::Result<i32> {
    let neo_graph = l.get_struct::<LuaNeoGraph>(1)?;

    // This is non async but it should be fine
    THREAD_WORKER.block_on(async {
        let tx = neo_graph.graph.start_txn().await?;

        println!("Start txn");

        l.push_struct::<LuaNeoTxn>(LuaNeoTxn(Arc::new(Mutex::new(Some(tx)))));

        Ok(1)
    })
}

#[lua_function]
pub fn new_txn_on(l: lua::State) -> anyhow::Result<i32> {
    let neo_graph = l.get_struct::<LuaNeoGraph>(1)?;

    let db = l.check_string(2)?;

    // This is non async but it should be fine
    THREAD_WORKER.block_on(async {
        let tx = neo_graph.graph.start_txn_on(db).await?;
        println!("Start txn");

        l.push_struct::<LuaNeoTxn>(LuaNeoTxn(Arc::new(Mutex::new(Some(tx)))));
        Ok(1)
    })
}

async fn handle_graph_execution(graph: Graph, query: Query) -> anyhow::Result<Vec<BoltMap>> {
    let result = async {
        let mut results = graph.execute(query).await?;
        let mut output = Vec::new();
        while let Some(row) = results.next().await? {
            match row.to::<BoltMap>() {
                Ok(entry) => output.push(entry),
                Err(err) => return Err(anyhow::anyhow!("Row conversion error: {}", err)),
            }
        }
        Ok(output)
    }
    .await;

    result
}

async fn handle_graph_execution_on(
    db: &str,
    graph: Graph,
    query: Query,
) -> anyhow::Result<Vec<BoltMap>> {
    let result = async {
        let mut results = graph.execute_on(db, query).await?;
        let mut output = Vec::new();
        while let Some(row) = results.next().await? {
            match row.to::<BoltMap>() {
                Ok(entry) => output.push(entry),
                Err(err) => return Err(anyhow::anyhow!("Row conversion error: {}", err)),
            }
        }
        Ok(output)
    }
    .await;

    result
}

#[lua_function]
pub fn execute(l: lua::State) -> anyhow::Result<i32> {
    let graph_container = l.get_struct::<LuaNeoGraph>(1)?;
    let neo_query = l.get_struct::<LuaNeoQuery>(2)?;
    let callback = l.check_function(3)?;

    let graph = graph_container.graph.clone();
    let arc_query = neo_query.0.clone();

    runtime::run_async(async move {
        let results = {
            let query = (*arc_query).clone();

            handle_graph_execution(graph, query).await
        };

        dispatch_callback(callback, results);
    });

    Ok(0)
}

#[lua_function]
pub fn execute_on(l: lua::State) -> anyhow::Result<i32> {
    let graph_container = l.get_struct::<LuaNeoGraph>(1)?;
    let db = l.check_string(2)?;
    let neo_query = l.get_struct::<LuaNeoQuery>(3)?;
    let callback = l.check_function(4)?;

    let graph = graph_container.graph.clone();
    let arc_query = neo_query.0.clone();

    runtime::run_async(async move {
        let results = {
            let query = (*arc_query).clone();

            handle_graph_execution_on(&db, graph, query).await
        };

        dispatch_callback(callback, results);
    });

    Ok(0)
}
