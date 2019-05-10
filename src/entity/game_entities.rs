#![allow(proc_macro_derive_resolution_fallback)]
use super::super::schema::games;
use chrono::{DateTime, Utc};

pub type Date = DateTime<Utc>;

#[derive(Queryable)]
pub struct GameEntity {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
    pub available_from: Option<Date>,
    pub available_to: Option<Date>,
    pub published: bool,
}

#[derive(Insertable)]
#[table_name = "games"]
pub struct GameInsert {
    pub name: String,
    pub owner_id: i32,
    pub available_from: Option<Date>,
    pub available_to: Option<Date>,
    pub published: bool,
}

#[derive(AsChangeset)]
#[table_name = "games"]
pub struct GameUpdate {
    pub name: Option<String>,
    pub available_from: Option<Date>,
    pub available_to: Option<Date>,
}

