use std::sync::Arc;

use crate::code_nodes::{ActiveModel as CodeNodeActiveModel, Entity as CodeNode};
use crate::config;
use crate::enums::{CodeLanguage, OutputType};
use crate::worker::CodeNodeTask;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, post, web};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use tokio::sync::mpsc;
use tokio::task;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[get("/code-node/{id}/")]
async fn get_code_node(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let post = CodeNode::find_by_id(id.into_inner()).one(&*data.db).await.unwrap();

    match post {
        Some(p) => HttpResponse::Ok().json(p),
        None => HttpResponse::NotFound().body("Post not found"),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateCodeNode {
    name: String,
    function_name: String,
    code: String,
    output_name: String,
    output_type: OutputType,
    language: CodeLanguage,
}

#[post("/code-node/")]
async fn create_code_node(data: web::Data<AppState>, node: web::Json<CreateCodeNode>) -> impl Responder {
    let node = node.into_inner();

    let post = CodeNodeActiveModel {
        name: Set(node.name),
        function_name: Set(node.function_name),
        code: Set(serde_json::to_string(&node.code).unwrap()),
        output_name: Set(node.output_name),
        output_type: Set(node.output_type),
        language: Set(node.language),
        ..Default::default()
    };

    match post.insert(&*data.db).await {
        Ok(created_node) => HttpResponse::Created().json(created_node),
        Err(err) => {
            tracing::error!("Failed to create code node: {}", err);
            HttpResponse::InternalServerError().body("Failed to create code node")
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RunCodeNode {
    #[serde(default)]
    args: Vec<String>,

    #[serde(default)]
    dependencies: Vec<String>,
}

#[post("/code-node/{id}/run")]
async fn run_code_node(
    id: web::Path<i32>,
    data: web::Data<AppState>,
    sender: web::Data<mpsc::Sender<CodeNodeTask>>,
    run_input: web::Json<RunCodeNode>,
) -> impl Responder {
    let node = match CodeNode::find_by_id(id.into_inner()).one(&*data.db).await {
        Ok(Some(node)) => node,
        Ok(None) => return HttpResponse::NotFound().body("Code node not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let task = CodeNodeTask::new(node.id, data.db.clone(), run_input.args.clone(), run_input.dependencies.clone());

    tracing::info!("Sending task for code node with ID: {}", node.id);
    if sender.send(task.clone()).await.is_err() {
        tracing::error!("Failed to send task to worker");
        return HttpResponse::InternalServerError().body("Failed to send task to worker");
    }

    HttpResponse::Accepted().json(serde_json::json!({
        "message": "Code node execution started",
        "task_id": task.id,
        "node_id": node.id,
    }))
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct AppState {
    db: Arc<DatabaseConnection>,
    config: config::Config,
}

#[actix_web::main]
pub async fn server() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).with_test_writer().init();

    let config = config::Config::from_env();

    let conn = Database::connect(&config.db_url).await.unwrap();
    if conn.ping().await.is_err() {
        eprintln!("Failed to connect to the database");
        std::process::exit(1);
    }
    tracing::info!("Connected to the database at {}", config.db_url);

    let _conn = Arc::new(conn);
    let app_state = AppState {
        db: _conn.clone(),
        config: config.clone(),
    };

    let (sender, receiver) = mpsc::channel(32);
    tracing::info!("Starting worker thread");
    task::spawn(crate::worker::worker(receiver));

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(get_code_node)
            .service(run_code_node)
            .service(create_code_node)
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(sender.clone()))
            .wrap(middleware::Logger::default())
            .wrap(actix_cors::Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .default_service(web::route().to(|| async { HttpResponse::NotFound().body("Not Found") }))
    })
    .bind((config.server_address, config.server_port))?
    .run()
    .await
}

pub fn main() {
    let result = server();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
