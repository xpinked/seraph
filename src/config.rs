use std::env;

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

    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Config {
            server_address: env::var("SERVER_ADDRESS").unwrap(),
            server_port: env::var("SERVER_PORT")
                .unwrap()
                .parse()
                .expect("Invalid server port"),
            db_host: env::var("DATABASE_HOST").unwrap(),
            db_port: env::var("DATABASE_PORT")
                .unwrap()
                .parse()
                .expect("Invalid database port"),
            db_user: env::var("POSTGRES_USER").unwrap(),
            db_password: env::var("POSTGRES_PASSWORD").unwrap(),
            db_name: env::var("DATABASE_NAME").unwrap(),
        }
    }
}
