mod structgen;
mod app_config;
mod ai_decomp;
mod app_data;

use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use hlbc::Bytecode;
use hlbc::Resolve;
use hlbc::fmt::EnhancedFmt;
use hlbc::types::Type;
use hlbc::opcodes::Opcode;
use hlbc_decompiler::{decompile_function, decompile_class};
use tauri::State;
use tauri::Manager;
use std::io::BufRead;
use std::collections::HashMap;
use chrono;
use std::fs;
use std::path::PathBuf;
use dirs;

use crate::app_config::AppConfig;
use crate::ai_decomp::AIDecompilation;
use crate::app_data::{AppData, Storage, AppItem, HistoryItem};

#[tauri::command]
fn set_target_file_path(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.target_file_path = file_path.to_string();

    let path = Path::new(&app_data.target_file_path);
    app_data.bytecode = Some(Bytecode::from_file(path).map_err(|e| e.to_string())?);

    Ok(())
}

#[tauri::command]
fn get_function_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    let mut function_names = Vec::new();
    let mut index = 0;
    for function in functions {
        function_names.push(function.name(&bytecode).to_string() + &function.findex.to_string() +
            "@" + &index.to_string());
        index += 1;
    }

    Ok(function_names)
}

#[tauri::command]
fn get_file_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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
fn get_type_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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

    Ok(type_names)
}

#[tauri::command]
fn get_string_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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
fn get_global_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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
fn get_native_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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
fn get_constant_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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
fn get_int_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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
fn get_float_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
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


#[tauri::command]
fn get_decompiled_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;

    match app_item.typ.as_str() {
        "function" => {
            let functions = &bytecode.functions;
            if index >= functions.len() {
                return Err("Function index out of bounds".to_string());
            }
            let function = &functions[index];
            let decompiled = decompile_function(&bytecode, &function);
            Ok(format!("{}", decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))))
        }
        "class" => {
            let types = &bytecode.types;
            if index >= types.len() {
                return Err("Type index out of bounds".to_string());
            }
            let type_obj = &types[index];
            match type_obj {
                Type::Obj(obj) => {
                    let decompiled = decompile_class(&bytecode, obj);
                    Ok(format!("{}", decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))))
                }
                _ => Err("Type is not an object".to_string()),
            }
        }
        _ => Err(format!("Unsupported item type: {}", app_item.typ)),
    }
}

