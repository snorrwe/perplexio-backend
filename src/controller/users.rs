use super::super::service::auth;
use super::super::service::config;
use super::super::service::db_client::db_client;
use super::games;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use rocket::State;
use std::str;

#[get("/login?<code>")]
pub fn login(code: String, mut cookies: Cookies, config: State<config::Config>) -> Redirect {
    let client = auth::client(&config);
    let token = client.exchange_code(code.clone()).unwrap();
    match auth::user(&token.access_token, &config) {
        Some(u) => {
            db_client(&config::Config::get())
                .execute(
                    "UPDATE users
                     SET auth_token=$1
                     WHERE googleid=$2",
                    &[&token.access_token, &u.googleid],
                )
                .unwrap();
            add_auth_cookies(&token.access_token, &mut cookies);
            let uri = uri!(games::get_games);
            Redirect::to(uri)
        }
        None => {
            let uri = uri!(register: token.access_token);
            Redirect::to(uri)
        }
    }
}

#[get("/register?<token>")]
pub fn register(token: String, mut cookies: Cookies, config: State<config::Config>) -> Redirect {
    let user_info = auth::get_user_from_google(&token);

    let db_result = db_client(&config).query(
        "INSERT INTO users (name, auth_token, googleid)
         VALUES ($1, $2, $3)",
        &[
            &user_info["displayName"].to_string(),
            &token,
            &user_info["id"].to_string(),
        ],
    );
    match db_result {
        Ok(_) => {
            add_auth_cookies(&token, &mut cookies);
            let uri = uri!(games::get_games);
            Redirect::to(uri)
        }
        Err(e) => panic!(e),
    }
}

fn add_auth_cookies(token: &String, cookies: &mut Cookies) {
    let auth_cookie = Cookie::new("Authorization", format!("Bearer {}", token));
    cookies.add(auth_cookie);
}
