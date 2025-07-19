use tauri::{State, Manager};
use std::path::Path;
use crate::app_config::AppConfig;
use crate::app_data::Storage;

#[tauri::command]
pub fn init_config(app_handle: tauri::AppHandle) -> Result<(), String> {
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
pub fn save_config(app_handle: tauri::AppHandle) -> Result<(), String> {
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
pub fn set_config_theme(theme: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.theme = Some(theme.to_string());
    Ok(())
}

#[tauri::command]
pub fn set_config_colorscheme(colorscheme: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.colorscheme = Some(colorscheme.to_string());
    Ok(())
}

#[tauri::command]
pub fn add_config_recent_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let recent_files = app_data.app_config.recent_files.get_or_insert_with(|| Vec::new());
    
    if !recent_files.contains(&file_path.to_string()) {
        recent_files.push(file_path.to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn get_config_theme(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let theme = app_data.app_config.theme.as_ref().ok_or("theme not set")?;
    Ok(theme.to_string())
}

#[tauri::command]
pub fn get_config_colorscheme(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let colorscheme = app_data.app_config.colorscheme.as_ref().ok_or("colorscheme not set")?;
    Ok(colorscheme.to_string())
}

#[tauri::command]
pub fn get_config_recent_files(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let recent_files = app_data.app_config.recent_files.as_ref().ok_or("recent_files not set")?;
    Ok(recent_files.clone())
}

#[tauri::command]
pub fn set_config_openrouter_key(key: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.openrouter_key = Some(key.to_string());
    Ok(())
}

#[tauri::command]
pub fn get_config_openrouter_key(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.app_config.openrouter_key.clone().unwrap_or_default())
}

#[tauri::command]
pub fn set_config_ai_decompiler(model: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.ai_decompiler = Some(model.to_string());
    Ok(())
}

#[tauri::command]
pub fn get_config_ai_decompiler(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.app_config.ai_decompiler.clone().unwrap_or("deepseek/deepseek-r1:free".to_string()))
}

#[tauri::command]
pub fn set_config_prompt(prompt: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.app_config.ai_prompt = Some(prompt.to_string());
    Ok(())
}

#[tauri::command]
pub fn get_config_prompt(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(app_data.app_config.ai_prompt.clone().unwrap_or(
r#"### ROLE ###
Act as a disassembler and decompiler for HashLink bytecode.

### CONTEXT ###
The provided input is a disassembly of HashLink bytecode.

### TASK ###
Decompile the given HashLink disassembly into high-level code.

### INSTRUCTIONS ###
Focus on translating the bytecode into equivalent code, likely in a language like Haxe or a similar high-level language.

### CONSTRAINTS ###
- Provide only the decompiled code, omitting any additional explanations or metadata.
- Ensure the output is syntactically correct and readable.

### OUTPUT FORMAT ###
The decompiled code in a high-level language.

```haxe
function charAt(index: Int): String {
    if (index < this.length)
        return String.fromCharCode(this.bytes.getUInt16(index << 1));
    else
        return global894;
}

### NEED TO DECOMPILE ###
```"#.to_string()
    ))
}