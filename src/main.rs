#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate postgres;
#[macro_use(database)]
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate dotenv;
extern crate oauth2;
extern crate rand;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
extern crate juniper_rocket;

pub mod entity;
pub mod fairing;
pub mod graphql;
pub mod guard;
pub mod handler;
pub mod model;
pub mod schema;
pub mod service;

use rocket::config::{Config as RocketConfig, Environment};
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

use crate::fairing::DieselConnection;
use crate::service::config::Config;

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
    debug_assert!(failed_origins.is_empty());
    if !failed_origins.is_empty() {
        error!("Failed origins: {:?}", failed_origins);
    }

    let cors_options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Delete,
            Method::Options,
        ]
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
            handler::graphiql,
            handler::graphql_handler,
            handler::users::login,
            handler::users::register,
            handler::users::user_info,
        ],
    )
    .attach(cors_options)
    .attach(DieselConnection::fairing())
    .manage(config)
    .manage(graphql::Schema::new(
        graphql::Query {},
        graphql::Mutation {},
    ))
    .launch();
}
