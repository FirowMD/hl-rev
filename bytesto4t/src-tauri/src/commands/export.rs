use tauri::State;
use hlbc::types::Function;
use hlbc::fmt::EnhancedFmt;
use std::io::{Write, BufRead};
use std::fs;
use crate::app_data::Storage;

#[tauri::command]
pub fn import_function_json(json_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    let json_file = std::fs::File::open(json_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(json_file);
    let json_content: String = reader.lines()
        .map(|line| line.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    let function = Function::from_json(json_content.as_str()).map_err(|e| e.to_string())?;
    bytecode.add_function(function);
    Ok(())
}

#[tauri::command]
pub fn export_function_json(function_index: &str, file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;

    let mut function = None;
    for f in functions {
        if f.findex.to_string() == function_index {
            function = Some(f);
            break;
        }
    }

    if function.is_none() {
        return Err(format!("Function with index {} not found", function_index));
    }

    let json_content = function.unwrap().to_json().map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    writeln!(file, "{}", json_content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_function_list(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    let mut function_names = Vec::new();
    for function in functions {
        function_names.push(function.name(&bytecode).to_string() + &function.findex.to_string());
    }

    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    for name in function_names {
        writeln!(file, "{}", name).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn save_type_list(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let types = &bytecode.types;
    let mut type_names = Vec::new();
    let mut index = 0;
    for t in types {
        type_names.push(t.display::<EnhancedFmt>(&bytecode).to_string() +
            "@" + &index.to_string());
        index += 1;
    }

    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    for name in type_names {
        writeln!(file, "{}", name).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn save_file_list(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
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

    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    for name in file_names {
        writeln!(file, "{}", name).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn save_stripped_bytecode(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let mut stripped_code = bytecode.clone();
    stripped_code.strip();

    let mut file = fs::File::create(file_path).map_err(|e| e.to_string())?;
    stripped_code.serialize(&mut file).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_disassembled_code(file_path: &str, function_index: &str, app_data: State<Storage>) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    
    let mut function = None;
    for f in functions {
        if f.findex.to_string() == function_index {
            function = Some(f);
            break;
        }
    }

    let function = match function {
        Some(f) => f,
        None => return Err(format!("Function with index {} not found", function_index)),
    };
    let disassembled_code = format!("{}", function.display::<EnhancedFmt>(&bytecode));

    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    writeln!(file, "{}", disassembled_code).map_err(|e| e.to_string())?;

    Ok(())
}