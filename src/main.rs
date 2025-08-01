#![allow(unused)]

mod config;
mod post;

use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use sea_orm::{Database, DatabaseConnection, EntityTrait};

#[get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[get("/posts/{id}/")]
async fn get_post(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let post = post::Entity::find_by_id(1).one(&*data.db).await.unwrap();

    match post {
        Some(p) => HttpResponse::Ok().json(p),
        None => HttpResponse::NotFound().body("Post not found"),
    }
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

    let _conn = Arc::new(conn);
    let app_state = AppState { db: _conn.clone() };

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(get_post)
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .default_service(
                web::route().to(|| async { HttpResponse::NotFound().body("Not Found") }),
            )
    })
    .bind((config.server_address, config.server_port))?
    .run()
    .await
}
