use gmod::{LuaReference, wait_lua_tick};
use neo4rs::BoltMap;

use crate::mapping::boltmap_to_lua_table;

pub fn dispatch_callback(callback: LuaReference, results: anyhow::Result<Vec<BoltMap>>) {
    // Dispatch the callback
    wait_lua_tick(move |l| {
        let _ = l.pcall_func_ref(callback, || match results {
            Ok(output) => {
                // No error, so first value is nil
                l.push_nil();
                l.new_table();
                for (i, item) in output.iter().enumerate() {
                    let _ = boltmap_to_lua_table(l, item)
                        .map_err(|e| panic!("Could not convert bolt map to Lua table: {}", e));

                    l.raw_seti(-2, i as i32 + 1);
                }

                1
            }
            Err(err) => {
                // Error, so second value is nil
                l.push_string(err.to_string().as_str());
                l.push_nil();
                1
            }
        });
    });
}
