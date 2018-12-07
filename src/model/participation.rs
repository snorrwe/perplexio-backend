use chrono::{DateTime, Utc};

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

