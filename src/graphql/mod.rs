pub mod games;
pub mod mutation;
pub mod participations;
pub mod puzzles;
pub mod query;
pub mod solutions;

pub use self::mutation::Mutation;
pub use self::query::Query;
use super::fairing::DieselConnection;
use super::model::user::User;
use juniper::RootNode;

pub struct Context {
    pub connection: DieselConnection,
    pub user: Option<User>,
}

impl juniper::Context for Context {}

pub type Schema = RootNode<'static, Query, Mutation>;
