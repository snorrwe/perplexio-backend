# Perplexio

[![Build Status](https://travis-ci.org/snorrwe/perplexio-backend.svg?branch=master)](https://travis-ci.org/snorrwe/perplexio-backend)

## Setting up

- Install [rustup](https://github.com/rust-lang/rustup.rs#installation)
- Install [libpq](https://www.postgresql.org/download/)
- Setup your environment for libpq, see [pq-sys](https://github.com/sgrif/pq-sys#building)
- Install [Diesel](http://diesel.rs/guides/getting-started/) `cargo install diesel --no-default-features --features postgres`
- `cp example.env .env`
- Edit `.env` with your parameters

## Running the application

- `docker-compose up # start postgres`
- `diesel migration run # run migrations`
- `cargo run # start the application`

## Migrations

- Use the `diesel` cli tool

## Integration Testing

- Have a PostgreSQL instance up and running
- Add a user named `test` with password: `almafa1`
- Add a database named `test` with owner `test`
- Run migrations `diesel migration --database-url="postgres://test:almafa1@localhost:5432" run`
- Run `cargo test -- --ignored`
