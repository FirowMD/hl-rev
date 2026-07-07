use tauri::State;
use std::path::Path;
use hlbc::Bytecode;
use hlbc::fmt::EnhancedFmt;
use crate::app_data::{Storage, AppItem};

#[tauri::command]
pub fn set_target_file_path(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.target_file_path = file_path.to_string();

    let path = Path::new(&app_data.target_file_path);
    app_data.bytecode = Some(Bytecode::from_file(path).map_err(|e| e.to_string())?);

    Ok(())
}

#[tauri::command]
pub fn get_dashboard_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let version = bytecode.version;
    let int_n = bytecode.ints.len();
    let float_n = bytecode.floats.len();
    let string_n = bytecode.strings.len();
    let byte_n = bytecode.bytes.as_ref().map(|(bytes, _)| bytes.len()).unwrap_or(0);
    let file_n = bytecode.debug_files.as_ref().map(|files| files.len()).unwrap_or(0);
    let type_n = bytecode.types.len();
    let global_n = bytecode.globals.len();
    let native_n = bytecode.natives.len();
    let function_n = bytecode.functions.len();
    let constant_n = bytecode.constants.as_ref().map(|constants| constants.len()).unwrap_or(0);
    let entrypoint = bytecode.entrypoint.display::<EnhancedFmt>(&bytecode);

    Ok(format!(
        "File path: {}\nVersion: {}\nInts: {}\nFloats: {}\nStrings: {}\nBytes: {}\nFiles: {}\nTypes: {}\nGlobals: {}\nNatives: {}\nFunctions: {}\nConstants: {}\nEntrypoint: {}",
        app_data.target_file_path,
        version,
        int_n,
        float_n,
        string_n,
        byte_n,
        file_n,
        type_n,
        global_n,
        native_n,
        function_n,
        constant_n,
        entrypoint
    ))
}

#[tauri::command]
pub fn set_selected_item(app_item: AppItem, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    // Defensive: make sure function index is valid if type=="function"
    if app_item.typ == "function" {
        let idx = app_item.index.parse::<usize>().unwrap_or(usize::MAX);
        if let Some(bc) = app_data.bytecode.as_ref() {
            if idx >= bc.functions.len() {
                return Err(format!("Tried to select non-existent function index {} (max is {})", idx, bc.functions.len().saturating_sub(1)));
            }
        }
    }
    app_data.selected_item = Some(app_item);
    Ok(())
}

#[tauri::command]
pub fn get_selected_item(app_data: State<Storage>) -> Result<Option<AppItem>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.selected_item.clone())
}

#[tauri::command]
pub fn get_target_file_path(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.target_file_path.clone())
}

#[tauri::command]
pub fn clear_references(app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.references = None;
    Ok(())
}

#[tauri::command]
pub fn get_saved_references(app_data: State<Storage>) -> Result<Option<(usize, Vec<String>)>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.references.as_ref().map(|r| (r.element_index, r.references.clone())))
}

#[tauri::command]
pub fn read_binary_file(path: String) -> Result<Vec<u8>, String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
pub fn merge_bytecode_with_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let current = app_data.bytecode.take().ok_or("bytecode not loaded")?;
    let other = Bytecode::from_file(Path::new(file_path)).map_err(|e| e.to_string())?;
    let merged = current.merge_with(other).map_err(|e| e.to_string())?;
    app_data.bytecode = Some(merged);
    Ok(())
}
