# Manage the database with diesel_cli

1. Install the diesel_cli tool 
`cargo install diesel_cli`

2. Create empty database file
`touch shortwave.db`

3. Create new migration
`diesel migration generate station_uuid --database-url=./shortwave.db --migration-dir=./data/database/migrations/`

4. List all migrations
`diesel migration list --database-url=./shortwave.db --migration-dir=./data/database/migrations/`

5. Run the migrations (onto the shortwave.db file)
`diesel migration run --database-url=./shortwave.db --migration-dir=./data/database/migrations/`

*Note: All of those commands are getting executed from the project root folder*
