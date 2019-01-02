use super::super::super::model::game::{GameDTO, GameEntity, GameId};
use super::super::super::model::participation::GameParticipation;
use super::super::super::model::user::User;
use super::super::super::schema;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::config;
use super::super::super::service::db_client::{diesel_client, DieselConnection};
use super::super::participations::{get_participation_inner, insert_participation};
use super::super::solutions::get_users_solutions;
use chrono::Utc;
use diesel::prelude::*;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::to_value;

/// Get a list of the available `GameId`s
/// If the user is logged in, then their unavailable games are listed as well
#[get("/games")]
pub fn get_games(mut cookies: Cookies, config: State<config::Config>) -> Json<Vec<GameId>> {
    use self::schema::games::dsl::{
        available_from, available_to, games, id, name as gname, owner_id,
    };
    use self::schema::users::dsl::{name as uname, users};

    let client = diesel_client(&config);
    let current_user = logged_in_user_from_cookie(&client, &mut cookies);
    let query = games
        .inner_join(users)
        .select((id, gname, uname, available_from, available_to))
        .limit(100)
        .order_by(available_from.desc());
    let items = if let Some(current_user) = &current_user {
        query
            .filter(
                available_from
                    .le(Utc::now())
                    .and(available_to.gt(Utc::now()).or(available_to.is_null()))
                    .or(owner_id.eq(current_user.id)),
            ).get_results::<GameId>(&client)
    } else {
        query
            .filter(
                available_from
                    .le(Utc::now())
                    .and(available_to.gt(Utc::now()).or(available_to.is_null())),
            ).get_results::<GameId>(&client)
    };
    let result = items
        .unwrap()
        .iter()
        .map(|game_id| GameId {
            id: game_id.id,
            name: game_id.name.clone(),
            owner: game_id.owner.clone(),
            available_from: game_id.available_from,
            available_to: game_id.available_to,
        }).collect();
    Json(result)
}

/// Get a specific game by ID
/// Requires a logged in user
/// Starts game participation if it's the first time of the user visiting this game
#[get("/game/<id>")]
pub fn get_game(
    id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    let connection = diesel_client(&config);
    let current_user = logged_in_user!(connection, cookies);
    let game = get_game_by_user(&connection, id, &current_user);
    if let Some(game) = game {
        Ok(Json(game))
    } else {
        return Err(Custom(Status::NotFound, "Game not found"));
    }
}

/// Get a game by ID and user
pub fn get_game_by_user(
    connection: &DieselConnection,
    game_id: i32,
    current_user: &User,
) -> Option<GameDTO> {
    use self::schema::games::dsl::id as gid;
    use self::schema::games::dsl::*;
    use self::schema::users::dsl::*;
    use self::schema::users::dsl::{id as uid, name as uname};
    games
        .filter(gid.eq(game_id))
        .get_result::<GameEntity>(connection)
        .ok()
        .map(|mut game| {
            let is_owner = game.owner_id == current_user.id;
            if !is_owner
                && (game.available_from.is_none() || game.available_from.unwrap() > Utc::now())
            {
                return None;
            } else if !is_owner {
                insert_solutions_and_participation(&connection, game_id, current_user, &mut game);
            }
            let owner = users
                .filter(uid.eq(game.owner_id))
                .select(uname)
                .get_result(connection)
                .expect("Owning user was not found");
            let game = game.into_dto(owner, is_owner);
            Some(game)
        }).map_or(None, |x| x)
}

fn insert_solutions_and_participation(
    client: &DieselConnection,
    game_id: i32,
    current_user: &User,
    game: &mut GameEntity,
) {
    if get_participation_inner(&current_user, game_id, &client).is_none() {
        insert_participation(
            GameParticipation {
                user_id: current_user.id,
                game_id: game_id,
                start_time: Some(Utc::now()),
                end_time: None,
            },
            &client,
        );
    }
    let solutions = get_users_solutions(&client, &current_user, game_id);
    game.puzzle["solutions"] = to_value(solutions).expect("Failed to serialize solutions");
}

fn is_owner(current_user: &User, owner_id: i32) -> bool {
    current_user.id == owner_id
}
