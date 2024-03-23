pub struct Config {
    pub pg_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            pg_url: "postgresql://postgres:test@localhost/postgres".to_owned(),
        }
    }
}
