use prism_mcp_rs::prelude::*;
use tauri::AppHandle;
use crate::mcp::cmd;

pub async fn start(app_handle: AppHandle) -> McpResult<()> {
    let mut server = McpServer::new("bytesto4t".to_string(), "0.2.2".to_string());

    cmd::get_function_list::register(&mut server, app_handle.clone()).await?;
    cmd::load_bytecode::register(&mut server, app_handle.clone()).await?;
    cmd::add_function::register(&mut server, app_handle.clone()).await?;
    cmd::update_function::register(&mut server, app_handle.clone()).await?;
    cmd::remove_function::register(&mut server, app_handle.clone()).await?;
    cmd::get_function_full_info::register(&mut server, app_handle.clone()).await?;
    cmd::get_function_name_by_index::register(&mut server, app_handle.clone()).await?;
    cmd::list_functions_with_constructors::register(&mut server, app_handle.clone()).await?;
    cmd::find_functions_using_type::register(&mut server, app_handle.clone()).await?;
    cmd::import_function_json::register(&mut server, app_handle.clone()).await?;
    cmd::export_function_json::register(&mut server, app_handle.clone()).await?;

    cmd::get_type_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_type::register(&mut server, app_handle.clone()).await?;
    cmd::update_type::register(&mut server, app_handle.clone()).await?;
    cmd::remove_type::register(&mut server, app_handle.clone()).await?;
    cmd::get_type_full_info::register(&mut server, app_handle.clone()).await?;
    cmd::import_type_json::register(&mut server, app_handle.clone()).await?;
    cmd::export_type_json::register(&mut server, app_handle.clone()).await?;
    cmd::generate_imhex_pattern::register(&mut server, app_handle.clone()).await?;

    server.start(prism_mcp_rs::transport::stdio::StdioServerTransport::new()).await?;
    Ok(())
}