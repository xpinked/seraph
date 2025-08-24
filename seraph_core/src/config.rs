use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    // Server configuration
    pub server_address: String,
    pub server_port: u16,
    // Database configuration
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_url: String,
    // Redis configuration
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let server_address = env::var("SERVER_ADDRESS").unwrap();
        let server_port: u16 = env::var("SERVER_PORT").unwrap().parse().expect("Invalid server port");
        let db_host = env::var("DATABASE_HOST").unwrap();
        let db_port: u16 = env::var("DATABASE_PORT").unwrap().parse().expect("Invalid database port");
        let db_user = env::var("POSTGRES_USER").unwrap();
        let db_password = env::var("POSTGRES_PASSWORD").unwrap();
        let db_name = env::var("DATABASE_NAME").unwrap();

        let db_url = format!("postgres://{}:{}@{}:{}/{}", db_user, db_password, db_host, db_port, db_name);

        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let redis_port: u16 = env::var("REDIS_PORT")
            .unwrap_or_else(|_| "6379".to_string())
            .parse()
            .expect("Invalid redis port");
        let redis_url = format!("redis://{}:{}", redis_host, redis_port);

        Config {
            server_address,
            server_port,
            db_host,
            db_port,
            db_user,
            db_password,
            db_name,
            db_url,
            redis_host,
            redis_port,
            redis_url,
        }
    }
}
