pub mod users;

use crate::graphql::{Context, Schema};
use crate::service::auth;
use crate::ConnectionPool;
use crate::DieselConnection;
use actix_web::middleware::identity::Identity;
use actix_web::{web, Error, HttpResponse};
use futures::future::{self, Future};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::sync::Arc;

pub fn graphiql() -> impl Future<Item = HttpResponse, Error = Error> {
    let html = graphiql_source("/graphql");
    future::ok(()).and_then(move |_| {
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html))
    })
}

pub fn graphql_handler(
    id: Identity,
    data: web::Json<GraphQLRequest>,
    schema: web::Data<Arc<Schema>>,
    pool: web::Data<ConnectionPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let connection: &DieselConnection = &pool.get().unwrap();
    let user = id
        .identity()
        .and_then(|token| auth::logged_in_user(connection, token.as_str()));
    web::block(move || {
        let connection: &DieselConnection = &pool.get().unwrap();
        let context = Context {
            connection: connection as *const DieselConnection,
            user: user,
        };
        let res = data.execute(&schema, &context);
        serde_json::to_string(&res)
    })
    .map_err(Error::from)
    .and_then(|response| {
        let response = HttpResponse::Ok()
            .content_type("application/json")
            .body(response);
        Ok(response)
    })
}

