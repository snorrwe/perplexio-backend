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

pub mod controller;
pub mod model;
pub mod service;

use self::controller::games;
use self::controller::users;
use self::service::auth;

use dotenv::dotenv;
use rocket::http::Cookies;
use rocket::response;
use rocket::State;

#[get("/")]
fn index(mut cookies: Cookies, config: State<service::config::Config>) -> response::Redirect {
    if auth::logged_in_user_from_cookie(&mut cookies, &config).is_some() {
        let uri = uri!(games::get_games);
        response::Redirect::to(uri)
    } else {
        let uri = auth::client(&config).authorize_url().to_string();
        response::Redirect::to(uri)
    }
}

fn main() {
    dotenv().ok();

    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                games::get_games,
                games::get_game,
                games::post_game,
                users::login,
                users::register
            ],
        )
        .manage(service::config::Config::get())
        .launch();
}

