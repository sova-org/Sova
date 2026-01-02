use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::{ShellExt, process::{CommandChild, CommandEvent}};

pub struct ServerManager {
    child: Option<CommandChild>,
    pid: Option<u32>,
    port: u16,
    ip: String,
    app_handle: AppHandle,
    is_alive: Arc<AtomicBool>,
}

impl ServerManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            child: None,
            pid: None,
            port: 8080,
            ip: "127.0.0.1".to_string(),
            app_handle,
            is_alive: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_pid(&self) -> Option<u32> {
        self.pid
    }

    fn kill_process_by_pid(pid: u32) {
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

    pub async fn start_server(&mut self, port: u16) -> Result<(), String> {
        self.start_server_with_ip(port, "127.0.0.1".to_string()).await
    }

    pub async fn start_server_with_ip(&mut self, port: u16, ip: String) -> Result<(), String> {
        if self.is_running() {
            return Err("Server already running".to_string());
        }

        let sidecar = self.app_handle
            .shell()
            .sidecar("sova_server")
            .map_err(|e| format!("Failed to create sidecar: {}", e))?
            .args(["--ip", &ip, "--port", &port.to_string()]);

        let (mut rx, child) = sidecar
            .spawn()
            .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

        self.pid = Some(child.pid());
        self.child = Some(child);
        self.port = port;
        self.ip = ip;
        self.is_alive.store(true, Ordering::SeqCst);

        let app_handle = self.app_handle.clone();
        let is_alive = self.is_alive.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let msg = String::from_utf8_lossy(&line).to_string();
                        let _ = app_handle.emit("server:log", msg);
                    }
                    CommandEvent::Stderr(line) => {
                        let msg = String::from_utf8_lossy(&line).to_string();
                        let _ = app_handle.emit("server:error", msg);
                    }
                    CommandEvent::Terminated(payload) => {
                        is_alive.store(false, Ordering::SeqCst);
                        let _ = app_handle.emit("server:terminated", payload.code);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), String> {
        self.is_alive.store(false, Ordering::SeqCst);
        if let Some(child) = self.child.take() {
            let _ = child.kill();
        }
        if let Some(pid) = self.pid.take() {
            Self::kill_process_by_pid(pid);
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some() && self.is_alive.load(Ordering::SeqCst)
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        if let Some(pid) = self.pid.take() {
            Self::kill_process_by_pid(pid);
        }
    }
}
