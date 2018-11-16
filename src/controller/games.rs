use super::super::model::game::{GameDTO, GameId, GameSubmission};
use super::super::model::puzzle::Puzzle;
use super::super::service::auth;
use super::super::service::config;
use super::super::service::db_client::db_client;
use rocket::http::{Cookies, Status};
use rocket::response::status;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/games")]
pub fn get_games(config: State<config::Config>) -> Json<Vec<GameId>> {
    let result = db_client(&config)
        .query(
            "
            SELECT g.id, g.name, u.name
            FROM games g
            JOIN users u
            ON g.owner_id=u.id
            ",
            &[],
        )
        .expect("Unexpected error while reading games")
        .iter()
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
    let current_user = auth::logged_in_user_from_cookie(&mut cookies, &config);
    db_client(&config)
        .query(
            "
        SELECT g.id, g.name, u.name, g.puzzle, g.owner_id
        FROM games g
        JOIN users u ON g.owner_id=u.id
        WHERE g.id=$1
        ",
            &[&id],
        )
        .expect("Failed to read games")
        .iter()
        .map(|row| {
            let mut table: Value = row.get(3);
            let is_owner = if let Some(current_user) = &current_user {
                current_user.id == row.get::<_, i32>(4)
            } else {
                false
            };
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

#[post("/games/regenerate_board/<id>")]
pub fn regenerate_board(
    id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, status::Custom<String>> {
    let current_user = auth::logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(status::Custom(
            Status::Unauthorized,
            "Log in first".to_string(),
        ));
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
        .map(|row| {
            let owner_id: i32 = row.get(2);
            if owner_id != current_user.id {
                Err(status::Custom(
                    Status::Unauthorized,
                    "You cannot alter someone else's game".to_string(),
                ))
            } else {
                // TODO implemented puzzle update and return the new puzzle
                unimplemented!()
            }
        })
        .next()
        .unwrap_or(Err(status::Custom(Status::NotFound, "".to_string())))
}

#[post("/game", data = "<game>")]
pub fn post_game(
    game: Json<GameSubmission>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<String, status::Custom<String>> {
    let current_user = auth::logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(status::Custom(
            Status::Unauthorized,
            "Log in first".to_string(),
        ));
    }
    let current_user = current_user.unwrap();

    let conn = db_client(&config);

    let transaction = conn.transaction().unwrap();

    let puzzle = Puzzle::from_words(game.words.clone(), 500).expect("Failed to create puzzle");
    let words = puzzle.get_words();

    let res = transaction.query(
        "
        INSERT INTO games (name, owner_id, puzzle, words)
        VALUES ($1, $2, $3, $4)
        RETURNING id;
        ",
        &[&game.name, &current_user.id, &puzzle.to_json(), &words],
    );

    match res {
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
                    return Err(status::Custom(
                        Status::BadRequest,
                        "Game with given name already exists".to_string(),
                    ));
                }
            }
            error!(
                "Unexpected error happened while inserting new game {}",
                error
            );
            Err(status::Custom(
                Status::InternalServerError,
                format!("Unexpected error while inserting the game"),
            ))
        }
    }
}
