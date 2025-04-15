# gmsv_neo4j
This module enables the use of Neo4j with Garry's Mod.
Written as a Rust module, it integrates the [neo4rs](https://github.com/neo4j-labs/neo4rs) crate.


## Example usage
This is *work in progress*.

```lua
require("neo4j")

local graph = neo4j.Graph(config.uri, config.username, config.password, { db = config.database })

function exampleQueryFunction(ply)
    return neo4j.Query("MATCH (u:User {steamId: $steamId}", { steamId = ply:SteamID64() })
end


local tx = graph:Tx()
tx:Execute(exampleQueryFunction(ply), function(result)
    -- Handle your results here.
    PrintTable(result)
end)
tx:Commit()
```