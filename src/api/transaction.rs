use std::sync::Arc;

use gmod::rstruct::RStruct;
use gmod::{lua, lua_function, register_lua_rstruct};
use neo4rs::{BoltMap, Query, Txn};
use tokio::sync::Mutex;

use crate::api::query::LuaNeoQuery;
use crate::api::result::dispatch_callback;
use crate::runtime::{self};

pub struct LuaNeoTxn(pub Arc<Mutex<Option<Txn>>>);

register_lua_rstruct!(LuaNeoTxn, c"Neo4jTransaction", &[
    (c"Execute", execute),
    (c"Commit", commit),
]);

async fn handle_execution<'a>(
    mut guard: tokio::sync::MutexGuard<'a, Option<Txn>>,
    query: Query,
) -> anyhow::Result<Vec<BoltMap>> {
    // Safely get mutable reference to Txn inside Option
    let tx_ref = guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Transaction has already been committed or consumed"))?;

    let result = async {
        let mut results = tx_ref.execute(query).await?;

        let mut output = Vec::new();
        while let Some(row) = results.next(&mut *tx_ref).await? {
            match row.to::<BoltMap>() {
                Ok(entry) => output.push(entry),
                Err(err) => return Err(anyhow::anyhow!("Row conversion error: {}", err)),
            }
        }

        Ok(output)
    }
    .await;

    // If an error occurred, rollback
    if let Err(ref e) = result {
        // Take ownership of txn to rollback it only once
        if let Some(txn) = guard.take() {
            let _ = txn.rollback().await.map_err(|rollback_err| {
                eprintln!("Rollback failed after error {}: {}", e, rollback_err);
            });
        }
    }

    result
}

#[lua_function]
pub fn execute(l: lua::State) -> anyhow::Result<i32> {
    let neo_tx = l.get_struct::<LuaNeoTxn>(1)?;
    let neo_query = l.get_struct::<LuaNeoQuery>(2)?;
    let callback = l.check_function(3)?;

    let tx_mutex = neo_tx.0.clone();
    let arc_query = neo_query.0.clone();

    runtime::run_async(async move {
        let results = {
            let guard = tx_mutex.lock().await;
            let query = (*arc_query).clone();

            handle_execution(guard, query).await
        };

        dispatch_callback(callback, results);
    });

    Ok(0)
}

#[lua_function]
pub fn commit(l: lua::State) -> anyhow::Result<i32> {
    let neo_tx = l.get_struct::<LuaNeoTxn>(1)?;

    let tx_mutex = neo_tx.0.clone();

    runtime::run_async(async move {
        let mut guard = tx_mutex.lock().await;
        if let Some(txn) = guard.take() {
            println!("Commit tx");
            txn.commit().await
        } else {
            Err(neo4rs::Error::UnexpectedMessage(
                "Could not commit transaction".to_string(),
            ))
        }
    });

    Ok(0)
}
