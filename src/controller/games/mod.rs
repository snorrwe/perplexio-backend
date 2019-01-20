/// Returns an expression to be used in diesel `filter` queries
/// The query specifies games where the `id` matches `game_id` and `owner_id` matches the parameter
/// and `published` is false
/// # Example: 
/// ```
/// #[macro_use] extern crate perplexio;
/// extern crate diesel;
///
/// mod my_module{
///     use perplexio::schema; // This is mandatory for the macro to work as expected
///     
///     // Misc imports for the example
///     use perplexio::model::game::GameEntity;
///     use perplexio::model::user::User;
///     use perplexio::service::db_client::DieselConnection;
///     use diesel::result::Error as DieselError;
///     use diesel::ExpressionMethods;
///     use diesel::prelude::*;
///
///     fn get_unpublised_game(game_id: i32, current_user: User, connection: DieselConnection) -> Result<GameEntity, DieselError> {
///         use self::schema::games::dsl::{games};
///         games
///             .filter(unpublished_game!(game_id, current_user))
///             .get_result(&connection)
///     }
/// }
///
/// ```
#[macro_export]
macro_rules! unpublished_game (
    ($game_id: ident, $current_user: ident) => {
        {
            use self::schema::games::dsl::{id as gid, owner_id as oid, published};
            gid.eq($game_id)
                .and(oid.eq($current_user.id))
                .and(published.eq(false))
        }
    }
);

pub mod get_games;
pub mod post_games;

pub use self::get_games::*;
pub use self::post_games::*;
