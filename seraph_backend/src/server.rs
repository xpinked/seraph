use std::sync::Arc;

use crate::code_nodes::{ActiveModel as CodeNodeActiveModel, Entity as CodeNode};
use crate::config;
use crate::enums::{CodeLanguage, OutputType};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, post, web};
use bollard::query_parameters::RemoveContainerOptions;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};

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

#[get("/code-node/{id}/run")]
async fn run_code_node(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    use bollard::Docker;
    use bollard::body_try_stream;
    use bollard::models::ContainerCreateBody;
    use bollard::query_parameters::CreateContainerOptions;
    use bollard::query_parameters::LogsOptions;
    use bollard::query_parameters::StartContainerOptions;
    use bollard::query_parameters::UploadToContainerOptions;
    use bollard::query_parameters::WaitContainerOptions;
    use futures_util::{StreamExt, TryFutureExt};
    use tokio::fs::File;
    use tokio_util::io::ReaderStream;

    let node = match CodeNode::find_by_id(id.into_inner()).one(&*data.db).await {
        Ok(Some(node)) => node,
        Ok(None) => return HttpResponse::NotFound().body("Code node not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let docker = Docker::connect_with_defaults().unwrap();

    let container = ContainerCreateBody {
        working_dir: Some("/tmp".to_string()),
        image: Some(node.language.get_image_name().to_string()),
        cmd: Some(node.get_command()),
        ..Default::default()
    };

    let container = docker.create_container(Some(CreateContainerOptions::default()), container).await.unwrap();

    let file = File::open(node.to_tar().await).map_ok(ReaderStream::new).try_flatten_stream();
    let body_stream = body_try_stream(file);

    let _upload_options = UploadToContainerOptions {
        path: "/tmp/".to_string(),
        ..Default::default()
    };

    docker
        .upload_to_container(&container.id, Some(_upload_options), body_stream)
        .await
        .unwrap();

    docker
        .start_container(&container.id, Some(StartContainerOptions::default()))
        .await
        .unwrap();

    docker
        .wait_container(&container.id, Some(WaitContainerOptions::default()))
        .for_each(|_| async {})
        .await;

    let logs = docker
        .logs(
            &container.id,
            Some(LogsOptions {
                follow: true,
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        )
        .collect::<Vec<_>>()
        .await;

    let output: String = logs
        .into_iter()
        .filter_map(Result::ok)
        .flat_map(|log| log.into_bytes().into_iter().map(|b| b as char))
        .collect();

    docker
        .remove_container(&container.id, Some(RemoveContainerOptions::default()))
        .await
        .unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "output": output.trim_end_matches('\n'),
        "language": node.language.to_string(),
        "output_type": node.output_type.to_string(),
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

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(get_code_node)
            .service(run_code_node)
            .service(create_code_node)
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
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
