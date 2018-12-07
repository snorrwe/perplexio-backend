use super::super::model::participation::{GameParticipation, GameParticipationDTO};
use super::super::model::user::User;
use super::super::service::auth::logged_in_user_from_cookie;
use super::super::service::config::Config;
use super::super::service::db_client::{db_client, Connection};
use chrono::{DateTime, Utc};
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
    client: &Connection,
) -> Option<GameParticipation> {
    client
        .query(
            &format!("{}{}", SELECT_PARTICIPATIONS, "WHERE u.id=$1 AND g.id=$2"),
            &[&user.id, &game_id],
        )
        .expect("Unexpected error while reading games")
        .iter()
        .map(|row| {
            let start: Option<DateTime<Utc>> = row.get(3);
            let end: Option<DateTime<Utc>> = row.get(4);
            GameParticipation {
                user_id: row.get(0),
                game_id: row.get(1),
                start_time: start,
                end_time: end,
            }
        })
        .next()
}

pub fn update_or_insert_participation(participation: GameParticipation, client: &Connection) {
    let insert_query = "
        INSERT INTO game_participations AS gp (user_id, game_id, start_time, end_time)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, game_id) DO
        UPDATE 
        SET
            start_time=COALESCE($3, gp.start_time),
            end_time=COALESCE($4, gp.end_time)
        WHERE gp.user_id=$1 AND gp.game_id=$2
        ";

    let transaction = client.transaction().expect("Failed to start transaction");
    transaction
        .query(
            insert_query,
            &[
                &participation.user_id,
                &participation.game_id,
                &participation.start_time,
                &participation.end_time,
            ],
        )
        .expect("Failed to insert participation");

    transaction.commit().expect("Failed to commit transaction");
}

