use super::super::model::participation::{
    GameParticipation, GameParticipationDTO, GameParticipationEntity,
};
use super::super::model::user::User;
use super::super::service::auth::logged_in_user_from_cookie;
use super::super::service::config::Config;
use super::super::service::db_client::{db_client, Connection, DieselConnection};
use chrono::{DateTime, Utc};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;

const SELECT_PARTICIPATIONS: &'static str = "
SELECT p.user_id, p.game_id, g.name, p.start_time, p.end_time
FROM game_participations p
JOIN games g
ON p.game_id=g.id
JOIN users u
ON p.user_id=u.id
";

#[get("/participations")]
pub fn get_participations(
    mut cookies: Cookies,
    config: State<Config>,
) -> Result<Json<Vec<GameParticipationDTO>>, Custom<&'static str>> {
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(Custom(Status::NotFound, "Log in first"));
    }
    let current_user = current_user.unwrap();
    let client = db_client(&config);
    let result = client
        .query(
            &format!("{}{}", SELECT_PARTICIPATIONS, "WHERE u.id=$1"),
            &[&current_user.id],
        )
        .expect("Unexpected error while reading games")
        .iter()
        .map(|row| {
            let start: Option<DateTime<Utc>> = row.get(3);
            let end: Option<DateTime<Utc>> = row.get(4);
            GameParticipationDTO {
                game_name: row.get(2),
                start_time: start.map_or(None, |t| Some(t.to_string())),
                end_time: end.map_or(None, |t| Some(t.to_string())),
            }
        })
        .collect();

    Ok(Json(result))
}

pub fn get_participation(
    user: &User,
    game_id: i32,
    client: &DieselConnection,
) -> Option<GameParticipationEntity> {
    use super::super::schema::game_participations::dsl::game_id as gid;
    use super::super::schema::game_participations::dsl::*;

    game_participations
        .filter(user_id.eq(user.id).and(gid.eq(game_id)))
        .get_result(client)
        .ok()
}

pub fn insert_participation(participation: GameParticipation, client: &DieselConnection) {
    use super::super::schema::game_participations::dsl::*;

    insert_into(game_participations)
        .values(&participation)
        .execute(client)
        .expect("Failed to insert participation");
}

