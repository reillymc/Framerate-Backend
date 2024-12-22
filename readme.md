# Framerate Backend

## Requirements

-   Diesel CLI

## Develop

Watch for changes with `cargo watch -x run`

Environment variables must be exported before running the program. Variables are automatically provided to the dev container via `./.devcontianer/.env` (changes require rebuild), and in the production container when specifying `env_file: .env`. These can be manually exported to override variables set by the container, or to run the program outside of a container. `dotenvy` was previously used to load from file, however it was removed to minimise dependencies in order to speed up ARM builds in GitHub Actions. It may be re-added in future once ARM runners are available.

## Database

Diesel allows SQL migrations to be generated from rust schema files or rust schema from a new SQL migration.

To run migrations, use `diesel migration run`

## Testing

The test database can be set up / updated by running `DATABASE_URL=postgres://${TEST_POSTGRES_USER}:${TEST_POSTGRES_PASSWORD}@${TEST_POSTGRES_HOST}/${POSTGRES_DB} && diesel migration run`
