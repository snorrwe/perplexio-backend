use super::super::super::model::game::{GameEntity, GameDTO, GameId, GameIdQuery};
use super::super::super::model::participation::GameParticipation;
use super::super::super::model::user::User;
use super::super::super::schema;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::config;
use super::super::super::service::db_client::{diesel_client, DieselConnection};
use super::super::participations::{get_participation, insert_participation};
use super::super::solutions::get_users_solutions;
use chrono::Utc;
use diesel::prelude::*;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::to_value;

#[get("/games")]
pub fn get_games(mut cookies: Cookies, config: State<config::Config>) -> Json<Vec<GameId>> {
    use self::schema::games::dsl::*;
    use self::schema::games::dsl::{id, name as gname};
    use self::schema::users::dsl::name as uname;
    use self::schema::users::dsl::*;

    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    let client = diesel_client(&config);
    let query = games
        .inner_join(users)
        .select((id, gname, uname, available_from))
        .limit(20)
        .order_by(available_from.desc());
    let query = if let Some(current_user) = &current_user {
        query
            .filter(
                available_from
                    .le(Utc::now())
                    .and(available_to.gt(Utc::now()))
                    .or(owner_id.eq(current_user.id)),
            )
            .get_results::<GameIdQuery>(&client)
    } else {
        query
            .filter(
                available_from
                    .le(Utc::now())
                    .and(available_to.gt(Utc::now())),
            )
            .get_results::<GameIdQuery>(&client)
    };
    let items = query.unwrap();
    let result = items
        .iter()
        .map(|game_id| GameId {
            id: game_id.id,
            name: game_id.name.clone(),
            owner: game_id.owner.clone(),
            available_from: game_id.available_from.map_or(None, |d| Some(d.to_string())),
        })
        .collect();
    Json(result)
}

#[get("/game/<id>")]
pub fn get_game(
    id: i32,
    mut cookies: Cookies,
    config: State<config::Config>,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
    let current_user = logged_in_user_from_cookie(&mut cookies, &config);
    if current_user.is_none() {
        return Err(Custom(Status::Unauthorized, "Log in first"));
    }
    let current_user = current_user.unwrap();
    let game = get_game_by_user(id, &current_user, &config);
    if let Some(game) = game {
        Ok(Json(game))
    } else {
        return Err(Custom(Status::NotFound, "Game not found"));
    }
}

pub fn get_game_by_user(
    game_id: i32,
    current_user: &User,
    config: &config::Config,
) -> Option<GameDTO> {
    use self::schema::games::dsl::id as gid;
    use self::schema::games::dsl::*;
    use self::schema::users::dsl::*;
    use self::schema::users::dsl::{id as uid, name as uname};

    let connection = diesel_client(&config);
    games
        .filter(gid.eq(game_id))
        .get_result::<GameEntity>(&connection)
        .ok()
        .map(|mut game| {
            let is_owner = game.owner_id == current_user.id;
            if !is_owner
                && (game.available_from.is_none() || game.available_from.unwrap() < Utc::now())
            {
                return None;
            } else if !is_owner {
                insert_solutions_and_participation(&connection, game_id, current_user, &mut game);
            }
            let owner = users
                .filter(uid.eq(game.owner_id))
                .select(uname)
                .get_result(&connection)
                .expect("Owning user was not found");
            let game = game.into_dto(owner, is_owner);
            Some(game)
        })
        .map_or(None, |x| x)
}

fn insert_solutions_and_participation(
    client: &DieselConnection,
    game_id: i32,
    current_user: &User,
    game: &mut GameEntity,
) {
    if get_participation(&current_user, game_id, &client).is_none() {
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

