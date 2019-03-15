pub mod users;

use super::graphql::{Context, Schema};
use super::guard::LoggedInUser;
use super::DieselConnection;
use rocket::response::content;
use rocket::State;

#[get("/")]
pub fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[post("/graphql", data = "<request>")]
pub fn graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
    connection: DieselConnection,
    user: Option<LoggedInUser>,
) -> juniper_rocket::GraphQLResponse {
    let context = Context {
        connection: connection,
        user: user.map(|u| u.0),
    };
    request.execute(&schema, &context)
}

