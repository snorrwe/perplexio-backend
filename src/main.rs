#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate postgres;
extern crate rocket_contrib;
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

pub mod controller;
pub mod model;
pub mod service;

use self::controller::games;
use self::controller::users;

use dotenv::dotenv;

fn main() {
    dotenv().ok();

    rocket::ignite()
        .mount(
            "/",
            routes![
                games::get_games,
                games::get_game,
                games::post_game,
                users::login,
                users::register,
                users::index,
            ],
        )
        .manage(service::config::Config::get())
        .launch();
}

