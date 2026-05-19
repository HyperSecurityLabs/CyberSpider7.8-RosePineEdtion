use anyhow::Result;
use tokio::sync::mpsc;
use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::distributed::NodeMessage;

pub struct HttpServer {
    message_sender: mpsc::UnboundedSender<NodeMessage>,
    node_id: String,
}

impl HttpServer {
    pub fn new(message_sender: mpsc::UnboundedSender<NodeMessage>, node_id: String) -> Self {
        Self {
            message_sender,
            node_id,
        }
    }

    pub async fn start(self, port: u16) -> Result<()> {
        let message_sender = Arc::new(Mutex::new(self.message_sender));
        let node_id = self.node_id.clone();
        let node_id_clone = node_id.clone();

        let message_handler = warp::path("api")
            .and(warp::path("message"))
            .and(warp::post())
            .and(warp::header::<String>("x-node-id"))
            .and(warp::body::json())
            .and(warp::any().map(move || message_sender.clone()))
            .and(warp::any().map(move || node_id.clone()))
            .and_then(
                |sender_id: String,
                 message: NodeMessage,
                 sender: Arc<Mutex<mpsc::UnboundedSender<NodeMessage>>>,
                 node_id: String| async move {
                    if sender_id == node_id {
                        return Err(warp::reject::custom(BadRequest("Self-message not allowed".to_string())));
                    }

                    if let Err(_) = sender.lock().await.send(message) {
                        return Err(warp::reject::custom(ServiceUnavailable("Message channel closed".to_string())));
                    }

                    Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                        "status": "ok",
                        "message": "Message received"
                    })))
                },
            );

        let health_check = warp::path("health")
            .and(warp::get())
            .and(warp::any().map(move || node_id_clone.clone()))
            .and_then(|node_id: String| async move {
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "status": "healthy",
                    "node_id": node_id,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })))
            });

        let routes = message_handler.or(health_check).with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec!["content-type", "x-node-id", "x-node-type"])
                .allow_methods(vec!["GET", "POST", "OPTIONS"])
        );

        println!("HTTP server starting on port {}", port);
        warp::serve(routes).run(([0, 0, 0, 0], port)).await;

        Ok(())
    }
}

#[derive(Debug)]
pub struct BadRequest(pub String);
impl warp::reject::Reject for BadRequest {}

#[derive(Debug)]
pub struct ServiceUnavailable(pub String);
impl warp::reject::Reject for ServiceUnavailable {}
