mod config;
mod client_manager;
mod server_manager;

use config::loader::ConfigLoader;
use config::types::{Config, ConfigUpdateEvent};
use config::watcher;
use tauri::{Emitter, Manager};
use std::sync::Arc;
use tokio::sync::Mutex;
use server_manager::ServerManager;
use client_manager::ClientManager;

type ServerManagerState = Arc<Mutex<ServerManager>>;
type ClientManagerState = Arc<Mutex<ClientManager>>;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    let loader = ConfigLoader::new()
        .map_err(|e| e.to_string())?;

    loader.load_or_create()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config_content() -> Result<String, String> {
    let loader = ConfigLoader::new()
        .map_err(|e| e.to_string())?;

    std::fs::read_to_string(loader.config_path())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config_content(content: String) -> Result<(), String> {
    use config::validation::Validate;

    let mut config: Config = toml::from_str(&content)
        .map_err(|e| format!("Invalid TOML syntax: {}", e))?;

    config.validate();

    let loader = ConfigLoader::new()
        .map_err(|e| e.to_string())?;

    std::fs::write(loader.config_path(), content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn start_server(
    port: u16,
    server_manager: tauri::State<'_, ServerManagerState>,
) -> Result<(), String> {
    server_manager.lock().await.start_server(port).await
}

#[tauri::command]
async fn stop_server(
    server_manager: tauri::State<'_, ServerManagerState>,
) -> Result<(), String> {
    server_manager.lock().await.stop_server().await
}

#[tauri::command]
async fn is_server_running(
    server_manager: tauri::State<'_, ServerManagerState>,
) -> Result<bool, String> {
    Ok(server_manager.lock().await.is_running())
}

#[tauri::command]
async fn connect_client(
    ip: String,
    port: u16,
    username: String,
    client_manager: tauri::State<'_, ClientManagerState>,
) -> Result<(), String> {
    let mut client = client_manager.lock().await;
    client.connect(ip, port).await.map_err(|e| e.to_string())?;
    client.send_message(sova_core::server::client::ClientMessage::SetName(username))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn disconnect_client(
    client_manager: tauri::State<'_, ClientManagerState>,
) -> Result<(), String> {
    client_manager.lock().await.disconnect();
    Ok(())
}

#[tauri::command]
async fn is_client_connected(
    client_manager: tauri::State<'_, ClientManagerState>,
) -> Result<bool, String> {
    Ok(client_manager.lock().await.is_connected())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let server_manager = Arc::new(Mutex::new(
                ServerManager::new(app.handle().clone())
            ));
            app.manage(server_manager.clone());

            let client_manager = Arc::new(Mutex::new(
                ClientManager::new()
            ));
            app.manage(client_manager);

            match ConfigLoader::new().and_then(|l| l.load_or_create()) {
                Ok(config) => {
                    if config.server.enabled {
                        let server_manager_clone = server_manager.clone();
                        let port = config.server.port;
                        tauri::async_runtime::spawn(async move {
                            println!("Auto-starting server on port {} (enabled in config)", port);
                            match server_manager_clone.lock().await.start_server(port).await {
                                Ok(_) => println!("Server started successfully"),
                                Err(e) => eprintln!("Failed to auto-start server: {}", e),
                            }
                        });
                    }

                    let event = ConfigUpdateEvent {
                        editor: config.editor,
                        appearance: config.appearance,
                        server: config.server,
                    };
                    let _ = app.emit("config-update", &event);
                }
                Err(e) => {
                    eprintln!("Failed to load initial config: {}. Using defaults.", e);
                    let _ = app.emit("config-update", &ConfigUpdateEvent {
                        editor: config::types::EditorConfig::default(),
                        appearance: config::types::AppearanceConfig::default(),
                        server: config::types::ServerConfig::default(),
                    });
                }
            }

            watcher::start_watcher(app.handle().clone(), server_manager.clone())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            get_config_content,
            save_config_content,
            start_server,
            stop_server,
            is_server_running,
            connect_client,
            disconnect_client,
            is_client_connected
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let server_manager = app_handle.state::<ServerManagerState>();
                tauri::async_runtime::block_on(async {
                    let _ = server_manager.lock().await.stop_server().await;
                });

                let client_manager = app_handle.state::<ClientManagerState>();
                tauri::async_runtime::block_on(async {
                    client_manager.lock().await.disconnect();
                });
            }
        });
}
