use std::sync::Arc;

use anyhow::Error;
use gmod::rstruct::RStruct;
use gmod::{lua, lua_function, register_lua_rstruct};
use neo4rs::{Query, Txn};

use crate::THREAD_WORKER;
use crate::api::graph::LuaNeoGraph;
use crate::mapping::lua_table_to_boltmap;

pub struct LuaNeoTxn(pub Arc<Txn>);

register_lua_rstruct!(LuaNeoTxn, c"Neo4jTransaction", &[]);
