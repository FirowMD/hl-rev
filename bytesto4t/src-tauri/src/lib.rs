use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use hlbc::Bytecode;
use hlbc::Resolve;
use hlbc::fmt::EnhancedFmt;
use hlbc::types::Type;
use hlbc_decompiler::{decompile_function, decompile_class};
use tauri::State;
use tauri::Manager;

#[derive(Serialize, Deserialize)]
struct AppConfig {
    file_path: String,
    theme: Option<String>,
    colorscheme: Option<String>,
    recent_files: Option<Vec<String>>,
}

struct AppData {
    target_file_path: String,
    bytecode: Option<Bytecode>,
    app_config: AppConfig,
}

struct Storage {
    app_data: Mutex<AppData>,
}

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
fn get_decompiled_code(function_index: &str, app_data: State<Storage>) -> Result<String, String> {
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

    let mut decompiled_code = String::new();

    if let Some(f) = function {
        decompiled_code = format!("{}", decompile_function(&bytecode, &f)
            .display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2)));
    }
    
    Ok(decompiled_code)
}

#[tauri::command]
fn get_decompiled_type(type_index: &str, app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let types = &bytecode.types;

    let tindex: usize = type_index[1..].parse().map_err(|_| "Invalid type index format")?;
    if tindex >= types.len() {
        return Err("Type index out of bounds".to_string());
    }

    let type_obj = types[tindex].clone();
    let decompiled_code = match type_obj {
        Type::Obj(obj) => {
            format!("{}", decompile_class(&bytecode, &obj)
                .display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2)))
        }
        _ => return Err("Type is not an object".to_string()),
    };

    Ok(decompiled_code)
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

#[tauri::command]
fn get_disassembled_code(function_index: &str, app_data: State<Storage>) -> Result<String, String> {
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
    
    Ok(disassembled_code)
}

#[tauri::command]
fn get_disassembled_type(type_index: &str, app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let types = &bytecode.types;

    let tindex: usize = type_index[1..].parse().map_err(|_| "Invalid type index format")?;
    if tindex >= types.len() {
        return Err("Type index out of bounds".to_string());
    }

    let type_obj = &types[tindex];
    let disassembled_type = match type_obj {
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
            disassembled_type
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
            disassembled_type
        }
        _ => return Err("Type is not an object".to_string()),
    };

    Ok(disassembled_type)
}

#[tauri::command]
async fn read_binary_file(path: String) -> Result<Vec<u8>, String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e.to_string())
    }
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
    let config_file_path = format!("{}/bytesto4t.json", config_path);

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
        colorscheme: Some("crimson".to_string()),
        recent_files: Some(Vec::new()),
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
    let config_file_path = format!("{}/bytesto4t.json", config_path);
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
                },
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
            get_decompiled_code,
            get_decompiled_type,
            get_dashboard_info,
            get_disassembled_code,
            get_disassembled_type,
            read_binary_file,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
