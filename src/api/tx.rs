use neo4rs::{BoltMap, Query, Txn};
use rglua::lua::{LuaState, lua_newtable, lua_pushnil, lua_rawseti};

use crate::{
    LogLevel, log,
    mapping::boltmap_to_lua_table,
    neo_client::THREAD_WORKER,
    userdata::{read_userdata_at, read_userdata_owned},
};

/// Executes a query within a transaction.
/// Will retrieve the entire result set at once.
#[lua_function]
pub fn execute(l: LuaState) -> i32 {
    THREAD_WORKER.block_on(async {
        let mut tx = match read_userdata_owned::<Txn>(l, 1) {
            Ok(tx) => tx,
            Err(err) => {
                log(
                    LogLevel::Error,
                    format!("Failed to fetch existing transaction: {}", err),
                );
                return 0;
            }
        };
        let query = match read_userdata_at::<Query>(l, 2) {
            Ok(query) => query,
            Err(err) => {
                log(LogLevel::Error, format!("Failed to fetch query: {}", err));
                return 0;
            }
        };

        let mut results = match tx.execute(query).await {
            Ok(results) => results,
            Err(err) => {
                log(
                    LogLevel::Error,
                    format!("Failed to fetch query results: {}", err),
                );
                return 0;
            }
        };

        let mut output: Vec<BoltMap> = Vec::new();
        let next = match results.next(&mut tx.handle()).await {
            Ok(next) => next,
            Err(err) => {
                log(
                    LogLevel::Error,
                    format!("Failed to fetch next row: {}", err),
                );
                return 0;
            }
        };

        while let Some(ref row) = next {
            match row.to::<BoltMap>() {
                Ok(entry) => {
                    let deserialized: BoltMap = entry;
                    output.push(deserialized);
                }
                Err(_) => (),
            };
        }

        if output.is_empty() {
            lua_pushnil(l);
            return 1;
        }

        lua_newtable(l);
        for (i, doc) in output.iter().enumerate() {
            boltmap_to_lua_table(l, doc.clone());
            lua_rawseti(l, -2, i as i32 + 1);
        }
        1
    })
}

/// Executes a query within a transaction.
/// Will retrieve the entire result set at once.
#[lua_function]
pub fn commit(l: LuaState) -> i32 {
    let tx = match read_userdata_owned::<Txn>(l, 1) {
        Ok(tx) => tx,
        Err(err) => {
            log(
                LogLevel::Error,
                format!("Failed to fetch existing transaction: {}", err),
            );
            return 0;
        }
    };

    THREAD_WORKER.block_on(async {
        match tx.commit().await {
            Ok(_) => 0,
            Err(err) => {
                log(
                    LogLevel::Error,
                    format!("Failed to commit transaction: {}", err),
                );
                0
            }
        }
    })
}

/// Executes a query within a transaction.
/// Will retrieve the entire result set at once.
#[lua_function]
pub fn close(l: LuaState) -> i32 {
    let txn_result = read_userdata_owned::<Txn>(l, 1);
    if txn_result.is_err() {
        log(LogLevel::Error, "Failed to close transaction.");
        return 0;
    }

    let tx = txn_result.unwrap();
    let _ = drop(tx);

    0
}
