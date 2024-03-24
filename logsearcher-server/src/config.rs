pub struct Config {
    pub pg_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            pg_url: dotenv::var("DATABASE_URL")
                .unwrap_or("postgresql://postgres:test@localhost/postgres".to_owned()),
        }
    }
}
