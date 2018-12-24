use super::super::schema::game_participations;
use chrono::{DateTime, Utc};

#[derive(Insertable)]
#[table_name = "game_participations"]
pub struct GameParticipation {
    pub user_id: i32,
    pub game_id: i32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Queryable)]
pub struct GameParticipationDTO {
    pub game_id: i32,
    pub game_name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Queryable)]
pub struct GameParticipationEntity {
    pub id: i32,
    pub user_id: i32,
    pub game_id: i32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl GameParticipationEntity {
    pub fn into_dto(self, game_name: String) -> GameParticipationDTO {
        GameParticipationDTO {
            game_id: self.game_id,
            game_name: game_name,
            start_time: self.start_time,
            end_time: self.end_time,
        }
    }
}

