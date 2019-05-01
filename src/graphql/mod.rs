pub mod mutation;
pub mod games;
pub mod query;
pub mod puzzles;
pub mod participations;

use super::model::user::User;
use super::fairing::DieselConnection;
use juniper::RootNode;
pub use self::mutation::Mutation;
pub use self::query::Query;

pub struct Context {
    pub connection: DieselConnection,
    pub user: Option<User>,
}

impl juniper::Context for Context {}

pub type Schema = RootNode<'static, Query, Mutation>;
