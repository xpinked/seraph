#![allow(unused)]

mod config;

use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use sea_orm::{Database, DatabaseConnection};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Clone, Debug)]
struct AppState {
    db: Arc<DatabaseConnection>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let config = config::Config::from_env();

    let conn = Database::connect(&config.db_url).await.unwrap();
    if conn.ping().await.is_err() {
        eprintln!("Failed to connect to the database");
        std::process::exit(1);
    }
    tracing::info!("Connected to the database at {}", config.db_url);

    let app_state = AppState { db: Arc::new(conn) };

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
    })
    .bind((config.server_address, config.server_port))?
    .run()
    .await
}
