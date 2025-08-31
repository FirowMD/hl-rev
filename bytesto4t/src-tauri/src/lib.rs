mod structgen;
mod app_config;
mod ai_decomp;
mod app_data;
mod commands;

use std::sync::Mutex;
use crate::app_config::AppConfig;
use crate::app_data::{AppData, Storage};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Storage {
            app_data: Mutex::new(AppData {
                target_file_path: String::new(),
                bytecode: None,
                app_config: AppConfig {
                    file_path: String::new(),
                    theme: None,
                    colorscheme: None,
                    recent_files: None,
                    openrouter_key: None,
                    ai_decompiler: None,
                    ai_prompt: None,
                },
                ai_decompilations: None,
                selected_item: None,
                function_addresses: None,
                history_items: Mutex::new(Vec::new()),
                references: None,
            }),
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::bytecode::set_target_file_path,
            commands::bytecode::get_dashboard_info,
            commands::bytecode::set_selected_item,
            commands::bytecode::get_selected_item,
            commands::bytecode::get_target_file_path,
            commands::bytecode::clear_references,
            commands::bytecode::get_saved_references,
            commands::bytecode::read_binary_file,
            
            commands::functions::get_function_list,
            commands::functions::list_functions_with_constructors,
            commands::functions::create_function,
            commands::functions::delete_function,
            commands::functions::update_function,
            commands::functions::get_function_full_info,
            commands::functions::get_function_name_by_index,
            commands::functions::load_function_addresses_from_file,
            commands::functions::get_function_addresses,
            
            commands::types::get_type_list,
            commands::types::create_type,
            commands::types::update_type,
            commands::types::delete_type,
            commands::types::get_type_full_info,
            commands::types::import_type_json,
            commands::types::export_type_json,
            commands::types::generate_imhex_pattern,
            
            commands::types::create_global,
            commands::types::update_global,
            commands::types::delete_global,
            commands::types::get_global_full_info,
            
            commands::types::create_native,
            commands::types::update_native,
            commands::types::delete_native,
            commands::types::get_native_full_info,
            
            commands::types::create_constant,
            commands::types::update_constant,
            commands::types::delete_constant,
            commands::types::get_constant_full_info,

            commands::types::create_string,
            commands::types::update_string,
            commands::types::delete_string,
            commands::types::get_string_full_info,
            
            commands::types::create_int,
            commands::types::update_int,
            commands::types::delete_int,
            commands::types::get_int_full_info,
            
            commands::types::create_float,
            commands::types::update_float,
            commands::types::delete_float,
            commands::types::get_float_full_info,
            
            commands::data::get_file_list,
            commands::data::get_string_list,
            commands::data::get_global_list,
            commands::data::get_native_list,
            commands::data::get_constant_list,
            commands::data::get_int_list,
            commands::data::get_float_list,
            
            commands::decompiler::get_decompiled_info,
            commands::decompiler::get_inspector_info,
            commands::decompiler::get_disassembler_info,
            
            commands::export::import_function_json,
            commands::export::export_function_json,
            commands::export::save_function_list,
            commands::export::save_type_list,
            commands::export::save_file_list,
            commands::export::save_stripped_bytecode,
            commands::export::save_disassembled_code,
            
            commands::config::init_config,
            commands::config::save_config,
            commands::config::set_config_theme,
            commands::config::set_config_colorscheme,
            commands::config::add_config_recent_file,
            commands::config::remove_config_recent_file,
            commands::config::get_config_theme,
            commands::config::get_config_colorscheme,
            commands::config::get_config_recent_files,
            commands::config::set_config_openrouter_key,
            commands::config::get_config_openrouter_key,
            commands::config::set_config_ai_decompiler,
            commands::config::get_config_ai_decompiler,
            commands::config::set_config_prompt,
            commands::config::get_config_prompt,
            
            commands::ai::save_ai_decompilation,
            commands::ai::get_ai_decompilation,
            commands::ai::get_ai_decompiled_functions,
            commands::ai::get_ai_decompilations,
            commands::ai::update_replaced_decompilations,
            commands::ai::remove_ai_decompilation,
            commands::ai::remove_all_decompilations,
            
            commands::history::add_history_item,
            commands::history::get_history_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}