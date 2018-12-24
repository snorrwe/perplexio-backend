use chrono::{DateTime, Utc};
use super::super::schema::game_participations;

#[derive(Insertable)]
#[table_name="game_participations"]
pub struct GameParticipation {
    pub user_id: i32,
    pub game_id: i32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct GameParticipationDTO {
    pub game_name: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Queryable)]
pub struct GameParticipationEntity{
    pub id: i32,
    pub user_id: i32,
    pub game_id: i32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

