use std::sync::Arc;

use crate::code_nodes::{CodeLanguage, Entity as CodeNode};
use crate::config;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use bollard::query_parameters::RemoveContainerOptions;
use futures_util::io::BufWriter;
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use tokio::io::BufReader;

#[get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[get("/code-node/{id}/")]
async fn get_code_node(id: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let post = CodeNode::find_by_id(id.into_inner())
        .one(&*data.db)
        .await
        .unwrap();

    match post {
        Some(p) => HttpResponse::Ok().json(p),
        None => HttpResponse::NotFound().body("Post not found"),
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
    use futures_util::{StreamExt, TryFutureExt};
    use tokio::fs::File;
    use tokio_tar as tar;
    use tokio_util::io::ReaderStream;
    use unescape::unescape;

    let node = CodeNode::find_by_id(id.into_inner())
        .one(&*data.db)
        .await
        .unwrap();

    if node.is_none() {
        return HttpResponse::NotFound().body("Code node not found");
    }

    let node = node.unwrap();

    let docker = Docker::connect_with_defaults().unwrap();

    let image_name = match node.language {
        CodeLanguage::Python => "python:3.12",
        CodeLanguage::JavaScript => "node:latest",
    };

    let extension = match node.language {
        CodeLanguage::Python => "py",
        CodeLanguage::JavaScript => "js",
    };

    let cmd = match node.language {
        CodeLanguage::Python => vec!["python".to_string(), format!("/tmp/{}.py", node.name)],
        CodeLanguage::JavaScript => todo!(),
    };
    let container = ContainerCreateBody {
        working_dir: Some("/tmp".to_string()),
        image: Some(image_name.to_string()),
        cmd: Some(cmd),
        // cmd: Some(vec![
        //     "tail".to_string(),
        //     "-f".to_string(),
        //     "/dev/null".to_string(),
        // ]),
        // cmd: Some(vec![
        //     "ls".to_string(),
        //     "-la".to_string(),
        // ]),
        ..Default::default()
    };

    let unescaped_code = unescape(&node.code).unwrap();
    let code_content = unescaped_code.trim_matches(char::from(0));
    let altered_code = match node.language {
        CodeLanguage::Python => format!("{}\nprint({}())", code_content, node.function_name),
        CodeLanguage::JavaScript => format!("{}\nconsole.log({}());", code_content, node.function_name),
    };

    let file_path = format!("/tmp/{}.{}", node.name, extension);
    tokio::fs::write(&file_path, altered_code)
        .await
        .expect("Failed to write code to file");

    let container = docker
        .create_container(Some(CreateContainerOptions::default()), container)
        .await
        .unwrap();

    // // Create a temporary tar file with the code content
    let tar_path = format!("/tmp/{}.tar", node.name);
    {
        let tar_file = File::create(&tar_path).await.unwrap();
        let mut tar_builder = tar::Builder::new(tar_file);

        tar_builder
            .append_file(
                format!("{}.{}", node.name, extension),
                &mut File::open(&file_path).await.unwrap(),
            )
            .await
            .unwrap();
    }

    // // Upload the tar file to the container

    let file = File::open(tar_path)
        .map_ok(ReaderStream::new)
        .try_flatten_stream();
    let body_stream = body_try_stream(file);

    docker
        .upload_to_container(
            &container.id,
            Some(UploadToContainerOptions {
                path: "/tmp/".to_string(),
                ..Default::default()
            }),
            body_stream,
        )
        .await
        .unwrap();

    docker
        .start_container(&container.id, Some(StartContainerOptions::default()))
        .await
        .unwrap();

    let logs = docker
        .logs(
            &container.id,
            Some(LogsOptions {
                stdout: true,
                stderr: true,
                follow: true,
                ..Default::default()
            }),
        )
        .collect::<Vec<_>>()
        .await;

    let mut output = String::new();
    for log in logs {
        if let Ok(log) = log {
            output.push_str(&log.to_string());
        }
    }

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
struct AppState {
    db: Arc<DatabaseConnection>,
    config: config::Config,
}

#[actix_web::main]
pub async fn server() -> std::io::Result<()> {
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
    let app_state = AppState {
        db: _conn.clone(),
        config: config.clone(),
    };

    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(get_code_node)
            .service(run_code_node)
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

pub fn main() {
    let result = server();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
