use tauri::{ipc::Invoke, Runtime};

pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
    let bytecode = bytecode::handler::<R>();
    let functions = functions::handler::<R>();
    let types = types::handler::<R>();
    let data = data::handler::<R>();
    let decompiler = decompiler::handler::<R>();
    let export = export::handler::<R>();
    let config = config::handler::<R>();
    let history = history::handler::<R>();

    move |invoke| {
        let command = invoke.message.command().to_string();
        match command.as_str() {
            command if bytecode::handles(command) => bytecode(invoke),
            command if functions::handles(command) => functions(invoke),
            command if types::handles(command) => types(invoke),
            command if data::handles(command) => data(invoke),
            command if decompiler::handles(command) => decompiler(invoke),
            command if export::handles(command) => export(invoke),
            command if config::handles(command) => config(invoke),
            command if history::handles(command) => history(invoke),
            _ => false,
        }
    }
}

mod bytecode {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "set_target_file_path"
                | "get_dashboard_info"
                | "set_selected_item"
                | "get_selected_item"
                | "get_target_file_info"
                | "clear_references"
                | "get_saved_references"
                | "merge_bytecode_with_file"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::bytecode::set_target_file_path,
            crate::commands::bytecode::get_dashboard_info,
            crate::commands::bytecode::set_selected_item,
            crate::commands::bytecode::get_selected_item,
            crate::commands::bytecode::get_target_file_info,
            crate::commands::bytecode::clear_references,
            crate::commands::bytecode::get_saved_references,
            crate::commands::bytecode::merge_bytecode_with_file,
        ]
    }
}

mod functions {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "get_function_list"
                | "list_functions_with_constructors"
                | "create_function"
                | "delete_function"
                | "update_function"
                | "get_function_full_info"
                | "get_function_name_by_index"
                | "load_function_addresses_from_file"
                | "get_function_addresses"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::functions::get_function_list,
            crate::commands::functions::list_functions_with_constructors,
            crate::commands::functions::create_function,
            crate::commands::functions::delete_function,
            crate::commands::functions::update_function,
            crate::commands::functions::get_function_full_info,
            crate::commands::functions::get_function_name_by_index,
            crate::commands::functions::load_function_addresses_from_file,
            crate::commands::functions::get_function_addresses,
        ]
    }
}

mod types {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "get_type_list"
                | "create_type"
                | "update_type"
                | "delete_type"
                | "get_type_full_info"
                | "import_type_json"
                | "export_type_json"
                | "generate_imhex_pattern"
                | "find_functions_using_type_cmd"
                | "create_global"
                | "update_global"
                | "delete_global"
                | "get_global_full_info"
                | "create_native"
                | "update_native"
                | "delete_native"
                | "get_native_full_info"
                | "create_constant"
                | "update_constant"
                | "delete_constant"
                | "get_constant_full_info"
                | "create_string"
                | "update_string"
                | "delete_string"
                | "get_string_full_info"
                | "create_int"
                | "update_int"
                | "delete_int"
                | "get_int_full_info"
                | "create_float"
                | "update_float"
                | "delete_float"
                | "get_float_full_info"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::types::get_type_list,
            crate::commands::types::create_type,
            crate::commands::types::update_type,
            crate::commands::types::delete_type,
            crate::commands::types::get_type_full_info,
            crate::commands::types::import_type_json,
            crate::commands::types::export_type_json,
            crate::commands::types::generate_imhex_pattern,
            crate::commands::types::find_functions_using_type_cmd,
            crate::commands::types::create_global,
            crate::commands::types::update_global,
            crate::commands::types::delete_global,
            crate::commands::types::get_global_full_info,
            crate::commands::types::create_native,
            crate::commands::types::update_native,
            crate::commands::types::delete_native,
            crate::commands::types::get_native_full_info,
            crate::commands::types::create_constant,
            crate::commands::types::update_constant,
            crate::commands::types::delete_constant,
            crate::commands::types::get_constant_full_info,
            crate::commands::types::create_string,
            crate::commands::types::update_string,
            crate::commands::types::delete_string,
            crate::commands::types::get_string_full_info,
            crate::commands::types::create_int,
            crate::commands::types::update_int,
            crate::commands::types::delete_int,
            crate::commands::types::get_int_full_info,
            crate::commands::types::create_float,
            crate::commands::types::update_float,
            crate::commands::types::delete_float,
            crate::commands::types::get_float_full_info,
        ]
    }
}

mod data {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "get_file_list"
                | "get_string_list"
                | "get_global_list"
                | "get_native_list"
                | "get_constant_list"
                | "get_int_list"
                | "get_float_list"
                | "get_bytes_list"
                | "get_bytes_full_info"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::data::get_file_list,
            crate::commands::data::get_string_list,
            crate::commands::data::get_global_list,
            crate::commands::data::get_native_list,
            crate::commands::data::get_constant_list,
            crate::commands::data::get_int_list,
            crate::commands::data::get_float_list,
            crate::commands::data::get_bytes_list,
            crate::commands::data::get_bytes_full_info,
        ]
    }
}

mod decompiler {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "get_decompiled_info" | "get_inspector_info" | "get_disassembler_info"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::decompiler::get_decompiled_info,
            crate::commands::decompiler::get_inspector_info,
            crate::commands::decompiler::get_disassembler_info,
        ]
    }
}

mod export {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "import_function_json"
                | "export_function_json"
                | "save_function_list"
                | "save_type_list"
                | "save_file_list"
                | "save_stripped_bytecode"
                | "save_disassembled_code"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::export::import_function_json,
            crate::commands::export::export_function_json,
            crate::commands::export::save_function_list,
            crate::commands::export::save_type_list,
            crate::commands::export::save_file_list,
            crate::commands::export::save_stripped_bytecode,
            crate::commands::export::save_disassembled_code,
        ]
    }
}

mod config {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(
            command,
            "init_config"
                | "save_config"
                | "set_config_theme"
                | "set_config_colorscheme"
                | "add_config_recent_file"
                | "remove_config_recent_file"
                | "get_config_theme"
                | "get_config_colorscheme"
                | "get_config_recent_files"
        )
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::config::init_config,
            crate::commands::config::save_config,
            crate::commands::config::set_config_theme,
            crate::commands::config::set_config_colorscheme,
            crate::commands::config::add_config_recent_file,
            crate::commands::config::remove_config_recent_file,
            crate::commands::config::get_config_theme,
            crate::commands::config::get_config_colorscheme,
            crate::commands::config::get_config_recent_files,
        ]
    }
}

mod history {
    use super::*;

    pub fn handles(command: &str) -> bool {
        matches!(command, "add_history_item" | "get_history_items")
    }

    pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
        tauri::generate_handler![
            crate::commands::history::add_history_item,
            crate::commands::history::get_history_items,
        ]
    }
}