#[tauri::command]
fn get_dashboard_info(app_data: State<Storage>) -> Result<String, String> {
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

#[allow(dead_code)]
#[tauri::command]
fn set_selected_item(app_item: AppItem, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.selected_item = Some(app_item);
    Ok(())
}

#[tauri::command]
fn get_inspector_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;
    let item_type = &app_item.typ;

    let info = match item_type.as_str() {
        "function" => {
            let functions = &bytecode.functions;
            if index >= functions.len() {
                return Err("Function index out of bounds".to_string());
            }
            let f = &functions[index];
            let mut info = format!("{}{}\n", 
                f.name.display::<EnhancedFmt>(&bytecode), 
                f.findex);
            
            let findex = f.findex;
            info.push_str("\n\nReferences:");
            
            functions
                .iter()
                .enumerate()
                .flat_map(|(i, f)| std::iter::repeat((i, f)).zip(f.find_fun_refs()))
                .for_each(|((_src_idx, f), (op_idx, op, fun))| {
                    if fun.0 == findex.0 {
                        info.push_str(&format!(
                            "\n{} at {}: {}",
                            f.display_header::<EnhancedFmt>(&bytecode),
                            op_idx,
                            op.name()
                        ));
                    }
                });
            
            info
        }
        "class" => {
            let types = &bytecode.types;
            if index >= types.len() {
                return Err("Type index out of bounds".to_string());
            }
            let type_obj = &types[index];
            let info = match type_obj {
                Type::Fun(fun) => format!("{}\n", 
                    type_obj.display::<EnhancedFmt>(&bytecode)),
                Type::Obj(obj) => format!("{}\n", 
                    type_obj.display::<EnhancedFmt>(&bytecode)),
                _ => format!("{}", type_obj.display::<EnhancedFmt>(&bytecode))
            };
            
            info
        }
        "file" => {
            let debug_files = bytecode.debug_files.as_ref().ok_or("debug_files not loaded")?;
            if index >= debug_files.len() {
                return Err("File index out of bounds".to_string());
            }
            let mut info = format!("File: {}\n\nFunctions:", debug_files[index]);
            
            for (_i, f) in bytecode.functions.iter().enumerate() {
                if let Some(debug_info) = f.debug_info.as_ref() {
                    if let Some(last_file_idx) = debug_info.last().map(|(file_idx, _)| file_idx) {
                        if *last_file_idx == index {
                            info.push_str(&format!(
                                "\n{}", 
                                f.display_header::<EnhancedFmt>(&bytecode)
                            ));
                        }
                    }
                }
            }
            
            info
        }
        "global" => {
            let globals = &bytecode.globals;
            if index >= globals.len() {
                return Err("Global index out of bounds".to_string());
            }
            let mut info = format!("{}", globals[index].display::<EnhancedFmt>(&bytecode));
            
            info.push_str("\n\nReferences:");
            
            if let Some(constants) = &bytecode.constants {
                for (i, c) in constants.iter().enumerate() {
                    if c.global.0 == index {
                        info.push_str(&format!("\nConstant@{}: {:?}", i, c));
                    }
                }
            }
            
            bytecode
                .functions
                .iter()
                .flat_map(|f| {
                    f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op))
                })
                .for_each(|(f, op_idx, op)| match op {
                    Opcode::GetGlobal { global, .. } | Opcode::SetGlobal { global, .. } => {
                        if global.0 == index {
                            info.push_str(&format!(
                                "\n{} at {}: {}",
                                f.display_header::<EnhancedFmt>(&bytecode),
                                op_idx,
                                op.name()
                            ));
                        }
                    }
                    _ => {}
                });
            
            info
        }
        "constant" => {
            let constants = bytecode.constants.as_ref().ok_or("constants not loaded")?;
            if index >= constants.len() {
                return Err("Constant index out of bounds".to_string());
            }
            format!("{:?}", constants[index])
        }
        "string" => {
            let strings = &bytecode.strings;
            if index >= strings.len() {
                return Err("String index out of bounds".to_string());
            }
            let mut info = format!("String@{}: {:?}", index, strings[index]);
            
            info.push_str("\n\nReferences:");
            
            bytecode
                .functions
                .iter()
                .flat_map(|f| {
                    f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op))
                })
                .for_each(|(f, op_idx, op)| match op {
                    Opcode::String { ptr, .. } => {
                        if ptr.0 == index {
                            info.push_str(&format!(
                                "\n{} at {}: {}",
                                f.display_header::<EnhancedFmt>(&bytecode),
                                op_idx,
                                op.name()
                            ));
                        }
                    }
                    _ => {}
                });
            
            info
        }
        "int" => {
            let ints = &bytecode.ints;
            if index >= ints.len() {
                return Err("Int index out of bounds".to_string());
            }
            let info = format!("Int@{}: {}", index, ints[index]);
            info
        }
        "float" => {
            let floats = &bytecode.floats;
            if index >= floats.len() {
                return Err("Float index out of bounds".to_string());
            }
            let info = format!("Float@{}: {}", index, floats[index]);
            info
        }
        "native" => {
            let natives = &bytecode.natives;
            if index >= natives.len() {
                return Err("Native index out of bounds".to_string());
            }
            let native = &natives[index];
            let info = format!("{}\n", 
                native.display::<EnhancedFmt>(&bytecode));
            
            info
        }
        _ => return Err(format!("Unsupported item type: {}", item_type)),
    };

    Ok(info)
}

#[tauri::command]
fn clear_references(app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.references = None;
    Ok(())
}

