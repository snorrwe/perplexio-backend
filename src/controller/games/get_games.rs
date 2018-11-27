use super::super::super::model::game::{GameDTO, GameId};
use super::super::super::model::user::User;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::config;
use super::super::super::service::db_client::db_client;
use chrono::{DateTime, Utc};
use rocket::http::Cookies;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/games")]
pub fn get_games(mut cookies: Cookies, config: State<config::Config>) -> Json<Vec<GameId>> {
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    let result = db_client(&config)
        .query(
            "
            SELECT g.id, g.name, u.name, g.available_from, u.id
            FROM games g
            JOIN users u
            ON g.owner_id=u.id
            ",
            &[],
        )
        .expect("Unexpected error while reading games")
        .iter()
        .filter(|row| {
            if let Some(current_user) = &current_user {
                if current_user.id == row.get::<_, i32>(4) {
                    // Users can see their own games even if its not available yet
                    return true;
                }
            }
            let from: Option<DateTime<Utc>> = row.get(3);
            match from {
                Some(from) => from > Utc::now(),
                None => false,
            }
        })
        .map(|row| GameId {
            id: row.get(0),
            name: row.get(1),
            owner: row.get(2),
        })
        .collect();
    Json(result)
}

#[get("/game/<id>")]
pub fn get_game(
    id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Option<Json<GameDTO>> {
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    let game = get_game_by_user(id, &current_user, &config);
    if let Some(game) = game {
        Some(Json(game))
    } else {
        None
    }
}

pub fn get_game_by_user(
    game_id: i32,
    current_user: &Option<User>,
    config: &config::Config,
) -> Option<GameDTO> {
    db_client(&config)
        .query(
            "
            SELECT g.id, g.name, u.name, g.puzzle, g.owner_id, g.available_from, g.available_to
            FROM games g
            JOIN users u ON g.owner_id=u.id
            WHERE g.id=$1
            ",
            &[&game_id],
        )
        .expect("Failed to read games")
        .iter()
        .filter(|row| {
            let is_owner = is_owner(&current_user, row.get::<_, i32>(4));
            if !is_owner {
                let available_from: Option<DateTime<Utc>> = row.get(5);
                if let Some(available_from) = available_from {
                    if available_from < Utc::now() {
                        return false;
                    }
                }
            }
            true
        })
        .map(|row| {
            let is_owner = is_owner(&current_user, row.get::<_, i32>(4));
            let mut table: Value = row.get(3);
            if !is_owner {
                // Remove the solutions field
                table["solutions"].take();
            }
            let available_from = date_to_string(row.get(5));
            let available_to = date_to_string(row.get(6));
            let game = GameDTO {
                id: GameId {
                    id: row.get(0),
                    name: row.get(1),
                    owner: row.get(2),
                },
                table: table,
                is_owner: is_owner,
                available_from: available_from,
                available_to: available_to,
            };
            game
        })
        .next()
}

fn date_to_string(date: Option<DateTime<Utc>>) -> Option<String> {
    if let Some(date) = date {
        Some(date.to_string())
    } else {
        None
    }
}

fn is_owner(current_user: &Option<User>, owner_id: i32) -> bool {
    if let Some(current_user) = &current_user {
        current_user.id == owner_id
    } else {
        false
    }
}
