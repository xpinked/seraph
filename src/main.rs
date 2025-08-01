#![allow(unused)]

mod config;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    tracing_subscriber::fmt::init();

    let config = config::Config::new();

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .wrap(middleware::Logger::default())
    })
    .bind((config.server_address, config.server_port))?
    .run()
    .await
}
