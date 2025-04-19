require("neo4j")

-- URI, User, Password
local graph = neo4j.Graph("bolt://localhost:7474", "neo4j", "MyNeo4jPassword")

local query = neo4j.Query("MATCH (u:User)-[:MEMBER_OF]->(ug:Usergroup) RETURN u, ug", {}) -- params

-- Simple execution mode:
graph:Execute(query, function(error, result)
	if error then return print(error) end
	for _, value in ipairs(result) do
		local usergroup = value.ug
		local user = value.u

		PrintTable(user)
		PrintTable(usergroup)
	end
end)

-- Execute with a specific db
local db = "mycommunitydb"
graph:ExecuteOn(db, query, function(error, result)
	if error then return print(error) end
	PrintTable(result)
end)

-- In depth transaction control:
local tx = graph:Tx()
tx:Execute(query, function(error, result)
	if error then return print(error) end

	for _, value in ipairs(result) do
		local usergroup = value.ug
		local user = value.u

		PrintTable(user)
		PrintTable(usergroup)
	end
end)
tx:Commit()

-- In depth transaction control with a specific db:
local db = "yourdb"
local tx = graph:TxOn(db)
tx:ExecuteOn(query, function(error, result)
	if error then return print(error) end

	for _, value in ipairs(result) do
		local usergroup = value.ug
		local user = value.u

		PrintTable(user)
		PrintTable(usergroup)
	end
end)
tx:Commit()