#[tauri::command]
fn get_disassembler_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;

    match app_item.typ.as_str() {
        "function" => {
            let functions = &bytecode.functions;
            if index >= functions.len() {
                return Err("Function index out of bounds".to_string());
            }
            let function = &functions[index];
            Ok(format!("{}", function.display::<EnhancedFmt>(&bytecode)))
        }
        "class" => {
            let types = &bytecode.types;
            if index >= types.len() {
                return Err("Type index out of bounds".to_string());
            }
            let type_obj = &types[index];
            match type_obj {
                Type::Obj(obj) => {
                    let mut disassembled_type = format!("{}", type_obj.display::<EnhancedFmt>(&bytecode));
                    if let Some(sup) = obj.super_ {
                        disassembled_type += &format!("\nextends {}", sup.display::<EnhancedFmt>(&bytecode));
                    }
                    disassembled_type += &format!("\nglobal: {}", obj.global.0);
                    disassembled_type += "\nfields:";
                    for f in &obj.own_fields {
                        disassembled_type += &format!("\n  {}: {}", f.name.display::<EnhancedFmt>(&bytecode), f.t.display::<EnhancedFmt>(&bytecode));
                    }
                    disassembled_type += "\nprotos:";
                    for p in &obj.protos {
                        disassembled_type += &format!("\n  {}: {} ({})", p.name.display::<EnhancedFmt>(&bytecode), bytecode.get(p.findex).display_header::<EnhancedFmt>(&bytecode), p.pindex);
                    }
                    disassembled_type += "\nbindings:";
                    for (fi, fun) in &obj.bindings {
                        disassembled_type += &format!("\n  {}: {}", fi.display::<EnhancedFmt>(&bytecode, type_obj), fun.display_header::<EnhancedFmt>(&bytecode));
                    }
                    Ok(disassembled_type)
                }
                Type::Enum { global, constructs, .. } => {
                    let mut disassembled_type = format!("global: {}", global.0);
                    disassembled_type += "\nconstructs:";
                    for c in constructs {
                        disassembled_type += &format!("\n  {}:", c.name(&bytecode));
                        for (i, p) in c.params.iter().enumerate() {
                            disassembled_type += &format!("\n    {i}: {}", p.display::<EnhancedFmt>(&bytecode));
                        }
                    }
                    Ok(disassembled_type)
                }
                _ => Err("Type is not an object or enum".to_string()),
            }
        }
        _ => Err(format!("Unsupported item type: {}", app_item.typ)),
    }
}

#[tauri::command]
fn read_binary_file(path: String) -> Result<Vec<u8>, String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
fn load_function_addresses_from_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut function_addresses = Vec::new();
    let file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        function_addresses.push(line.map_err(|e| e.to_string())?);
    }

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.function_addresses = Some(function_addresses);

    Ok(())
}

#[tauri::command]
fn get_function_addresses(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let function_addresses = app_data.function_addresses.as_ref().ok_or("function_addresses not loaded")?;
    Ok(function_addresses.clone())
}

#[tauri::command]
fn save_function_list(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
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
fn save_type_list(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
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
fn save_file_list(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
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
fn save_disassembled_code(file_path: &str, function_index: &str, app_data: State<Storage>) -> Result<(), String> {
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
    
    let disassembled_code = format!("{}", function.unwrap().display::<EnhancedFmt>(&bytecode));

    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    writeln!(file, "{}", disassembled_code).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn init_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    let config_dir = app_handle.path().config_dir().unwrap();
    let config_path = config_dir.to_str().unwrap();
    let config_file_path = format!("{}/bytesto4t.v2.json", config_path);

    if Path::new(&config_file_path).exists() {
        read_config(&config_file_path, &app_handle)?;
    } else {
        create_default_config(&config_file_path, &app_handle)?;
    }

    Ok(())
}

fn read_config(config_file_path: &str, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let config_file = std::fs::File::open(config_file_path).map_err(|e| e.to_string())?;
    let app_config: AppConfig = serde_json::from_reader(config_file).map_err(|e| e.to_string())?;
    let app_data = app_handle.state::<Storage>();
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config = app_config;
    Ok(())
}

fn create_default_config(config_file_path: &str, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let _ = std::fs::File::create(config_file_path).map_err(|e| e.to_string())?;
    let default_config = AppConfig {
        file_path: config_file_path.to_string(),
        theme: Some("dark".to_string()),
        colorscheme: Some("hamlindigo".to_string()),
        recent_files: Some(Vec::new()),
        openrouter_key: None,
        ai_decompiler: None,
        ai_prompt: None,
    };
    let default_config_str = serde_json::to_string(&default_config).map_err(|e| e.to_string())?;
    std::fs::write(config_file_path, default_config_str).map_err(|e| e.to_string())?;

    let app_data = app_handle.state::<Storage>().inner();
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config = default_config;

    Ok(())
}

#[tauri::command]
fn save_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    let config_dir = app_handle.path().config_dir().unwrap();
    let config_path = config_dir.to_str().unwrap();
    let config_file_path = format!("{}/bytesto4t.v2.json", config_path);
    let app_data = app_handle.state::<Storage>().inner();
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_config = &app_data.app_config;
    let app_config_str = serde_json::to_string(app_config).map_err(|e| e.to_string())?;
    std::fs::write(config_file_path, app_config_str).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_config_theme(theme: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.theme = Some(theme.to_string());

    Ok(())
}

#[tauri::command]
fn set_config_colorscheme(colorscheme: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.colorscheme = Some(colorscheme.to_string());

    Ok(())
}

#[tauri::command]
fn add_config_recent_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let recent_files = app_data.app_config.recent_files.get_or_insert_with(|| Vec::new());
    
    if !recent_files.contains(&file_path.to_string()) {
        recent_files.push(file_path.to_string());
    }

    Ok(())
}

#[tauri::command]
fn get_config_theme(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let theme = app_data.app_config.theme.as_ref().ok_or("theme not set")?;
    Ok(theme.to_string())
}

#[tauri::command]
fn get_config_colorscheme(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let colorscheme = app_data.app_config.colorscheme.as_ref().ok_or("colorscheme not set")?;
    Ok(colorscheme.to_string())
}

#[tauri::command]
fn get_config_recent_files(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let recent_files = app_data.app_config.recent_files.as_ref().ok_or("recent_files not set")?;
    Ok(recent_files.clone())
}

#[tauri::command]
fn get_target_file_path(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.target_file_path.clone())
}

#[tauri::command]
async fn add_history_item(
    app_data: State<'_, Storage>,
    item: HistoryItem,
) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let mut history = app_data.history_items.lock().map_err(|e| e.to_string())?;
    
    if history.is_empty() || history[0].name != item.name || history[0].typ != item.typ {
        history.insert(0, item);
    }
    
    Ok(())
}

