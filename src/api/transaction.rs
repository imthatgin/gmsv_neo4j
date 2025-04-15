use std::sync::Arc;

use gmod::rstruct::RStruct;
use gmod::{lua, lua_function, register_lua_rstruct, wait_lua_tick};
use neo4rs::{BoltMap, Txn};
use tokio::sync::Mutex;

use crate::api::query::LuaNeoQuery;
use crate::mapping::boltmap_to_lua_table;
use crate::runtime::{self};

pub struct LuaNeoTxn(pub Arc<Mutex<Txn>>);

register_lua_rstruct!(LuaNeoTxn, c"Neo4jTransaction", &[(c"Execute", execute)]);

#[lua_function]
pub fn execute(l: lua::State) -> anyhow::Result<i32> {
    let neo_tx = l.get_struct::<LuaNeoTxn>(1)?;
    let neo_query = l.get_struct::<LuaNeoQuery>(2)?;
    let callback = l.check_function(3)?;

    let tx = neo_tx.0.clone();
    let arc_query = neo_query.0.clone();
    let mut output = Vec::new();

    runtime::run_async(async move {
        let mut tx = tx.lock().await;
        let query = (*arc_query).clone();

        let mut results = tx.execute(query).await.unwrap();

        while let Some(ref row) = results.next(&mut *tx).await.unwrap() {
            match row.to::<BoltMap>() {
                Ok(entry) => {
                    let deserialized: BoltMap = entry;
                    output.push(deserialized.clone());
                }
                Err(err) => println!("{}", err),
            };
        }

        wait_lua_tick(move |l| {
            let _ = l.pcall_func_ref(callback, || {
                l.new_table();
                for (i, item) in output.iter().enumerate() {
                    if let Err(err) = boltmap_to_lua_table(l, item.clone()) {
                        println!("Could not convert boltmap: {}", err);
                        continue;
                    };
                    l.raw_seti(-2, i as i32 + 1);
                }
                1
            });
        });
    });

    Ok(0)
}
