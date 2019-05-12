use std::env;

pub struct Config {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub postgres_url: String,
    pub allowed_origins: Vec<String>,
    pub oauth_redirect_url: String,
    pub on_login_redirect: Option<String>,
    pub client_domain: String,
    pub heroku: bool,
    pub address: String,
    pub port: u16,
    pub secure: bool,
    pub auth_private_key: Option<String>,
}

impl Config {
    pub fn get() -> Config {
        let on_login_redirect = env::var("ON_LOGIN_REDIRECT").unwrap_or("/".to_string());
        let on_login_redirect = if on_login_redirect.is_empty() {
            None
        } else {
            Some(on_login_redirect)
        };
        let heroku = env::var("IS_HEROKU").unwrap_or("0".to_string());
        let heroku = heroku.parse::<i32>().expect("IS_HEROKU must be a number");
        let heroku = heroku != 0;

        let secure = env::var("PERPLEXIO_SECURE").unwrap_or("1".to_string());
        let secure = secure
            .parse::<i32>()
            .expect("PERPLEXIO_SECURE must be a number");
        let secure = secure != 0;

        Config {
            auth_private_key: env::var("AUTH_PRIVATE_KEY").ok(),
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
            address: env::var("ADDRESS").unwrap_or("0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or("8000".to_string())
                .parse::<u16>()
                .expect("Port must be a 16 bit long number"),
            secure,
            heroku,
        }
    }
}

