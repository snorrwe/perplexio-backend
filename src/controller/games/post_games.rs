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

/// Regenerate the puzzle of the game specified by ID
/// Returns the new Game
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
        .filter(id.eq(game_id).and(owner_id.eq(current_user.id)))
        .get_result::<GameEntity>(&connection)
        .ok()
        .map_or(Err(Custom(Status::NotFound, "Game not found")), |game| {
            regenerate_game_board(game, &connection, &current_user)
        })
}

fn regenerate_game_board(
    game: GameEntity,
    connection: &DieselConnection,
    current_user: &User,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    use self::schema::games::dsl::*;

    let puzz = Puzzle::from_words(game.words, 500)
        .map_err(|e| {
            error!("{:#?}", e);
            Custom(Status::InternalServerError, "Failed to regenerate puzzle")
        })?
        .to_json();
    update(games.filter(id.eq(game.id)))
        .set(puzzle.eq(puzz))
        .execute(connection)
        .map_err(|e| {
            error!("{:#?}", e);
            Custom(Status::InternalServerError, "Failed to update games")
        })?;
    let game = get_game_by_user(&connection, game.id, &current_user)
        .ok_or(Custom(Status::InternalServerError, "Failed to get game"))?;
    info!("Regenerating the board for game [{}] succeeded", game.id.id);
    Ok(Json(game))
}

/// Update the parameters of the game specified by the ID
/// Returns the new Game
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

    if let Some(ref w) = game.words {
        let puzz = Puzzle::from_words(w.clone(), 500)
            .map_err(|_| Custom(Status::InternalServerError, "Failed to create puzzle"))?
            .to_json();
        update(games.filter(id.eq(game_id).and(owner_id.eq(current_user.id))))
            .set((words.eq(w), puzzle.eq(puzz)))
            .execute(&connection)
            .map_err(|_| Custom(Status::InternalServerError, "Failed to update puzzle"))?;
    }

    update(games.filter(id.eq(game_id).and(owner_id.eq(current_user.id))))
        .set(game.into_inner())
        .execute(&connection)
        .map_err(|_| Custom(Status::InternalServerError, "Failed to update game"))?;

    let game = get_game_by_user(&connection, game_id, &current_user)
        .ok_or(Custom(Status::InternalServerError, "Failed to get game"))?;
    info!("Updating game [{}] succeeded", game_id);
    Ok(Json(game))
}

/// Create a new game
/// Returns the new Game
#[post("/game", data = "<game>")]
pub fn post_game(
    game: Json<GameSubmission>,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    info!("Creating new game {:?}", game);

    let connection = diesel_client(&config);
    let current_user = logged_in_user!(connection, cookies);

    create_game(game.into_inner(), &current_user, &connection)
}

fn create_game(
    game: GameSubmission,
    current_user: &User,
    connection: &DieselConnection,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    use self::schema::games::dsl::*;

    let puzz = Puzzle::from_words(game.words, 500)
        .map_err(|_| Custom(Status::InternalServerError, "Failed to create puzzle"))?;
    let result = insert_into(games)
        .values((
            name.eq(game.name.clone()),
            owner_id.eq(current_user.id),
            words.eq(puzz.get_words()),
            puzzle.eq(puzz.to_json()),
            available_from.eq(game.available_from),
            available_to.eq(game.available_to),
        ))
        .returning(id)
        .get_result(connection);

    handle_post_game_result(result, &connection, &current_user)
}

fn handle_post_game_result(
    result: Result<i32, DieselError>,
    connection: &DieselConnection,
    current_user: &User,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    result
        .map(|id| {
            info!("Creation of new game succeeded, id: {}", id);
            let game =
                get_game_by_user(connection, id as i32, &current_user).expect("Failed to get game");
            Json(game)
        })
        .map_err(|error| {
            if let DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = &error {
                return Custom(Status::BadRequest, "Game with given name already exists");
            }
            error!(
                "Unexpected error happened while inserting new game {:#?}",
                error
            );
            Custom(
                Status::InternalServerError,
                "Unexpected error while inserting the game",
            )
        })
}

