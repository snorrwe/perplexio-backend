use std::env;

pub struct Config {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub postgres_url: String,
    pub oauth_redirect_url: String,
}

impl Config {
    pub fn get() -> Config {
        Config {
            google_client_id: env::var("GOOGLE_CLIENT_ID").unwrap(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").unwrap(),
            postgres_url: env::var("DATABASE_URL")
                .unwrap_or("postgres://postgres:almafa1@localhost:5433".to_string()),
            oauth_redirect_url: env::var("OAUTH_REDIRECT_URL")
                .unwrap_or("http://localhost:8000/login".to_string()),
        }
    }
}
