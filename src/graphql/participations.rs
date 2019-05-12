use crate::entity::game_entities::GameEntity;
use crate::DieselConnection;
use crate::model::participation::{GameParticipation, GameParticipationEntity};
use crate::model::user::User;
use crate::schema;
use chrono::{DateTime, Utc};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
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
        .get_results::<(GameParticipationEntity, GameEntity, User)>(connection)
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
        .get_results::<(GameParticipationEntity, GameEntity, User)>(connection)
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
        .get_result::<(GameParticipationEntity, GameEntity)>(connection)
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
        .execute(connection)?;

    Ok(true)
}

pub fn end_participation(
    _connection: &DieselConnection,
    _current_user: &User,
    _game_id: i32,
) -> FieldResult<bool> {
    unimplemented!()
}

fn is_participating(connection: &DieselConnection, user: &User, game_id: i32) -> FieldResult<bool> {
    use super::super::schema::game_participations::dsl;

    let result = dsl::game_participations
        .filter(dsl::game_id.eq(game_id).and(dsl::user_id.eq(user.id)))
        .count()
        .get_result::<i64>(connection)
        .optional()?
        .map(|n| n > 0)
        .unwrap_or(false);

    Ok(result)
}
