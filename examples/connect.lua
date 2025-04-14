require("neo4j")

-- URI, User, Password
local graph = neo4j.Graph("bolt://localhost:7474", "neo4j", "MyNeo4jPassword")

local query = neo4j.Query("MATCH (u:User)-[:MEMBER_OF]->(ug:Usergroup) RETURN u, ug", {}) -- params
local tx = graph:Tx()
local result = tx:Execute(query)

for _, value in ipairs(result) do
	local usergroup = value.ug
	local user = value.u

	PrintTable(user)
	PrintTable(usergroup)
end
tx:Close()