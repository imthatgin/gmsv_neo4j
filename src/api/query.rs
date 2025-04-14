use anyhow::Error;
use gmod::rstruct::RStruct;
use gmod::{lua, lua_function, register_lua_rstruct};
use neo4rs::Query;

use crate::mapping::lua_table_to_boltmap;

pub struct LuaNeoQuery(pub Query);

register_lua_rstruct!(LuaNeoQuery, c"Neo4jQuery", &[]);

#[lua_function]
pub fn new_query(l: lua::State) -> anyhow::Result<i32> {
    // Argument 1: query string
    let query_str = l.check_string(1)?;

    let mut query = neo4rs::query(query_str.as_str());

    // Catch bad parameter sets
    if !l.is_none_or_nil(2) && !l.is_table(2) {
        return Err(Error::msg(
            "Parameter argument of Query must be a table of parameters",
        ));
    }

    if l.is_table(2) {
        let table = lua_table_to_boltmap(l, 2)?;
        for (key, value) in table.value {
            query = query.param(&key.value, value)
        }
    }

    l.push_struct::<LuaNeoQuery>(LuaNeoQuery(query));

    Ok(1)
}
