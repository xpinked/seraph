pub struct Config {
    pub server_address: String,
    pub server_port: u16,
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            server_address: "0.0.0.0".to_string(),
            server_port: 8080,
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_user: "user".to_string(),
            db_password: "password".to_string(),
            db_name: "seraph".to_string(),
        }
    }
}
