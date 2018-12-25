#![allow(proc_macro_derive_resolution_fallback)]
use super::super::schema::games;
use chrono::{DateTime, Utc};
use serde_json::Value;

pub type Date = Option<DateTime<Utc>>;

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct GameId {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub available_from: Date,
    pub available_to: Date,
}

#[derive(Queryable)]
pub struct GameIdQuery {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub available_from: Date,
    pub available_to: Date,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameSubmission {
    pub name: String,
    pub words: Vec<String>,
    pub available_from: Date,
    pub available_to: Date,
}

#[derive(Deserialize, AsChangeset)]
#[serde(rename_all = "camelCase")]
#[table_name = "games"]
pub struct GameUpdateForm {
    pub name: Option<String>,
    pub available_from: Date,
    pub available_to: Date,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDTO {
    pub id: GameId,
    pub table: Value,
    pub is_owner: bool,
}

#[derive(Queryable)]
pub struct GameEntity {
    pub id: i32,
    pub name: String,
    pub owner_id: i32,
    pub puzzle: Value,
    pub words: Vec<String>,
    pub available_from: Date,
    pub available_to: Date,
}

impl GameEntity {
    pub fn into_dto(self, owner: String, is_owner: bool) -> GameDTO {
        GameDTO {
            id: GameId {
                id: self.id,
                name: self.name,
                owner: owner,
                available_from: self.available_from,
                available_to: self.available_to,
            },
            table: self.puzzle,
            is_owner: is_owner,
        }
    }
}

