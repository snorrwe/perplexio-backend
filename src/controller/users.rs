use super::super::service::auth;
use super::super::service::config::Config;
use super::super::service::db_client::db_client;
use super::games;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::Absolute;
use rocket::http::{Cookie, Cookies, SameSite};
use rocket::response::Redirect;
use rocket::State;
use std::borrow::Cow;
use std::str;

#[get("/login")]
pub fn index(mut cookies: Cookies, config: State<Config>) -> Redirect {
    if auth::logged_in_user_from_cookie(&mut cookies, &config).is_some() {
        get_login_redirect(&config)
    } else {
        let uri = auth::client(&config).authorize_url().to_string();
        Redirect::to(uri)
    }
}

#[get("/login?<code>")]
pub fn login(code: String, mut cookies: Cookies, config: State<Config>) -> Redirect {
    let client = auth::client(&config);
    let token = client.exchange_code(code.clone()).unwrap();
    match auth::user(&token.access_token, &config) {
        Some(u) => {
            db_client(&Config::get())
                .execute(
                    "
                    UPDATE users
                    SET auth_token=$1
                    WHERE googleid=$2
                     ",
                    &[&token.access_token, &u.googleid],
                )
                .unwrap();
            add_auth_cookies(&config, &token.access_token, &mut cookies);
            get_login_redirect(&config)
        }
        None => {
            let uri = uri!(register: token.access_token);
            Redirect::to(uri)
        }
    }
}

fn get_login_redirect(config: &Config) -> Redirect {
    if let Some(url) = &config.on_login_redirect {
        let url: Absolute = Absolute::parse(url.as_str()).unwrap();
        Redirect::to(url.into_owned())
    } else {
        let uri = uri!(games::get_games);
        Redirect::to(uri)
    }
}

#[get("/register?<token>")]
pub fn register(token: String, mut cookies: Cookies, config: State<Config>) -> Redirect {
    let user_info = auth::get_user_from_google(&token);

    db_client(&config)
        .query(
            "
        INSERT INTO users (name, auth_token, googleid)
        VALUES ($1, $2, $3)
        ",
            &[
                &user_info["displayName"].to_string(),
                &token,
                &user_info["id"].to_string(),
            ],
        )
        .expect("Failed to insert new user");
    add_auth_cookies(&config, &token, &mut cookies);
    get_login_redirect(&config)
}

fn add_auth_cookies(config: &Config, token: &String, cookies: &mut Cookies) {
    let client_domain = Cow::Owned(config.client_domain.clone()) as Cow<'static, str>;
    let auth_cookie = Cookie::build("Authorization", format!("Bearer {}", token))
        .path("/")
        .finish();
    cookies.add(auth_cookie);
}
