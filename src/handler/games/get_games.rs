use super::super::super::fairing::DieselConnection;
use super::super::super::model::game::{GameDTO, GameEntity, GameId};
use super::super::super::model::paginated::Paginated;
use super::super::super::model::participation::GameParticipation;
use super::super::super::model::user::User;
use super::super::super::schema;
use super::super::super::service::auth::logged_in_user_from_cookie;
use super::super::super::service::pagination::*;
use super::super::participations::{get_participation_inner, insert_participation};
use super::super::solutions::get_users_solutions;
use chrono::Utc;
use diesel::prelude::*;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket_contrib::json::Json;
use serde_json::to_value;

/// Get a list of the available `GameId`s
/// If the user is logged in, then their unavailable games are listed as well
#[get("/games?<page>")]
pub fn get_games(
    page: Option<u32>,
    mut cookies: Cookies,
    client: DieselConnection,
) -> Result<Json<Paginated<GameId>>, Custom<&'static str>> {
    use self::schema::games::dsl::{
        available_from, available_to, games, id, name as gname, owner_id, published,
    };
    use self::schema::users::dsl::{name as uname, users};

    let current_user = logged_in_user_from_cookie(&client, &mut cookies);

    let page = page.unwrap_or(0) as i64;
    let query = games
        .inner_join(users)
        .select((id, gname, uname, available_from, available_to, published))
        .order_by(available_from.desc());
    let items = if let Some(current_user) = &current_user {
        query
            .filter(
                owner_id.eq(current_user.id).or(published
                    .eq(true)
                    .and(available_from.le(Utc::now()))
                    .and(available_to.gt(Utc::now()).or(available_to.is_null()))),
            )
            .paginate(page)
            .per_page(25)
            .load_and_count_pages(&client)
    } else {
        query
            .filter(
                available_from
                    .le(Utc::now())
                    .and(available_to.gt(Utc::now()).or(available_to.is_null()))
                    .and(published.eq(true)),
            )
            .paginate(page)
            .per_page(25)
            .load_and_count_pages(&client)
    };
    let (items, total_pages) = items.map_err(|err| {
        error!("Failed to read games {:?}", err);
        Custom(Status::InternalServerError, "Failed to read games")
    })?;
    let result = items.iter().cloned().collect();
    let result = Paginated::<GameId>::new(result, total_pages, page);
    Ok(Json(result))
}

/// Get a specific game by ID
/// Requires a logged in user
/// Starts game participation if it's the first time of the user visiting this game
#[get("/game/<id>")]
pub fn get_game(
    id: i32,
    mut cookies: Cookies,
    connection: DieselConnection,
) -> Result<Json<GameDTO>, Custom<&'static str>> {
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
    use self::schema::games::dsl::{games, id as gid};
    use self::schema::users::dsl::{id as uid, name as uname, users};
    games
        .filter(gid.eq(game_id))
        .get_result::<GameEntity>(&connection.0)
        .ok()
        .map(|mut game| {
            let is_owner = game.owner_id == current_user.id;
            if !is_owner {
                if !game.published
                    || game.available_from.is_none()
                    || game.available_from.unwrap() > Utc::now()
                {
                    return None;
                } else {
                    insert_solutions_and_participation(
                        &connection,
                        game_id,
                        current_user,
                        &mut game,
                    );
                }
            }
            let owner = users
                .filter(uid.eq(game.owner_id))
                .select(uname)
                .get_result(&connection.0)
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
    if get_participation_inner(&current_user, game_id, &client).is_none() {
        insert_participation(
            GameParticipation {
                user_id: current_user.id,
                game_id: game_id,
                start_time: Utc::now(),
                end_time: None,
            },
            &client,
        );
    }
    let solutions = get_users_solutions(&client, &current_user, game_id);
    game.puzzle["solutions"] = to_value(solutions).expect("Failed to serialize solutions");
}
