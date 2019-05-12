extern crate arrayvec;
extern crate postgres;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate dotenv;
extern crate futures;
extern crate oauth2;
extern crate rand;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
extern crate simple_logger;

pub mod entity;
pub mod fairing;
pub mod graphql;
pub mod guard;
pub mod handler;
pub mod model;
pub mod schema;
pub mod service;

use crate::graphql::{mutation::Mutation, query::Query, Schema};
use crate::service::config::Config;

use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use std::sync::Arc;

fn main() {
    dotenv().ok();
    simple_logger::init_with_level(log::Level::Debug).expect("Failed to init logging");

    let config = Arc::new(Config::get());
    let schema = Arc::new(Schema::new(Query, Mutation));

    // TODO: db connection pool

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(schema.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to_async(handler::graphiql)))
            .service(
                web::resource("/graphql").route(web::post().to_async(handler::graphql_handler)),
            )
    })
    .bind("localhost:8000")
    .expect("Failed to start the application")
    .run()
    .unwrap();
}
