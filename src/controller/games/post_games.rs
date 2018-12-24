use super::super::super::model::game::{GameDTO, GameEntity, GameSubmission, GameUpdateForm};
use super::super::super::model::puzzle::Puzzle;
use super::super::super::model::user::User;
use super::super::super::schema;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::config;
use super::super::super::service::db_client::{diesel_client, DieselConnection};
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
    use self::schema::games::dsl::*;
    info!("Regenerating board for game [{}]", game_id);

    let connection = diesel_client(&config);
    let current_user = logged_in_user!(connection, cookies);

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
                get_game_by_user(&connection, game_id, &current_user).expect("Failed to get game");
            info!("Regenerating the board for game [{}] succeeded", game_id);
            Ok(Json(game))
        })
}

#[put("/game/<game_id>", data = "<game>")]
pub fn update_game(
    game_id: i32,
    game: Json<GameUpdateForm>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    use self::schema::games::dsl::*;
    info!("Updating game [{}]", game_id);

    let connection = diesel_client(&config);
    let current_user = logged_in_user!(connection, cookies);

    update(games.filter(id.eq(game_id).and(owner_id.eq(current_user.id))))
        .set(game.into_inner())
        .execute(&connection)
        .expect("Failed to update game");

    let game = get_game_by_user(&connection, game_id, &current_user).expect("Failed to get game");
    info!("Updating game [{}] succeeded", game_id);
    Ok(Json(game))
}

#[post("/game", data = "<game>")]
pub fn post_game(
    game: Json<GameSubmission>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    info!("Creating new game {:?}", game);

    let connection = diesel_client(&config);
    let current_user = logged_in_user!(connection, cookies);

    let puzz = Puzzle::from_words(game.words.clone(), 500).expect("Failed to create puzzle");
    let result = connection.transaction(|| {
        use self::schema::games::dsl::*;
        insert_into(games)
            .values((
                name.eq(game.name.clone()),
                owner_id.eq(current_user.id),
                words.eq(puzz.get_words()),
                puzzle.eq(puzz.to_json()),
                available_from.eq(game.available_from),
                available_to.eq(game.available_to),
            ))
            .execute(&connection)
    });

    handle_post_game_result(result, &connection, &current_user)
}

fn handle_post_game_result(
    result: Result<usize, DieselError>,
    connection: &DieselConnection,
    current_user: &User,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    match result {
        Ok(id) => {
            info!("Creation of new game succeeded, id: {}", id);
            let game =
                get_game_by_user(connection, id as i32, &current_user).expect("Failed to get game");
            Ok(Json(game))
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

