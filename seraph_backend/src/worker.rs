use std::sync::Arc;

use bollard::Docker;
use bollard::body_try_stream;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::CreateContainerOptions;
use bollard::query_parameters::LogsOptions;
use bollard::query_parameters::StartContainerOptions;
use bollard::query_parameters::UploadToContainerOptions;
use bollard::query_parameters::WaitContainerOptions;
use futures_util::{StreamExt, TryFutureExt};
use sea_orm::DatabaseConnection;
use tokio::fs::File;
use tokio::sync::mpsc;
use tokio_util::io::ReaderStream;
use uuid;

use crate::code_nodes::Entity as CodeNode;
use bollard::query_parameters::RemoveContainerOptions;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

#[derive(Debug, Clone)]
pub struct CodeNodeTask {
    pub id: uuid::Uuid,
    pub node_id: i32,
    db: Arc<DatabaseConnection>,
}

impl CodeNodeTask {
    pub fn new(node_id: i32, db: Arc<DatabaseConnection>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            node_id,
            db,
        }
    }
}

pub async fn worker(mut receiver: mpsc::Receiver<CodeNodeTask>) {
    while let Some(task) = receiver.recv().await {
        tracing::info!("Processing code node with ID: {}", task.id);

        let node = match CodeNode::find_by_id(task.node_id).one(&*task.db).await {
            Ok(Some(node)) => node,
            _ => {
                tracing::error!("Code node with ID {} not found", task.id);
                continue;
            }
        };

        let code_result = crate::code_result::ActiveModel {
            code_node_id: Set(node.id),
            status: Set(crate::enums::ResultStatus::Pending),
            output: Set(None),
            task_id: Set(task.id),
            ..Default::default()
        };

        let mut code_result: crate::code_result::ActiveModel = code_result.insert(&*task.db).await.unwrap().into();

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

        code_result.status = Set(crate::enums::ResultStatus::Running);
        let mut code_result: crate::code_result::ActiveModel = code_result.update(&*task.db).await.unwrap().into();

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
            .flat_map(|log| log.into_bytes().into_iter().map(|b: u8| b as char))
            .collect();

        docker
            .remove_container(&container.id, Some(RemoveContainerOptions::default()))
            .await
            .unwrap();

        code_result.status = Set(crate::enums::ResultStatus::Success);
        code_result.output = Set(Some(output.clone()));
        code_result.update(&*task.db).await.unwrap();

        tracing::info!("Successfully processed code node with ID: {}", task.id);
    }
}
