use anyhow::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::{mpsc::channel, Arc};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use super::loader::ConfigLoader;
use super::types::{ConfigUpdateEvent, ServerConfig};
use crate::server_manager::ServerManager;

type ServerManagerState = Arc<Mutex<ServerManager>>;

pub fn start_watcher(app_handle: AppHandle, server_manager: ServerManagerState) -> Result<()> {
    std::thread::spawn(move || {
        if let Err(e) = watch_config_file(app_handle, server_manager) {
            sova_core::log_error!("Config watcher error: {}", e);
        }
    });

    Ok(())
}

fn watch_config_file(app_handle: AppHandle, server_manager: ServerManagerState) -> Result<()> {
    let loader = ConfigLoader::new()?;
    let config_path = loader.config_path().clone();

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        NotifyConfig::default()
            .with_poll_interval(Duration::from_secs(2))
    )?;

    watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

    sova_core::log_info!("Watching config file: {:?}", config_path);

    // Load initial config to establish baseline for comparison
    // This ensures the first detected change is properly handled
    let initial_config = loader.load().ok();
    let mut previous_server_config: Option<ServerConfig> =
        initial_config.as_ref().map(|c| c.server.clone());

    for res in rx {
        match res {
            Ok(_event) => {
                match loader.load() {
                    Ok(config) => {
                        let new_server_config = config.server.clone();

                        if let Some(old_config) = &previous_server_config {
                            handle_server_config_change(
                                old_config,
                                &new_server_config,
                                server_manager.clone(),
                            );
                        }

                        previous_server_config = Some(new_server_config);

                        let event = ConfigUpdateEvent {
                            editor: config.editor,
                            appearance: config.appearance,
                            server: config.server,
                            client: config.client,
                        };

                        if let Err(e) = app_handle.emit("config-update", &event) {
                            sova_core::log_error!("Failed to emit config-update event: {}", e);
                        }
                    }
                    Err(e) => {
                        sova_core::log_error!("Failed to reload config: {}", e);
                    }
                }
            }
            Err(e) => sova_core::log_error!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

fn handle_server_config_change(
    old_config: &ServerConfig,
    new_config: &ServerConfig,
    server_manager: ServerManagerState,
) {
    let old_config = old_config.clone();
    let new_config = new_config.clone();

    tauri::async_runtime::spawn(async move {
        let mut manager = server_manager.lock().await;

        if old_config.enabled != new_config.enabled {
            if new_config.enabled {
                sova_core::log_info!("Server enabled in config, starting server on port {}", new_config.port);
                if let Err(e) = manager.start_server(new_config.port).await {
                    sova_core::log_error!("Failed to start server: {}", e);
                } else {
                    sova_core::log_info!("Server started successfully");
                }
            } else {
                sova_core::log_info!("Server disabled in config, stopping server");
                if let Err(e) = manager.stop_server().await {
                    sova_core::log_error!("Failed to stop server: {}", e);
                } else {
                    sova_core::log_info!("Server stopped successfully");
                }
            }
        } else if new_config.enabled {
            if old_config.port != new_config.port || old_config.ip != new_config.ip {
                sova_core::log_info!("Server config changed (port: {} -> {}, ip: {} -> {}), restarting server",
                    old_config.port, new_config.port, old_config.ip, new_config.ip);

                if let Err(e) = manager.stop_server().await {
                    sova_core::log_error!("Failed to stop server for restart: {}", e);
                    return;
                }

                if let Err(e) = manager.start_server(new_config.port).await {
                    sova_core::log_error!("Failed to start server after config change: {}", e);
                } else {
                    sova_core::log_info!("Server restarted successfully");
                }
            }
        }
    });
}
