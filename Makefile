ping_db:

	pg_isready --dbname=feather --host=localhost --port=5432 --username=feather || echo "Connection to DB failed!"
	

migrate:
	psql postgresql://feather:feather@localhost:5432/feather -c "CREATE EXTENSION IF NOT EXISTS citext;"
	psql postgresql://feather:feather@localhost:5432/feather -c "CREATE EXTENSION IF NOT EXISTS semver;"
	refinery migrate -d files -p ./migrations