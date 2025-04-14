use gmod::gmod13_close;
use gmod::gmod13_open;
use gmod::lua;

use logging::LogLevel;
use logging::log;

use std::ffi::CStr;
use std::sync::atomic::AtomicBool;

mod api;
mod logging;
mod mapping;
mod neo_client;
mod runtime;

const NAMESPACE: &CStr = c"neo4j";
const MT_QUERY: &CStr = c"Neo4jQuery";

static SUPPRESS_MESSAGES: AtomicBool = AtomicBool::new(false);

#[gmod13_open]
unsafe fn open(l: lua::State) -> i32 {
    let cargo_name = env!("CARGO_PKG_NAME");
    let cargo_version = env!("CARGO_PKG_VERSION");

    runtime::load(l);

    l.new_table();
    {
        l.push_string(cargo_version);
        l.set_field(-2, c"VERSION");

        l.push_function(api::query::new_query);
    }
    l.set_global(NAMESPACE);

    // Just inform the user that it has been successfully loaded
    let log_message = format!("Module {} ({}) loaded", cargo_name, cargo_version);
    log(LogLevel::Info, log_message);

    0
}

#[gmod13_close]
fn close(l: lua::State) -> i32 {
    let cargo_name = env!("CARGO_PKG_NAME");
    let cargo_version = env!("CARGO_PKG_VERSION");
    let log_message = format!(
        "Module '{} ({})' is shutting down",
        cargo_name, cargo_version
    );
    log(LogLevel::Info, log_message);

    runtime::unload(l);

    0
}
