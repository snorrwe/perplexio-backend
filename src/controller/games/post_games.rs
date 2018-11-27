use super::super::super::model::game::{GameDTO, GameSubmission};
use super::super::super::model::puzzle::Puzzle;
use super::super::super::model::user::User;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::config;
use super::super::super::service::db_client::db_client;
use super::get_games::get_game_by_user;
use postgres::rows::{Row, Rows};
use postgres::transaction::Transaction;
use postgres::Error as PostgresError;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;

#[post("/games/regenerate_board/<id>")]
pub fn regenerate_board(
    id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
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
) -> Result<Json<GameDTO>, Custom<&'static str>> {
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
    let game = get_game_by_user(id, &Some(current_user.clone()), &config).expect("");
    Ok(Json(game))
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
