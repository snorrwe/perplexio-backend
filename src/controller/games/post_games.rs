use super::super::super::model::game::{GameDTO, GameEntity, GameSubmission};
use super::super::super::model::puzzle::Puzzle;
use super::super::super::schema;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::config;
use super::super::super::service::db_client::diesel_client;
use super::get_games::get_game_by_user;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel::{insert_into, update};
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;

#[post("/games/regenerate_board/<game_id>")]
pub fn regenerate_board(
    game_id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    info!("Regenerating board for game [{}]", game_id);
    let current_user = logged_in_user!(cookies, config);
    use self::schema::games::dsl::*;

    let connection = diesel_client(&config);

    games
        .filter(id.eq(game_id))
        .get_result::<GameEntity>(&connection)
        .ok()
        .map_or(Err(Custom(Status::NotFound, "Game not found")), |game| {
            let p = Puzzle::from_words(game.words, 500).expect("Failed to generate puzzle");
            connection
                .transaction(|| {
                    update(games.filter(id.eq(game_id)))
                        .set(puzzle.eq(p.to_json()))
                        .execute(&connection)
                })
                .expect("Failed to commit transaction");
            let game =
                get_game_by_user(game_id, &current_user, &config).expect("Failed to get game");
            info!("Regenerating the board for game [{}] succeeded", game_id);
            Ok(Json(game))
        })
}

#[post("/game", data = "<game>")]
pub fn post_game(
    game: Json<GameSubmission>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<String, Custom<&'static str>> {
    info!("Creating new game {:?}", game);
    let current_user = logged_in_user!(cookies, config);

    let connection = diesel_client(&config);

    let puzz = Puzzle::from_words(game.words.clone(), 500).expect("Failed to create puzzle");
    let result = connection.transaction(|| {
        use self::schema::games::dsl::*;
        insert_into(games)
            .values((
                name.eq(game.name.clone()),
                owner_id.eq(current_user.id),
                words.eq(puzz.get_words()),
                puzzle.eq(puzz.to_json()),
            ))
            .execute(&connection)
    });

    handle_post_game_result(result)
}

fn handle_post_game_result(
    result: Result<usize, DieselError>,
) -> Result<String, Custom<&'static str>> {
    match result {
        Ok(id) => {
            info!("Creation of new game succeeded, id: {}", id);
            Ok(id.to_string())
        }
        Err(error) => {
            if let DieselError::DatabaseError(kind, _value) = &error {
                if let DatabaseErrorKind::UniqueViolation = kind {
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

