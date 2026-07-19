mod app_config;
mod app_data;
mod bytecode_refs;
mod commands;
mod mcp;
mod structgen;

use crate::app_data::Storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let is_mcp_mode = std::env::args().any(|a| a == "--mcp");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Storage::default());

    if is_mcp_mode {
        let app = builder
            .build(tauri::generate_context!())
            .expect("error while building tauri application");
        if let Err(error) =
            tauri::async_runtime::block_on(crate::mcp::server::start(app.handle().clone()))
        {
            eprintln!("MCP server failed: {error}");
            std::process::exit(1);
        }
        return;
    }

    builder
        .invoke_handler(commands::invoke::handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
