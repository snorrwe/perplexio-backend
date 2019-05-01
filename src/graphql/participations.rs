use crate::entity::game_entities::GameEntity;
use crate::fairing::DieselConnection;
use crate::model::participation::{GameParticipation, GameParticipationEntity};
use crate::model::user::User;
use crate::schema;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{insert_into, update};
use juniper::FieldResult;

#[derive(GraphQLObject)]
pub struct GameParticipationDTO {
    pub game_id: i32,
    pub game_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub user_name: String,
}

/// Get the participations for the given game
/// Requires user to be the owner
pub fn get_all_participations(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<Vec<GameParticipationDTO>> {
    use self::schema::game_participations::dsl::{
        duration, game_id as gp_gid, game_participations,
    };
    use self::schema::games::dsl::{games, owner_id};
    use self::schema::users::dsl::users;

    let result = game_participations
        .filter(
            gp_gid
                .eq(game_id)
                .and(owner_id.eq(current_user.id))
                .and(duration.is_not_null()),
        )
        .inner_join(games)
        .inner_join(users)
        .order_by(duration.desc())
        .get_results::<(GameParticipationEntity, GameEntity, User)>(&connection.0)
        .map_err(|_| "Game was not found")?;

    let result = result
        .into_iter()
        .map(|(parti, game, user)| GameParticipationDTO {
            game_id: parti.game_id,
            game_name: game.name,
            start_time: parti.start_time,
            end_time: parti.end_time,
            user_name: user.name,
        })
        .collect();

    Ok(result)
}

/// Get the participations of the current user
pub fn get_participations(
    connection: &DieselConnection,
    current_user: &User,
) -> FieldResult<Vec<GameParticipationDTO>> {
    use self::schema::game_participations::dsl::{game_participations, start_time, user_id};
    use self::schema::games::dsl::games;
    use self::schema::users::dsl::users;

    let result = game_participations
        .filter(user_id.eq(current_user.id))
        .inner_join(games)
        .inner_join(users)
        .limit(100)
        .order_by(start_time.desc())
        .get_results::<(GameParticipationEntity, GameEntity, User)>(&connection.0)
        .map_err(|_| "Failed to read games")?;

    let result = result
        .into_iter()
        .map(|(parti, game, user)| GameParticipationDTO {
            game_id: parti.game_id,
            game_name: game.name,
            start_time: parti.start_time,
            end_time: parti.end_time,
            user_name: user.name,
        })
        .collect();

    Ok(result)
}

/// Get the participation belonging to the game
pub fn get_participation(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<GameParticipationDTO> {
    use self::schema::game_participations::dsl::{game_participations, user_id};
    use self::schema::games::dsl::{games, id as gid};

    let result = game_participations
        .filter(user_id.eq(current_user.id).and(gid.eq(game_id)))
        .inner_join(games)
        .get_result::<(GameParticipationEntity, GameEntity)>(&connection.0)
        .map(|(parti, game)| GameParticipationDTO {
            game_id: parti.game_id,
            game_name: game.name,
            start_time: parti.start_time,
            end_time: parti.end_time,
            user_name: current_user.name.clone(),
        })?;

    Ok(result)
}

pub fn add_participation(
    connection: &DieselConnection,
    current_user: &User,
    game_id: i32,
) -> FieldResult<bool> {
    use super::super::schema::game_participations::dsl;

    if is_participating(connection, current_user, game_id)? {
        Err("User is already participating in the game")?;
    }

    let participation = GameParticipation {
        game_id: game_id,
        user_id: current_user.id,
        start_time: Utc::now(),
        end_time: None,
    };

    insert_into(dsl::game_participations)
        .values(&participation)
        .execute(&connection.0)?;

    Ok(true)
}

pub fn end_participation(
    client: &DieselConnection,
    user: &User,
    game_id: i32,
) -> FieldResult<bool> {
    use super::super::schema::game_participations::dsl::{
        duration, end_time as et, game_id as gid, game_participations as gp, start_time as st,
        user_id,
    };

    let end_time = Utc::now();
    let start_time = gp
        .filter(user_id.eq(user.id).and(gid.eq(game_id)))
        .select((st,))
        .get_result::<(DateTime<Utc>,)>(&client.0)
        .optional()?
        .ok_or("User is not participating")?;

    let dur = (end_time - start_time.0).num_milliseconds();
    update(gp.filter(user_id.eq(user.id).and(gid.eq(game_id))))
        .set((et.eq(end_time), duration.eq(dur as i32)))
        .execute(&client.0)?;

    Ok(true)
}

fn is_participating(connection: &DieselConnection, user: &User, game_id: i32) -> FieldResult<bool> {
    use super::super::schema::game_participations::dsl;

    let result = dsl::game_participations
        .filter(dsl::game_id.eq(game_id).and(dsl::user_id.eq(user.id)))
        .count()
        .get_result::<i64>(&connection.0)
        .optional()?
        .map(|n| n > 0)
        .unwrap_or(false);

    Ok(result)
}

