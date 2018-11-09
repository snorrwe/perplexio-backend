use super::config::Config;
use postgres::{Connection, TlsMode};

pub fn db_client(config: &Config) -> Connection {
    let url = &config.postgres_url;
    Connection::connect(url.clone(), TlsMode::None).expect("Failed to connect to database")
}
