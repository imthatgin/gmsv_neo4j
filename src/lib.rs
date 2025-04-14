extern crate core;
#[macro_use]
extern crate rglua;

use rglua::lua::LuaState;
use rglua::prelude::*;

use logging::LogLevel;
use logging::log;

use std::sync::atomic::{AtomicBool, Ordering};

pub const MT_NEO4J_GRAPH: *const i8 = cstr!("Neo4jGraph");
pub const MT_NEO4J_TXN: *const i8 = cstr!("Neo4jTransaction");
pub const MT_NEO4J_QUERY: *const i8 = cstr!("Neo4jQuery");

mod api;
mod logging;
mod lua_state;
mod mapping;
mod neo_client;
mod userdata;

static SUPPRESS_MESSAGES: AtomicBool = AtomicBool::new(false);

#[gmod_open]
unsafe fn open(l: LuaState) -> i32 {
    // Sets up the lua state (ie. exposing the necessary functions)
    lua_state::init_luastate(l);

    // Just inform the user that it has been successfully loaded
    let cargo_name = env!("CARGO_PKG_NAME");
    let cargo_version = env!("CARGO_PKG_VERSION");
    let log_message = format!("Module {} ({}) loaded", cargo_name, cargo_version);
    log(LogLevel::Info, log_message);

    0
}

extern "C" fn suppress_messages(l: LuaState) -> i32 {
    let suppress = lua_toboolean(l, 1) != 0;
    SUPPRESS_MESSAGES.store(suppress, Ordering::Relaxed);
    0
}

#[gmod_close]
fn close(_l: LuaState) -> i32 {
    let cargo_name = env!("CARGO_PKG_NAME");
    let cargo_version = env!("CARGO_PKG_VERSION");
    let log_message = format!(
        "Module '{} ({})' is shutting down",
        cargo_name, cargo_version
    );
    log(LogLevel::Info, log_message);

    0
}
