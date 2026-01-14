mod client_manager;
mod disk;
mod server_manager;

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;
use server_manager::ServerManager;
use client_manager::ClientManager;

type ServerManagerState = Arc<Mutex<ServerManager>>;
type ClientManagerState = Arc<Mutex<ClientManager>>;

#[derive(serde::Serialize)]
struct AudioDeviceInfo {
    name: String,
    index: usize,
    max_channels: u16,
    is_default: bool,
}

#[tauri::command]
fn list_audio_devices() -> Vec<AudioDeviceInfo> {
    doux::audio::list_output_devices()
        .into_iter()
        .map(|d| AudioDeviceInfo {
            name: d.name,
            index: d.index,
            max_channels: d.max_channels,
            is_default: d.is_default,
        })
        .collect()
}

#[tauri::command]
fn list_audio_input_devices() -> Vec<AudioDeviceInfo> {
    doux::audio::list_input_devices()
        .into_iter()
        .map(|d| AudioDeviceInfo {
            name: d.name,
            index: d.index,
            max_channels: d.max_channels,
            is_default: d.is_default,
        })
        .collect()
}

#[tauri::command]
async fn start_server(
    port: u16,
    audio_enabled: bool,
    audio_device: Option<String>,
    audio_input_device: Option<String>,
    audio_channels: u16,
    sample_paths: Vec<String>,
    server_manager: tauri::State<'_, ServerManagerState>,
) -> Result<(), String> {
    server_manager.lock().await.start_server_with_audio(
        port,
        audio_enabled,
        audio_device,
        audio_input_device,
        audio_channels,
        sample_paths,
    ).await
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
    client.send_message(sova_server::ClientMessage::SetName(username))
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

#[tauri::command]
async fn send_client_message(
    message: sova_server::ClientMessage,
    client_manager: tauri::State<'_, ClientManagerState>,
) -> Result<(), String> {
    client_manager.lock().await.send_message(message)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn create_default_frame() -> sova_core::scene::Frame {
    sova_core::scene::Frame::default()
}

#[tauri::command]
fn create_default_line() -> sova_core::scene::Line {
    sova_core::scene::Line::default()
}

#[tauri::command]
async fn list_projects() -> Result<Vec<disk::ProjectInfo>, String> {
    disk::list_projects().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_project(
    snapshot: sova_server::Snapshot,
    project_name: String,
) -> Result<(), String> {
    disk::save_project(&snapshot, &project_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_project(project_name: String) -> Result<sova_server::Snapshot, String> {
    disk::load_project(&project_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_project(project_name: String) -> Result<(), String> {
    disk::delete_project(&project_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn rename_project(old_name: String, new_name: String) -> Result<(), String> {
    disk::rename_project(&old_name, &new_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_projects_folder() -> Result<(), String> {
    let path = disk::get_projects_directory()
        .await
        .map_err(|e| e.to_string())?;
    tauri_plugin_opener::open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_project(path: String) -> Result<sova_server::Snapshot, String> {
    disk::load_project_from_path(std::path::Path::new(&path))
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let server_manager = Arc::new(Mutex::new(
                ServerManager::new(app.handle().clone())
            ));
            app.manage(server_manager);

            let client_manager = Arc::new(Mutex::new(
                ClientManager::new(app.handle().clone())
            ));
            app.manage(client_manager);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            is_server_running,
            connect_client,
            disconnect_client,
            is_client_connected,
            send_client_message,
            create_default_frame,
            create_default_line,
            list_projects,
            save_project,
            load_project,
            delete_project,
            rename_project,
            open_projects_folder,
            import_project,
            list_audio_devices,
            list_audio_input_devices
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    let server_manager = app_handle.state::<ServerManagerState>();
                    let pid = server_manager.try_lock().ok().and_then(|g| g.get_pid());
                    if let Some(pid) = pid {
                        #[cfg(unix)]
                        {
                            let _ = std::process::Command::new("kill")
                                .args(["-9", &pid.to_string()])
                                .output();
                        }
                        #[cfg(windows)]
                        {
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string()])
                                .output();
                        }
                    }
                }
                tauri::RunEvent::Exit => {
                    let cleanup_timeout = std::time::Duration::from_secs(2);

                    let server_manager = app_handle.state::<ServerManagerState>();
                    let _ = tauri::async_runtime::block_on(async {
                        let _ = tokio::time::timeout(cleanup_timeout, async {
                            let _ = server_manager.lock().await.stop_server().await;
                        }).await;
                    });

                    let client_manager = app_handle.state::<ClientManagerState>();
                    let _ = tauri::async_runtime::block_on(async {
                        let _ = tokio::time::timeout(cleanup_timeout, async {
                            client_manager.lock().await.disconnect();
                        }).await;
                    });
                }
                _ => {}
            }
        });
}
