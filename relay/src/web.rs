use anyhow::Result;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request, Response, StatusCode};
use mime_guess::from_path;
use serde_json::json;
use std::{collections::HashMap, path::Path, sync::Arc, time::UNIX_EPOCH};
use tokio::{fs, sync::RwLock};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::types::{InstanceInfo, SOVA_VERSION};

/// Simple template engine for replacing placeholders
fn render_template(template: &str, data: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in data {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Serve static files from the web directory
async fn serve_static_file(path: &str) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    // Validate path to prevent directory traversal
    if path.contains("..") || path.contains('\0') || path.contains('\\') {
        warn!("Blocked potential path traversal attempt: {}", path);
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("Forbidden")))
            .expect("Failed to build forbidden response"));
    }
    
    // Try container path first, fall back to development path
    let web_root = if Path::new("/opt/bubocore-relay/web").exists() {
        Path::new("/opt/bubocore-relay/web").to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
    };
    
    let file_path = web_root.join(path.trim_start_matches('/'));
    
    // Security check: canonicalize paths and ensure the file is within web directory
    let canonical_web_root = match web_root.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            warn!("Failed to canonicalize web root: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("Internal server error")))
                .expect("Failed to build error response"));
        }
    };
    
    let canonical_file_path = match file_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            // File doesn't exist or can't be canonicalized
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("File not found")))
                .expect("Failed to build not found response"));
        }
    };
    
    // Ensure the canonical file path is within the canonical web root
    if !canonical_file_path.starts_with(&canonical_web_root) {
        warn!("Blocked path traversal attempt: {} -> {}", path, canonical_file_path.display());
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("Forbidden")))
            .expect("Failed to build forbidden response"));
    }
    
    match fs::read(&canonical_file_path).await {
        Ok(contents) => {
            let mime_type = from_path(&canonical_file_path).first_or_octet_stream();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type.as_ref())
                .body(Full::new(Bytes::from(contents)))
                .expect("Failed to build success response"))
        }
        Err(_) => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("File not found")))
                .expect("Failed to build not found response"))
        }
    }
}

/// Generate the instances HTML content
fn generate_instances_html(instances: &[&InstanceInfo]) -> String {
    if instances.is_empty() {
        r#"<div class="empty">No instances currently connected</div>"#.to_string()
    } else {
        instances
            .iter()
            .map(|instance| {
                let connected_duration = instance.connected_at.elapsed().unwrap_or_default();
                format!(
                    r#"<div class="instance">
                        <strong>{}</strong><br>
                        <small>ID: {}</small><br>
                        <small>Connected: {}s ago</small><br>
                        <small>Version: {}</small>
                    </div>"#,
                    instance.name,
                    instance.id,
                    connected_duration.as_secs(),
                    instance.version
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Calculate server uptime (simplified)
fn get_uptime() -> String {
    // For now, just return "Running" - could be enhanced with actual start time
    "Running".to_string()
}

/// Handle HTTP requests for the web interface
pub async fn handle_http_request(
    req: Request<hyper::body::Incoming>,
    instances: Arc<RwLock<HashMap<Uuid, InstanceInfo>>>,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    let path = req.uri().path();
    debug!("HTTP request: {} {}", req.method(), path);
    
    match (req.method(), path) {
        (&Method::GET, "/") => {
            // Try container path first, fall back to development path
            let web_root = if Path::new("/opt/bubocore-relay/web").exists() {
                Path::new("/opt/bubocore-relay/web").to_path_buf()
            } else {
                Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
            };
            let template_path = web_root.join("templates/index.html");
            
            match fs::read_to_string(&template_path).await {
                Ok(template) => {
                    let instances_read = instances.read().await;
                    let instances_list: Vec<&InstanceInfo> = instances_read.values().collect();
                    let instance_count = instances_list.len();
                    
                    let mut template_data = HashMap::new();
                    template_data.insert("status", "Online".to_string());
                    template_data.insert("instance_count", instance_count.to_string());
                    template_data.insert("version", SOVA_VERSION.to_string());
                    template_data.insert("uptime", get_uptime());
                    template_data.insert("instances_html", generate_instances_html(&instances_list));
                    
                    let html = render_template(&template, &template_data);
                    
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(Full::new(Bytes::from(html)))
                        .expect("Failed to build HTML response"))
                }
                Err(e) => {
                    warn!("Failed to read template: {}", e);
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Template error")))
                        .expect("Failed to build error response"))
                }
            }
        }
        (&Method::GET, "/status") => {
            let instances_read = instances.read().await;
            let status = json!({
                "status": "online",
                "version": SOVA_VERSION,
                "connected_instances": instances_read.len(),
                "uptime": get_uptime(),
                "instances": instances_read.values().map(|i| {
                    json!({
                        "id": i.id,
                        "name": i.name,
                        "version": i.version,
                        "connected_at": i.connected_at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
                    })
                }).collect::<Vec<_>>()
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(status.to_string())))
                .expect("Failed to build JSON response"))
        }
        (&Method::GET, _) if path.starts_with("/css/") || path.starts_with("/js/") || path.starts_with("/assets/") => {
            serve_static_file(path).await
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404 Not Found")))
                .expect("Failed to build 404 response"))
        }
    }
}