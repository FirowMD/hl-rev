pub mod server;
pub mod cmd;

pub fn start_server(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let _ = server::start(app_handle).await;
    });
}