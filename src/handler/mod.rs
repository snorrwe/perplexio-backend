pub mod users;

use crate::actix_web::HttpMessage;
use crate::graphql::{Context, Schema};
use crate::service::auth::logged_in_user_from_cookie;
use crate::service::config::Config;
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
    data: web::Json<GraphQLRequest>,
    request: web::HttpRequest,
    config: web::Data<Arc<Config>>,
    schema: web::Data<Arc<Schema>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // TODO: use pool
    let connection = crate::service::db_client::diesel_client(&*config.clone())
        .expect("Failed to connect to database");
    let user = request
        .cookies()
        .ok()
        .and_then(|cookies| logged_in_user_from_cookie(&connection, &cookies));
    web::block(move || {
        let context = Context {
            connection: connection,
            user: user,
        };
        let res = data.execute(&schema, &context);
        serde_json::to_string(&res)
    })
    .map_err(Error::from)
    .and_then(|response| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(response))
    })
}

