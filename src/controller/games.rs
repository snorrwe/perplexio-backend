use super::super::model::game::{GameDTO, GameId, GameSubmission};
use super::super::model::puzzle::Puzzle;
use super::super::model::user::User;
use super::super::service::auth::logged_in_user_from_cookie;
use super::super::service::config;
use super::super::service::db_client::db_client;
use chrono::{DateTime, Utc};
use postgres::rows::{Row, Rows};
use postgres::transaction::Transaction;
use postgres::Error as PostgresError;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
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
            let from: Option<DateTime<Utc>> = row.get(3);
            match &current_user {
                Some(current_user) => {
                    if current_user.id == row.get::<_, i32>(4) {
                        // Users can see their own games even if its not available yet
                        return true;
                    }
                }
                None => {}
            };
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
    db_client(&config)
        .query(
            "
            SELECT g.id, g.name, u.name, g.puzzle, g.owner_id, g.available_from
            FROM games g
            JOIN users u ON g.owner_id=u.id
            WHERE g.id=$1
            ",
            &[&id],
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
            let game = GameDTO {
                id: GameId {
                    id: row.get(0),
                    name: row.get(1),
                    owner: row.get(2),
                },
                table: table,
                is_owner: is_owner,
            };
            Json(game)
        })
        .next()
}

fn is_owner(current_user: &Option<User>, owner_id: i32) -> bool {
    if let Some(current_user) = &current_user {
        current_user.id == owner_id
    } else {
        false
    }
}

#[post("/games/regenerate_board/<id>")]
pub fn regenerate_board(
    id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<(), Custom<&'static str>> {
    info!("Regenerate game {}", id);
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(Custom(Status::Unauthorized, "Log in first"));
    }
    let current_user = current_user.unwrap();
    db_client(&config)
        .query(
            "
            SELECT g.id, g.words, g.owner_id
            FROM games g
            JOIN users u ON g.owner_id=u.id
            WHERE g.id=$1
            ",
            &[&id],
        )
        .expect("Failed to read games")
        .iter()
        .map(|row| handle_regenerate_board_result(row, &current_user, &config))
        .next()
        .unwrap_or(Err(Custom(Status::NotFound, "Game not found")))
}

fn handle_regenerate_board_result(
    row: Row,
    current_user: &User,
    config: &State<config::Config>,
) -> Result<(), Custom<&'static str>> {
    let owner_id: i32 = row.get(2);
    if owner_id != current_user.id {
        return Err(Custom(
            Status::Unauthorized,
            "You cannot alter someone else's game!",
        ));
    }

    let words = row.get(1);
    let puzzle = Puzzle::from_words(words, 500).expect("Failed to create puzzle");
    let id: i32 = row.get(0);

    let conn = db_client(&config);
    let transaction = conn.transaction().unwrap();

    transaction
        .query(
            "
            UPDATE games
            SET puzzle = $1
            WHERE id=$2
            ",
            &[&puzzle.to_json(), &id],
        )
        .expect("Failed to update the game");
    transaction
        .commit()
        .expect("Failed to commit the transaction, aborting");
    Ok(())
}

#[post("/game", data = "<game>")]
pub fn post_game(
    game: Json<GameSubmission>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<String, Custom<&'static str>> {
    info!("Creating new game {:?}", game);
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(Custom(Status::Unauthorized, "Log in first"));
    }
    let current_user = current_user.unwrap();

    let conn = db_client(&config);
    let transaction = conn.transaction().unwrap();

    let puzzle = Puzzle::from_words(game.words.clone(), 500).expect("Failed to create puzzle");
    let words = puzzle.get_words();

    let result = transaction.query(
        "
        INSERT INTO games (name, owner_id, puzzle, words)
        VALUES ($1, $2, $3, $4)
        RETURNING id;
        ",
        &[&game.name, &current_user.id, &puzzle.to_json(), &words],
    );

    handle_post_game_result(result, transaction)
}

fn handle_post_game_result(
    result: Result<Rows, PostgresError>,
    transaction: Transaction,
) -> Result<String, Custom<&'static str>> {
    match result {
        Ok(inserted) => {
            transaction
                .commit()
                .expect("Failed to commit the transaction, aborting");
            let id: i32 = inserted.iter().map(|row| row.get(0)).next().unwrap();
            Ok(id.to_string())
        }
        Err(error) => {
            if let Some(error) = error.code() {
                if error.code() == "23505" {
                    return Err(Custom(
                        Status::BadRequest,
                        "Game with given name already exists",
                    ));
                }
            }
            error!(
                "Unexpected error happened while inserting new game {}",
                error
            );
            Err(Custom(
                Status::InternalServerError,
                "Unexpected error while inserting the game",
            ))
        }
    }
}
