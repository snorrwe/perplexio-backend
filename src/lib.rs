#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate postgres;
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

pub mod controller;
pub mod model;
pub mod schema;
pub mod service;
