use crate::service::auth;
use crate::service::config::Config;
use crate::ConnectionPool;
use crate::DieselConnection;
use actix_identity::Identity;
use actix_web::{dev::HttpResponseBuilder, http, web, Error, HttpResponse};
use diesel::prelude::*;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginParams {
    code: Option<String>,
}

pub fn login(
    id: Identity,
    params: web::Query<LoginParams>,
    pool: web::Data<ConnectionPool>,
    config: web::Data<Arc<Config>>,
) -> Result<HttpResponse, Error> {
    let token = id.identity();
    let token = token.as_ref();
    let code = params.code.as_ref();
    let connection: &DieselConnection = &pool.get().unwrap();
    if code.is_none() {
        let result = HttpResponse::Found()
            .login_redirect(token.map(|x| x.as_ref()).unwrap_or(""), &config, connection)
            .finish();
        return Ok(result);
    }
    let client = auth::client(&config);
    let token = client.exchange_code(code.unwrap().clone()).unwrap();
    match auth::user(&token.access_token, &config) {
        Some(user) => {
            use crate::schema::users as u;
            use diesel::update;

            update(u::table)
                .filter(u::dsl::googleid.eq(user.googleid))
                .set(u::dsl::auth_token.eq(&token.access_token))
                .execute(connection)
                .map(|_| ())
                .unwrap_or_else(|e| {
                    error!("Failed to update login info {:?}", e);
                });
        }
        None => {
            register(token.access_token.as_str(), connection).unwrap_or_else(|e| {
                error!("Failed to update login info {:?}", e);
            });
        }
    }
    let result = HttpResponse::Found()
        .login_redirect(token.access_token.as_str(), &config, connection)
        .finish();

    id.remember(token.access_token);

    Ok(result)
}

pub fn logout(
    id: Identity,
    pool: web::Data<ConnectionPool>,
    config: web::Data<Arc<Config>>,
) -> Result<HttpResponse, Error> {
    use crate::schema::users as u;
    use diesel::update;

    let connection: &DieselConnection = &pool.get().unwrap();

    if let Some(token) = id.identity() {
        update(u::table)
            .filter(u::dsl::auth_token.eq(token))
            .set(u::dsl::auth_token.eq(None::<&str>))
            .execute(connection)
            .map(|_| ())
            .unwrap_or(());

        id.forget();
    }
    let result = HttpResponse::Found()
        .header(
            http::header::LOCATION,
            config.on_login_redirect.clone().unwrap_or("/".to_string()),
        )
        .finish();
    Ok(result)
}

pub fn register(token: &str, connection: &DieselConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users as u;
    use diesel::insert_into;

    let user_info = auth::get_user_from_google(&token);

    let name = user_info["displayName"].to_string();

    insert_into(u::table)
        .values((
            u::dsl::name.eq(&name[1..name.len() - 1]), // Cut the apostrophes returned by Google
            u::dsl::auth_token.eq(&token),
            u::dsl::googleid.eq(user_info["id"].to_string()),
        ))
        .execute(connection)?;

    Ok(())
}

pub trait LoginRedirect {
    fn login_redirect(
        &mut self,
        token: &str,
        config: &Config,
        connection: &DieselConnection,
    ) -> &mut Self;
}

impl LoginRedirect for HttpResponseBuilder {
    fn login_redirect(
        &mut self,
        token: &str,
        config: &Config,
        connection: &DieselConnection,
    ) -> &mut Self {
        if auth::logged_in_user(&connection, token).is_some() {
            let url = config
                .on_login_redirect
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("/");
            self.header(http::header::LOCATION, url)
        } else {
            let uri = auth::client(&config).authorize_url().to_string();
            self.header(http::header::LOCATION, uri)
        }
    }
}

