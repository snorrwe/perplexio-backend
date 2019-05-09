#![allow(proc_macro_derive_resolution_fallback)]
use super::super::schema::games;
use chrono::{DateTime, Utc};

pub type Date = Option<DateTime<Utc>>;

#[derive(Queryable)]
pub struct GameEntity {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
    pub available_from: Date,
    pub available_to: Date,
    pub published: bool,
}

#[derive(Insertable)]
#[table_name = "games"]
pub struct GameInsert {
    pub name: String,
    pub owner_id: i32,
    pub available_from: Date,
    pub available_to: Date,
    pub published: bool,
}
