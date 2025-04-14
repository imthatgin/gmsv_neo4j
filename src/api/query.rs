use anyhow::Result;
use gmod::rstruct::RStruct;
use gmod::{lua, lua_function, register_lua_rstruct};
use neo4rs::Query;

use crate::MT_QUERY;

register_lua_rstruct!(Query, MT_QUERY, &[]);

#[lua_function]
pub fn new_query(l: lua::State) -> Result<i32> {
    // Argument 1: query string
    let query_str = l.check_string(1)?;
    l.pop();

    let mut query = neo4rs::query(query_str.as_str());

    l.push_struct::<Query>(query);

    Ok(1)
}
