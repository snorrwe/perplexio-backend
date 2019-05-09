use super::config::Config;
use postgres::{self, TlsMode};

pub type Connection = postgres::Connection;

pub fn db_client(config: &Config) -> Connection {
    let url = &config.postgres_url;
    Connection::connect(url.clone(), TlsMode::None).expect("Failed to connect to database")
}
