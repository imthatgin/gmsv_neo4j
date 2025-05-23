use gmod::LuaReg;
use gmod::gmod13_close;
use gmod::gmod13_open;
use gmod::lua;
use gmod::lua_regs;
use lazy_static::lazy_static;
use neo4rs::BoltMap;
use tokio::runtime::Runtime;

use std::ffi::CStr;

mod api;
mod mapping;
mod runtime;

pub enum NeoThreadMessage {
    ResultSet(Vec<BoltMap>),
    CommitTx,
}

lazy_static! {
    pub static ref THREAD_WORKER: Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

const NAMESPACE: &CStr = c"neo4j";

#[gmod13_open]
unsafe fn gmod13_open(l: lua::State) -> i32 {
    let cargo_name = env!("CARGO_PKG_NAME");
    let cargo_version = env!("CARGO_PKG_VERSION");

    runtime::load(l);

    let regs = lua_regs! [
        "Query" => api::query::new_query,
        "Graph" => api::graph::new_graph
    ];

    l.register(NAMESPACE.as_ptr(), regs.as_ptr());

    // Just inform the user that it has been successfully loaded
    let log_message = format!("Module {} ({}) loaded", cargo_name, cargo_version);
    println!("{}", log_message);

    0
}

#[gmod13_close]
fn gmod13_close(l: lua::State) -> i32 {
    let cargo_name = env!("CARGO_PKG_NAME");
    let cargo_version = env!("CARGO_PKG_VERSION");
    let log_message = format!(
        "Module '{} ({})' is shutting down",
        cargo_name, cargo_version
    );
    println!("{}", log_message);

    runtime::unload(l);

    0
}
