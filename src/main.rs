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
pub mod graphql;
pub mod handler;
pub mod model;
pub mod schema;
pub mod service;

use crate::graphql::{mutation::Mutation, query::Query, Schema};
use crate::service::config::Config;
use actix_web::http::header;
use actix_web::middleware::cors::Cors;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpServer};
pub use diesel::pg::PgConnection as DieselConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::sync::Arc;

pub type ConnectionPool = r2d2::Pool<ConnectionManager<DieselConnection>>;

fn main() {
    dotenv().ok();
    simple_logger::init_with_level(log::Level::Debug).expect("Failed to init logging");

    let config = Arc::new(Config::get());
    let schema = Arc::new(Schema::new(Query {}, Mutation {}));

    let manager = ConnectionManager::<DieselConnection>::new(config.postgres_url.as_str());
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = format!("{}:{}", config.address, config.port);

    HttpServer::new(move || {
        let mut cors = Cors::new();

        for url in config.allowed_origins.iter() {
            cors = cors.allowed_origin(url.as_str());
        }

        App::new()
            .data(config.clone())
            .data(schema.clone())
            .data(pool.clone())
            .wrap(
                cors.allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::CONTENT_TYPE])
                    .supports_credentials(),
            )
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(
                    config
                        .auth_private_key
                        .as_ref()
                        .map(|x| x.as_bytes())
                        .unwrap_or(&[42; 64]),
                )
                .name("Authorization")
                .secure(config.secure),
            ))
            .service(web::resource("/").route(web::get().to_async(handler::graphiql)))
            .service(
                web::resource("/graphql").route(web::post().to_async(handler::graphql_handler)),
            )
            .service(web::resource("/login").route(web::get().to_async(handler::users::login)))
            .service(web::resource("/logout").route(web::get().to_async(handler::users::logout)))
    })
    .bind(&bind)
    .expect("Failed to start the application")
    .run()
    .unwrap();
}

