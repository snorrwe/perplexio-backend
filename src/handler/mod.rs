pub mod users;

use crate::graphql::{Context, Schema};
use crate::guard::LoggedInUser;
use crate::DieselConnection;
use crate::actix_web::FromRequest;
use crate::actix_web::HttpMessage;
use crate::service::auth::logged_in_user_from_cookie;
use actix_web::{App, Error, HttpRequest, HttpResponse, Json};
use futures::future::{self, Future};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::io;
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
    schema: Arc<Schema>,
    connection: DieselConnection,
    request: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    future::lazy(move || {
        let user = request
            .cookies()
            .ok()
            .and_then(|cookies| logged_in_user_from_cookie(&connection, &cookies));
        let context = Context {
            connection: connection,
            user: user,
        };
        let res = data.execute(&schema, &context);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|response| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(response))
    })
}