#[tauri::command]
async fn get_history_items(
    app_data: State<'_, Storage>,
) -> Result<Vec<HistoryItem>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let history = app_data.history_items.lock().map_err(|e| e.to_string())?;
    Ok(history.clone())
}

#[tauri::command]
fn get_saved_references(app_data: State<Storage>) -> Result<Option<(usize, Vec<String>)>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.references.as_ref().map(|r| (r.element_index, r.references.clone())))
}

#[tauri::command]
fn generate_imhex_pattern(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;

    match app_item.typ.as_str() {
        "class" => {
            let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;
            if index >= bytecode.types.len() {
                return Err("Type index out of bounds".to_string());
            }
            
            Ok(structgen::generate_imhex_pattern(bytecode, index))
        },
        _ => Err(format!("Cannot generate ImHex pattern for item type: {}", app_item.typ))
    }
}

#[tauri::command]
fn set_config_openrouter_key(key: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.openrouter_key = Some(key.to_string());
    Ok(())
}

#[tauri::command]
fn get_config_openrouter_key(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.app_config.openrouter_key.clone().unwrap_or_default())
}

#[tauri::command]
fn set_config_ai_decompiler(model: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.ai_decompiler = Some(model.to_string());
    Ok(())
}

#[tauri::command]
fn get_config_ai_decompiler(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.app_config.ai_decompiler.clone().unwrap_or("deepseek/deepseek-r1:free".to_string()))
}

#[tauri::command]
fn set_config_prompt(prompt: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.ai_prompt = Some(prompt.to_string());
    Ok(())
}

#[tauri::command]
fn get_config_prompt(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.app_config.ai_prompt.clone().unwrap_or(
        "Decompile please following hashlink disassembly. Provide only decompiled code, no other words.".to_string()
    ))
}

#[tauri::command]
fn save_ai_decompilation(
    function_name: &str, 
    result: &str,
    model: &str,
    app_data: State<Storage>
) -> Result<(), String> {
    let decompilation = AIDecompilation {
        function_name: function_name.to_string(),
        result: result.to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
        model: model.to_string(),
    };

    let dir = get_decompilations_dir()?;
    let file_name = format!("{}.json", function_name.replace(['/', '\\', ':'], "_"));
    let file_path = dir.join(file_name);
    
    fs::write(
        file_path,
        serde_json::to_string_pretty(&decompilation)
            .map_err(|e| format!("Failed to serialize decompilation: {}", e))?
    ).map_err(|e| format!("Failed to write decompilation file: {}", e))?;

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.ai_decompilations = Some(load_decompilations()?);

    Ok(())
}

