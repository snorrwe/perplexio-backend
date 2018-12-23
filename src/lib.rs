#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

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
#[macro_use]
extern crate diesel;

pub mod controller;
pub mod model;
pub mod schema;
pub mod service;

