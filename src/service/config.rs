use std::env;

pub struct Config {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub postgres_url: String,
    pub oauth_redirect_url: String,
    pub allowed_origins: Vec<String>,
    pub on_login_redirect: Option<String>,
    pub client_domain: String,
}

impl Config {
    pub fn get() -> Config {
        let on_login_redirect =
            env::var("ON_LOGIN_REDIRECT").unwrap_or("http://localhost:3000".to_string());
        let on_login_redirect = if on_login_redirect.is_empty() {
            None
        } else {
            Some(on_login_redirect)
        };

        Config {
            google_client_id: env::var("GOOGLE_CLIENT_ID")
                .expect("Google client id must be present!"),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .expect("Google client secret must be present!"),
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
            client_domain: env::var("DOMAIN").unwrap_or("localhost:3000".to_string()),
        }
    }
}
