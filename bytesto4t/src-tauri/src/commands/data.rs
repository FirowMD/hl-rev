use tauri::State;
use hlbc::fmt::EnhancedFmt;
use crate::app_data::Storage;

#[tauri::command]
pub fn get_file_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let debug_files = bytecode
        .debug_files
        .as_ref()
        .ok_or("debug_files not loaded")?;
    let mut file_names = Vec::new();
    let mut index = 0;
    for file in debug_files {
        file_names.push(file.to_string() + "@" + &index.to_string());
        index += 1;
    }
    Ok(file_names)
}

#[tauri::command]
pub fn get_string_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let strings = &bytecode.strings;
    let mut string_list = Vec::new();
    let mut index = 0;
    for s in strings {
        string_list.push(s.to_string() + "@" + &index.to_string());
        index += 1;
    }
    Ok(string_list)
}

#[tauri::command]
pub fn get_global_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let globals = &bytecode.globals;
    let mut global_list = Vec::new();
    let mut index = 0;
    for g in globals {
        global_list.push(g.display::<EnhancedFmt>(&bytecode).to_string() +
            "@" + &index.to_string());
        index += 1;
    }
    Ok(global_list)
}

#[tauri::command]
pub fn get_native_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let natives = &bytecode.natives;
    let mut native_list = Vec::new();
    let mut index = 0;
    for n in natives {
        native_list.push(n.display::<EnhancedFmt>(&bytecode).to_string() +
            "@" + &index.to_string());
        index += 1;
    }
    Ok(native_list)
}

#[tauri::command]
pub fn get_constant_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let constants = bytecode.constants.as_ref().ok_or("constants not loaded")?;
    let mut constant_list = Vec::new();
    let mut index = 0;
    for c in constants {
        constant_list.push(format!("{:?}@{}", c, index));
        index += 1;
    }
    Ok(constant_list)
}

#[tauri::command]
pub fn get_int_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let ints = &bytecode.ints;
    let mut int_list = Vec::new();
    let mut index = 0;
    for i in ints {
        int_list.push(format!("{:?}@{}", i, index));
        index += 1;
    }
    Ok(int_list)
}

#[tauri::command]
pub fn get_float_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let floats = &bytecode.floats;
    let mut float_list = Vec::new();
    let mut index = 0;
    for f in floats {
        float_list.push(format!("{:?}@{}", f, index));
        index += 1;
    }
    Ok(float_list)
}