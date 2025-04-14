use lazy_static::lazy_static;
use neo4rs::{Config, Graph};
use tokio::runtime::Runtime;

lazy_static! {
    pub static ref THREAD_WORKER: Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

/// Opens a connection to the neo4j database using a provided [`Config`].
pub fn open_graph(config: Config) -> Result<Graph, String> {
    THREAD_WORKER.block_on(async { Graph::connect(config).await.map_err(|e| e.to_string()) })
}
