#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;

use rocket::config::{Config as RocketConfig, Environment};
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

use perplexio::controller;
use perplexio::controller::games;
use perplexio::controller::participations;
use perplexio::controller::solutions;
use perplexio::controller::users;
use perplexio::service::config::Config;

use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let config = Config::get();

    let allowed_origins: Vec<&str> = config
        .allowed_origins
        .iter()
        .map(|string| string.as_str())
        .collect();
    let (allowed_origins, failed_origins) = AllowedOrigins::some(allowed_origins.as_slice());
    assert!(failed_origins.is_empty());

    let options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    };
    let app = if config.heroku {
        let rocket_config = RocketConfig::build(Environment::Production)
            .address(config.address.clone())
            .port(config.port)
            .finalize()
            .expect("Failed to init custom rocket options");
        rocket::custom(rocket_config)
    } else {
        rocket::ignite()
    };
    app.mount(
        "/",
        routes![
            controller::index,
            games::get_games,
            games::get_game,
            games::post_game,
            games::update_game,
            games::regenerate_board,
            users::login,
            users::user_info,
            users::register,
            solutions::get_solution_by_game_id,
            solutions::submit_solutions,
            participations::get_participations,
            participations::get_participation,
        ],
    )
    .attach(options)
    .manage(config)
    .launch();
}

