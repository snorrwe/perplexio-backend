use super::super::schema::game_participations;
use crate::fairing::DieselConnection;
use crate::model::user::User;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::update;
use juniper::FieldResult;

#[derive(Insertable)]
#[table_name = "game_participations"]
pub struct GameParticipation {
    pub user_id: i32,
    pub game_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct GameParticipationDTO {
    pub game_id: i32,
    pub game_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub user_name: String,
}

#[derive(Queryable)]
pub struct GameParticipationEntity {
    pub id: i32,
    pub user_id: i32,
    pub game_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
}

impl GameParticipationEntity {
    pub fn into_dto(self, game_name: String, user_name: String) -> GameParticipationDTO {
        GameParticipationDTO {
            game_id: self.game_id,
            game_name: game_name,
            start_time: self.start_time,
            end_time: self.end_time,
            user_name: user_name,
        }
    }
}

pub fn end_participation(
    client: &DieselConnection,
    user: &User,
    game_id: i32,
) -> FieldResult<bool> {
    use crate::schema::game_participations::dsl::{
        duration, end_time as et, game_id as gid, game_participations as gp, start_time as st,
        user_id,
    };

    let end_time = Utc::now();
    let start_time = gp
        .filter(user_id.eq(user.id).and(gid.eq(game_id)))
        .select((st,))
        .get_result::<(DateTime<Utc>,)>(&client.0)
        .optional()?
        .ok_or("User is not participating")?;

    let dur = (end_time - start_time.0).num_milliseconds();
    update(gp.filter(user_id.eq(user.id).and(gid.eq(game_id))))
        .set((et.eq(end_time), duration.eq(dur as i32)))
        .execute(&client.0)?;

    Ok(true)
}
