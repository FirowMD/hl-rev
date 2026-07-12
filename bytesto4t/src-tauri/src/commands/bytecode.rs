use crate::app_data::{AppItem, Storage};
use hlbc::fmt::EnhancedFmt;
use hlbc::Bytecode;
use std::path::Path;
use tauri::State;

#[derive(serde::Serialize)]
pub struct TargetFileInfo {
    pub name: String,
    pub size: u64,
}

#[tauri::command]
pub fn set_target_file_path(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let path = Path::new(file_path);
    let bytecode = Bytecode::from_file(path).map_err(|e| e.to_string())?;

    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    bytecode_store.target_file_path = file_path.to_string();
    bytecode_store.bytecode = Some(bytecode);

    Ok(())
}

#[tauri::command]
pub fn get_dashboard_info(app_data: State<Storage>) -> Result<String, String> {
    let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_ref()
        .ok_or("bytecode not loaded")?;
    let version = bytecode.version;
    let int_n = bytecode.ints.len();
    let float_n = bytecode.floats.len();
    let string_n = bytecode.strings.len();
    let byte_n = bytecode
        .bytes
        .as_ref()
        .map(|(bytes, _)| bytes.len())
        .unwrap_or(0);
    let file_n = bytecode
        .debug_files
        .as_ref()
        .map(|files| files.len())
        .unwrap_or(0);
    let type_n = bytecode.types.len();
    let global_n = bytecode.globals.len();
    let native_n = bytecode.natives.len();
    let function_n = bytecode.functions.len();
    let constant_n = bytecode
        .constants
        .as_ref()
        .map(|constants| constants.len())
        .unwrap_or(0);
    let entrypoint = bytecode.entrypoint.display::<EnhancedFmt>(&bytecode);

    Ok(format!(
        "File path: {}\nVersion: {}\nInts: {}\nFloats: {}\nStrings: {}\nBytes: {}\nFiles: {}\nTypes: {}\nGlobals: {}\nNatives: {}\nFunctions: {}\nConstants: {}\nEntrypoint: {}",
        bytecode_store.target_file_path,
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
    // Defensive: make sure function index is valid if type=="function"
    if app_item.typ == "function" {
        let idx = app_item.index.parse::<usize>().unwrap_or(usize::MAX);
        let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
        if let Some(bc) = bytecode_store.bytecode.as_ref() {
            if idx >= bc.functions.len() {
                return Err(format!(
                    "Tried to select non-existent function index {} (max is {})",
                    idx,
                    bc.functions.len().saturating_sub(1)
                ));
            }
        }
    }
    let mut ui = app_data.ui.lock().map_err(|e| e.to_string())?;
    ui.selected_item = Some(app_item);
    Ok(())
}

#[tauri::command]
pub fn get_selected_item(app_data: State<Storage>) -> Result<Option<AppItem>, String> {
    let ui = app_data.ui.lock().map_err(|e| e.to_string())?;
    Ok(ui.selected_item.clone())
}

#[tauri::command]
pub fn get_target_file_info(app_data: State<Storage>) -> Result<TargetFileInfo, String> {
    let target_file_path = {
        let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
        bytecode_store
            .bytecode
            .as_ref()
            .ok_or("bytecode not loaded")?;
        bytecode_store.target_file_path.clone()
    };

    let path = Path::new(&target_file_path);
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(TargetFileInfo {
        name,
        size: metadata.len(),
    })
}

#[tauri::command]
pub fn clear_references(app_data: State<Storage>) -> Result<(), String> {
    let mut references = app_data.references.lock().map_err(|e| e.to_string())?;
    *references = None;
    Ok(())
}

#[tauri::command]
pub fn get_saved_references(
    app_data: State<Storage>,
) -> Result<Option<(usize, Vec<String>)>, String> {
    let references = app_data.references.lock().map_err(|e| e.to_string())?;
    Ok(references
        .as_ref()
        .map(|r| (r.element_index, r.references.clone())))
}

#[tauri::command]
pub fn merge_bytecode_with_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let current = bytecode_store
        .bytecode
        .take()
        .ok_or("bytecode not loaded")?;
    let other = Bytecode::from_file(Path::new(file_path)).map_err(|e| e.to_string())?;
    let merged = current.merge_with(other).map_err(|e| e.to_string())?;
    bytecode_store.bytecode = Some(merged);
    Ok(())
}
