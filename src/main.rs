#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

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

use crate::fairing::DieselConnection;
use crate::graphql::{mutation::Mutation, query::Query, Schema};
use crate::service::config::Config;
use std::sync::Arc;
use actix_web::{server, App, HttpRequest};
use actix::Addr;
use dotenv::dotenv;

pub struct State {
    connection: Addr<DieselConnection>,
}

fn main() {
    dotenv().ok();
    simple_logger::init_with_level(log::Level::Debug).expect("Failed to init logging");

    let config = Config::get();
    let schema = Arc::new(Schema::new(Query, Mutation));

    server::new(move || {
        App::new()
            .resource("/", |r| r.get().a(|r| self::handler::graphiql()))
            .resource("/graphql", |r| {
                r.post()
                    .a(|r| self::handler::graphql_handler(schema.clone(), connection, request))
            })
    })
    .bind("localhost:8000")
    .expect("Failed to start the application")
    .run()
}

