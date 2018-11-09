# Perplexio

## Setting up

- Install Cargo
- Install [Diesel](http://diesel.rs/guides/getting-started/) `cargo install diestl --no-default-features --features postgres`
- `cp example.env .env`
- Edit `.env` with your parameters

## Running the application

- `docker-compose up # start postgres`
- `diesel migration run # run migrations`
- `cargo run # start the application`

## Migrations

- Use the `diesel` cli tool
