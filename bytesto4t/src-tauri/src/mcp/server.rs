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

    cmd::get_dashboard_info::register(&mut server, app_handle.clone()).await?;
    cmd::get_decompiled_info::register(&mut server, app_handle.clone()).await?;
    cmd::get_disassembler_info::register(&mut server, app_handle.clone()).await?;
    cmd::get_inspector_info::register(&mut server, app_handle.clone()).await?;
    cmd::set_target_file_path::register(&mut server, app_handle.clone()).await?;
    cmd::clear_references::register(&mut server, app_handle.clone()).await?;
    cmd::get_saved_references::register(&mut server, app_handle.clone()).await?;
    cmd::read_binary_file::register(&mut server, app_handle.clone()).await?;
    cmd::merge_bytecode_with_file::register(&mut server, app_handle.clone()).await?;

    cmd::get_file_list::register(&mut server, app_handle.clone()).await?;
    cmd::get_string_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_string::register(&mut server, app_handle.clone()).await?;
    cmd::update_string::register(&mut server, app_handle.clone()).await?;
    cmd::remove_string::register(&mut server, app_handle.clone()).await?;
    cmd::get_string_full_info::register(&mut server, app_handle.clone()).await?;

    cmd::get_constant_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_constant::register(&mut server, app_handle.clone()).await?;
    cmd::update_constant::register(&mut server, app_handle.clone()).await?;
    cmd::remove_constant::register(&mut server, app_handle.clone()).await?;
    cmd::get_constant_full_info::register(&mut server, app_handle.clone()).await?;

    cmd::get_int_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_int::register(&mut server, app_handle.clone()).await?;
    cmd::update_int::register(&mut server, app_handle.clone()).await?;
    cmd::remove_int::register(&mut server, app_handle.clone()).await?;
    cmd::get_int_full_info::register(&mut server, app_handle.clone()).await?;

    cmd::get_float_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_float::register(&mut server, app_handle.clone()).await?;
    cmd::update_float::register(&mut server, app_handle.clone()).await?;
    cmd::remove_float::register(&mut server, app_handle.clone()).await?;
    cmd::get_float_full_info::register(&mut server, app_handle.clone()).await?;

    cmd::get_bytes_list::register(&mut server, app_handle.clone()).await?;
    cmd::get_bytes_full_info::register(&mut server, app_handle.clone()).await?;

    cmd::get_type_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_type::register(&mut server, app_handle.clone()).await?;
    cmd::update_type::register(&mut server, app_handle.clone()).await?;
    cmd::remove_type::register(&mut server, app_handle.clone()).await?;
    cmd::get_type_full_info::register(&mut server, app_handle.clone()).await?;
    cmd::get_global_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_global::register(&mut server, app_handle.clone()).await?;
    cmd::update_global::register(&mut server, app_handle.clone()).await?;
    cmd::remove_global::register(&mut server, app_handle.clone()).await?;
    cmd::get_global_full_info::register(&mut server, app_handle.clone()).await?;
    cmd::get_native_list::register(&mut server, app_handle.clone()).await?;
    cmd::add_native::register(&mut server, app_handle.clone()).await?;
    cmd::update_native::register(&mut server, app_handle.clone()).await?;
    cmd::remove_native::register(&mut server, app_handle.clone()).await?;
    cmd::get_native_full_info::register(&mut server, app_handle.clone()).await?;
    cmd::import_type_json::register(&mut server, app_handle.clone()).await?;
    cmd::export_type_json::register(&mut server, app_handle.clone()).await?;
    cmd::generate_imhex_pattern::register(&mut server, app_handle.clone()).await?;

    server.start(prism_mcp_rs::transport::stdio::StdioServerTransport::new()).await?;
    Ok(())
}
