web: ROCKET_PORT=$PORT ROCKET_ENVIRONMENT=staging ROCKET_DATABASES={perplexio={url=$DATABASE_URL}} ./target/release/perplexio
release: ./target/release/diesel database setup; ./target/release/diesel migration run
