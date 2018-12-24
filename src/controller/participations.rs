use super::super::model::participation::{
    GameParticipation, GameParticipationDTO, GameParticipationEntity,
};
use super::super::model::user::User;
use super::super::schema;
use super::super::service::auth::logged_in_user_from_cookie;
use super::super::service::config::Config;
use super::super::service::db_client::{diesel_client, DieselConnection};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use rocket::http::{Cookies, Status};
use rocket::response::status::Custom;
use rocket::State;
use rocket_contrib::json::Json;

#[get("/participations")]
pub fn get_participations(
    mut cookies: Cookies,
    config: State<Config>,
) -> Result<Json<Vec<GameParticipationDTO>>, Custom<&'static str>> {
    use self::schema::game_participations::dsl::{
        end_time, game_participations, start_time, user_id,
    };
    use self::schema::games::dsl::{games, id as game_id, name as gname};

    let current_user = logged_in_user!(cookies, config);
    let connection = diesel_client(&config);

    let result = game_participations
        .filter(user_id.eq(current_user.id))
        .inner_join(games)
        .select((game_id, gname, start_time, end_time))
        .limit(100)
        .order_by(start_time.desc())
        .get_results::<GameParticipationDTO>(&connection)
        .expect("Failed to read games");

    Ok(Json(result))
}

#[get("/participation/<game_id>")]
pub fn get_participation(
    game_id: i32,
    mut cookies: Cookies,
    config: State<Config>,
) -> Result<Json<GameParticipationDTO>, Custom<&'static str>> {
    use self::schema::games::dsl::{games, id as gid, name as game_name};
    let current_user = logged_in_user!(cookies, config);
    let connection = diesel_client(&config);
    let result = get_participation_inner(&current_user, game_id, &connection);
    result.map_or(
        Err(Custom(Status::NotFound, "Participation was not found")),
        |participation| {
            let name = games
                .filter(gid.eq(game_id))
                .select(game_name)
                .get_result(&connection);
            if name.is_err() {
                error!("Unexpected error while getting game name {:?}", name);
                return Err(Custom(Status::NotFound, "Game was not found"));
            }
            let participation = participation.into_dto(name.unwrap());
            Ok(Json(participation))
        },
    )
}

pub fn get_participation_inner(
    user: &User,
    game_id: i32,
    client: &DieselConnection,
) -> Option<GameParticipationEntity> {
    use self::schema::game_participations::dsl::{game_id as gid, game_participations, user_id};

    game_participations
        .filter(user_id.eq(user.id).and(gid.eq(game_id)))
        .get_result(client)
        .ok()
}

pub fn insert_participation(participation: GameParticipation, client: &DieselConnection) {
    use super::super::schema::game_participations::dsl::*;

    insert_into(game_participations)
        .values(&participation)
        .execute(client)
        .expect("Failed to insert participation");
}

