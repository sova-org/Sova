mod client;
mod messages;
mod link;
mod disk;
mod server_manager;

use client::ClientManager;
use messages::{ClientMessage, ServerMessage, Snapshot};
use link::LinkClock;
use disk::ProjectInfo;
use server_manager::{ServerManager, ServerConfig, ServerState};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{Mutex, RwLock};

type ClientState = Arc<Mutex<ClientManager>>;
type MessagesState = Arc<RwLock<Vec<ServerMessage>>>;
type LinkState = Arc<LinkClock>;
type ServerManagerState = Arc<ServerManager>;

// Disk operation commands
#[tauri::command]
async fn list_projects() -> Result<Vec<ProjectInfo>, String> {
    disk::list_projects().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_project(snapshot: Snapshot, project_name: String) -> Result<(), String> {
    disk::save_project(&snapshot, &project_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_project(project_name: String) -> Result<Snapshot, String> {
    disk::load_project(&project_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_project(project_name: String) -> Result<(), String> {
    disk::delete_project(&project_name).await.map_err(|e| e.to_string())
}

// Network operation commands
#[tauri::command]
async fn connect_to_server(
    ip: String,
    port: u16,
    client_state: State<'_, ClientState>,
) -> Result<(), String> {
    let mut client = client_state.lock().await;
    client.connect(ip, port).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn disconnect_from_server(client_state: State<'_, ClientState>) -> Result<(), String> {
    let mut client = client_state.lock().await;
    client.disconnect();
    Ok(())
}

#[tauri::command]
async fn send_message(
    message: ClientMessage,
    client_state: State<'_, ClientState>,
) -> Result<(), String> {
    let client = client_state.lock().await;
    client.send_message(message).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_messages(messages_state: State<'_, MessagesState>) -> Result<Vec<ServerMessage>, String> {
    let mut messages = messages_state.write().await;
    let result = messages.clone();
    messages.clear();
    Ok(result)
}

#[tauri::command]
async fn is_connected(client_state: State<'_, ClientState>) -> Result<bool, String> {
    let client = client_state.lock().await;
    Ok(client.is_connected())
}

#[tauri::command]
fn get_link_phase(link_state: State<'_, LinkState>) -> Result<f64, String> {
    Ok(link_state.get_phase())
}

#[tauri::command]
fn get_link_tempo(link_state: State<'_, LinkState>) -> Result<f64, String> {
    Ok(link_state.get_tempo())
}

#[tauri::command]
fn set_link_tempo(tempo: f64, link_state: State<'_, LinkState>) -> Result<(), String> {
    link_state.set_tempo(tempo);
    Ok(())
}

#[tauri::command]
fn set_link_quantum(quantum: f64, link_state: State<'_, LinkState>) -> Result<(), String> {
    link_state.set_quantum(quantum);
    Ok(())
}

#[tauri::command]
fn get_link_quantum(link_state: State<'_, LinkState>) -> Result<f64, String> {
    Ok(link_state.get_quantum())
}

// Server management commands
#[tauri::command]
async fn get_server_state(server_manager: State<'_, ServerManagerState>) -> Result<ServerState, String> {
    Ok(server_manager.get_state())
}


#[tauri::command]
async fn shutdown_app(
    server_manager: State<'_, ServerManagerState>,
    client_state: State<'_, ClientState>,
) -> Result<(), String> {
    // First disconnect the client
    {
        let mut client = client_state.lock().await;
        client.disconnect();
    }
    
    // Then stop the server if it's running
    let server_state = server_manager.get_state();
    if matches!(server_state.status, server_manager::ServerStatus::Running) {
        server_manager.stop_server().await.map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn close_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
async fn update_server_config(
    config: ServerConfig,
    server_manager: State<'_, ServerManagerState>,
) -> Result<(), String> {
    server_manager.update_config(config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_server(server_manager: State<'_, ServerManagerState>) -> Result<(), String> {
    server_manager.start_server().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_server(server_manager: State<'_, ServerManagerState>) -> Result<(), String> {
    server_manager.stop_server().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn restart_server(server_manager: State<'_, ServerManagerState>) -> Result<(), String> {
    server_manager.restart_server().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_server_logs(
    limit: usize,
    server_manager: State<'_, ServerManagerState>,
) -> Result<Vec<server_manager::LogEntry>, String> {
    Ok(server_manager.get_recent_logs(limit))
}

#[tauri::command]
async fn list_audio_devices(server_manager: State<'_, ServerManagerState>) -> Result<Vec<String>, String> {
    server_manager.list_audio_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn detect_running_server(server_manager: State<'_, ServerManagerState>) -> Result<bool, String> {
    server_manager.detect_running_server().await.map_err(|e| e.to_string())
}

async fn message_polling_task(
    client_state: ClientState,
    messages_state: MessagesState,
    app_handle: AppHandle,
    link_state: LinkState,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
    
    loop {
        interval.tick().await;
        
        let mut client = client_state.lock().await;
        while let Some(message) = client.try_receive_message() {
            {
                let mut messages = messages_state.write().await;
                messages.push(message.clone());
            }
            
            // Sync Link state based on server messages
            match &message {
                ServerMessage::Hello { link_state: link_info, .. } => {
                    let (tempo, _beat, _phase, _peers, _enabled) = link_info;
                    link_state.set_tempo(*tempo);
                }
                ServerMessage::ClockState(tempo, _beat, _micros, quantum) => {
                    link_state.set_tempo(*tempo);
                    link_state.set_quantum(*quantum);
                }
                _ => {}
            }
            
            if let Err(e) = app_handle.emit("server-message", &message) {
                eprintln!("Failed to emit server message: {}", e);
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    tauri::async_runtime::set(runtime.handle().clone());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let client_state: ClientState = Arc::new(Mutex::new(ClientManager::new()));
            let messages_state: MessagesState = Arc::new(RwLock::new(Vec::new()));
            let link_state: LinkState = Arc::new(LinkClock::new());
            let server_manager_state: ServerManagerState = Arc::new(ServerManager::new());
            
            app.manage(client_state.clone());
            app.manage(messages_state.clone());
            app.manage(link_state.clone());
            app.manage(server_manager_state.clone());
            
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(message_polling_task(client_state, messages_state, app_handle, link_state));
            
            // Handle window close events
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent the window from closing
                    api.prevent_close();
                    
                    // Emit an event to the frontend to show the confirmation dialog
                    let _ = window_clone.emit("show-close-confirmation", ());
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            save_project,
            load_project,
            delete_project,
            connect_to_server,
            disconnect_from_server,
            send_message,
            get_messages,
            is_connected,
            get_link_phase,
            get_link_tempo,
            set_link_tempo,
            set_link_quantum,
            get_link_quantum,
            get_server_state,
            update_server_config,
            start_server,
            stop_server,
            restart_server,
            get_server_logs,
            list_audio_devices,
            detect_running_server,
            shutdown_app,
            close_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
