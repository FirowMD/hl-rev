use prism_mcp_rs::prelude::*;
use tauri::AppHandle;
use crate::mcp::cmd;

pub async fn start(app_handle: AppHandle) -> McpResult<()> {
    let mut server = McpServer::new("bytesto4t".to_string(), "0.2.2".to_string());

    cmd::get_function_list::register(&mut server, app_handle.clone()).await?;
    cmd::load_bytecode::register(&mut server, app_handle.clone()).await?;

    server.start(prism_mcp_rs::transport::stdio::StdioServerTransport::new()).await?;
    Ok(())
}