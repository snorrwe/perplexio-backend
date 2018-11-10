use std::env;

pub struct Config {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub postgres_url: String,
    pub oauth_redirect_url: String,
    pub allowed_origins: Vec<String>,
    pub on_login_redirect: Option<Box<String>>,
}

impl Config {
    pub fn get() -> Config {
        let on_login_redirect = env::var("ON_LOGIN_REDIRECT").unwrap_or("".to_string());
        let on_login_redirect = if on_login_redirect.is_empty() {
            None
        } else {
            Some(Box::new(on_login_redirect))
        };

        Config {
            google_client_id: env::var("GOOGLE_CLIENT_ID").unwrap(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").unwrap(),
            postgres_url: env::var("DATABASE_URL")
                .unwrap_or("postgres://postgres:almafa1@localhost:5433".to_string()),
            oauth_redirect_url: env::var("OAUTH_REDIRECT_URL")
                .unwrap_or("http://localhost:8000/login".to_string()),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .unwrap_or("http://localhost:3000".to_string())
                .split(';')
                .map(|substr| substr.to_string())
                .collect(),
            on_login_redirect: on_login_redirect,
        }
    }
}
