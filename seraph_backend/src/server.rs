use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, middleware, post, web};
use seraph_core::code_nodes::{ActiveModel as CodeNodeActiveModel, Entity as CodeNode};
use seraph_core::config;
use seraph_core::enums::{CodeLanguage, OutputType};
use seraph_core::sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use seraph_workers::broccoli_queue::queue::BroccoliQueue;
use seraph_workers::code_nodes as code_nodes_worker;
use tracing::instrument::WithSubscriber;

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

#[delete("/code-node/{id}/")]
async fn delete_code_node(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let node = match CodeNode::find_by_id(id.into_inner()).one(&*data.db).await {
        Ok(Some(node)) => node,
        Ok(None) => return HttpResponse::NotFound().body("Code node not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    match node.into_active_model().delete(&*data.db).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => {
            tracing::error!("Failed to delete code node: {}", err);
            HttpResponse::InternalServerError().body("Failed to delete code node")
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
async fn run_code_node(id: web::Path<i32>, data: web::Data<AppState>, run_input: web::Json<RunCodeNode>) -> impl Responder {
    let node = match CodeNode::find_by_id(id.into_inner()).one(&*data.db).await {
        Ok(Some(node)) => node,
        Ok(None) => return HttpResponse::NotFound().body("Code node not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let task = code_nodes_worker::CodeNodeTask::new(node.id, run_input.args.clone(), run_input.dependencies.clone());

    tracing::info!("Sending task for code node with ID: {}", node.id);

    if code_nodes_worker::publisher(&*data.queue, vec![task.clone()], None).await.is_err() {
        tracing::error!("Failed to send task to worker");
        return HttpResponse::InternalServerError().body("Failed to send task to worker");
    }

    HttpResponse::Accepted().json(serde_json::json!({
        "message": "Code node execution started",
        "task_id": &task.id,
        "task_name": &task.task_name,
        "node_id": &node.id,
    }))
}

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    db: Arc<DatabaseConnection>,
    queue: Arc<BroccoliQueue>,
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

    let _broker = BroccoliQueue::builder(&config.redis_url)
        .pool_connections(5)
        .failed_message_retry_strategy(Default::default())
        .build()
        .with_current_subscriber()
        .await
        .expect("Failed to create Broccoli queue");

    let broker = Arc::new(_broker);

    let app_state = AppState {
        db: _conn.clone(),
        queue: broker.clone(),
        config: config.clone(),
    };

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(get_code_node)
            .service(run_code_node)
            .service(create_code_node)
            .service(delete_code_node)
            .app_data(web::Data::new(app_state.clone()))
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