#[tauri::command]
fn get_ai_decompilation(function_name: &str, app_data: State<Storage>) -> Result<Option<AIDecompilation>, String> {
    let dir = get_decompilations_dir()?;
    let file_name = format!("{}.json", function_name.replace(['/', '\\', ':'], "_"));
    let file_path = dir.join(file_name);

    if !file_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read decompilation file: {}", e))?;
    let decompilation: AIDecompilation = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse decompilation file: {}", e))?;

    Ok(Some(decompilation))
}

#[tauri::command]
fn get_ai_decompilations(_app_data: State<Storage>) -> Result<Vec<AIDecompilation>, String> {
    let decompilations = load_decompilations()?;
    Ok(decompilations.into_values().collect())
}

#[tauri::command]
fn get_ai_decompiled_functions(_app_data: State<Storage>) -> Result<Vec<String>, String> {
    let decompilations = load_decompilations()?;
    Ok(decompilations.into_keys().collect())
}

#[tauri::command]
fn update_replaced_decompilations(app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.ai_decompilations = Some(load_decompilations()?);
    Ok(())
}

#[tauri::command]
fn remove_ai_decompilation(function_name: &str, app_data: State<Storage>) -> Result<(), String> {
    let dir = get_decompilations_dir()?;
    let file_name = format!("{}.json", function_name.replace(['/', '\\', ':'], "_"));
    let file_path = dir.join(file_name);
    
    if file_path.exists() {
        fs::remove_file(file_path)
            .map_err(|e| format!("Failed to remove decompilation file: {}", e))?;
    }

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.ai_decompilations = Some(load_decompilations()?);

    Ok(())
}

#[tauri::command]
fn remove_all_decompilations(app_data: State<Storage>) -> Result<(), String> {
    let dir = get_decompilations_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to remove decompilations directory: {}", e))?;
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to recreate decompilations directory: {}", e))?;
    }

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.ai_decompilations = Some(HashMap::new());

    Ok(())
}

fn load_decompilations() -> Result<HashMap<String, AIDecompilation>, String> {
    let mut decompilations = HashMap::new();
    let dir = get_decompilations_dir()?;

    if !dir.exists() {
        return Ok(decompilations);
    }

    for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read decompilations directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read decompilation file: {}", e))?;
            let decompilation: AIDecompilation = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse decompilation file: {}", e))?;
            decompilations.insert(decompilation.function_name.clone(), decompilation);
        }
    }

    Ok(decompilations)
}

fn get_decompilations_dir() -> Result<PathBuf, String> {
    let mut path = dirs::data_dir()
        .ok_or("Could not determine data directory")?;
    path.push("bytesto4t_decompilations");
    
    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create decompilations directory: {}", e))?;
    }
    
    Ok(path)
}

#[tauri::command]
fn get_selected_item(app_data: State<Storage>) -> Result<Option<AppItem>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.selected_item.clone())
}

#[tauri::command]
fn get_function_name_by_index(index: usize, app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    
    if index >= functions.len() {
        return Err("Function index out of bounds".to_string());
    }

    Ok(functions[index].name(&bytecode).to_string() + &functions[index].findex.to_string() +
    "@" + &index.to_string())
}

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
                #[allow(dead_code)]
                selected_item: None,
                function_addresses: None,
                history_items: Mutex::new(Vec::new()),
                references: None,
            }),
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            set_target_file_path,
            get_function_list,
            get_file_list,
            get_type_list,
            get_string_list,
            get_global_list,
            get_native_list,
            get_constant_list,
            get_int_list,
            get_float_list,
            get_decompiled_info,
            get_dashboard_info,
            set_selected_item,
            get_inspector_info,
            clear_references,
            get_disassembler_info,
            read_binary_file,
            load_function_addresses_from_file,
            get_function_addresses,
            save_function_list,
            save_type_list,
            save_file_list,
            save_disassembled_code,
            init_config,
            save_config,
            set_config_theme,
            set_config_colorscheme,
            add_config_recent_file,
            get_config_theme,
            get_config_colorscheme,
            get_config_recent_files,
            get_target_file_path,
            add_history_item,
            get_history_items,
            get_saved_references,
            generate_imhex_pattern,
            set_config_openrouter_key,
            get_config_openrouter_key,
            set_config_ai_decompiler,
            get_config_ai_decompiler,
            set_config_prompt,
            get_config_prompt,
            save_ai_decompilation,
            get_ai_decompilation,
            get_ai_decompiled_functions,
            get_ai_decompilations,
            update_replaced_decompilations,
            remove_ai_decompilation,
            remove_all_decompilations,
            get_selected_item,
            get_function_name_by_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}