#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate postgres;
extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate oauth2;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate serde_json;
extern crate rand;
#[macro_use]
extern crate log;
extern crate chrono;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

pub mod controller;
pub mod model;
pub mod service;

use self::controller::games;
use self::controller::users;
use self::service::config::Config;

use dotenv::dotenv;

#[get("/")]
fn index() -> &'static str {
    "hello there"
}

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

    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                games::get_games,
                games::get_game,
                games::post_game,
                games::regenerate_board,
                users::login,
                users::register,
                users::index,
            ],
        )
        .attach(options)
        .manage(config)
        .launch();
}

