use crate::app_config::AppConfig;
use crate::app_data::Storage;
use std::path::Path;
use tauri::{Manager, Runtime, State};

fn write_config_file(config_file_path: &str, app_config: &AppConfig) -> Result<(), String> {
    let app_config_str = serde_json::to_string(app_config).map_err(|e| e.to_string())?;
    std::fs::write(config_file_path, app_config_str).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn init_config<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Result<(), String> {
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

fn read_config<R: Runtime>(
    config_file_path: &str,
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    let config_file = std::fs::File::open(config_file_path).map_err(|e| e.to_string())?;
    let app_config: AppConfig = serde_json::from_reader(config_file).map_err(|e| e.to_string())?;

    let app_data = app_handle.state::<Storage>();
    let mut config = app_data.config.lock().map_err(|e| e.to_string())?;
    *config = app_config;
    Ok(())
}

fn create_default_config<R: Runtime>(
    config_file_path: &str,
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    let _ = std::fs::File::create(config_file_path).map_err(|e| e.to_string())?;
    let default_config = AppConfig {
        file_path: config_file_path.to_string(),
        theme: Some("dark".to_string()),
        colorscheme: Some("bytesto4t".to_string()),
        recent_files: Some(Vec::new()),
    };
    write_config_file(config_file_path, &default_config)?;

    let app_data = app_handle.state::<Storage>().inner();
    let mut config = app_data.config.lock().map_err(|e| e.to_string())?;
    *config = default_config;

    Ok(())
}

#[tauri::command]
pub fn save_config<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Result<(), String> {
    let config_dir = app_handle.path().config_dir().unwrap();
    let config_path = config_dir.to_str().unwrap();
    let config_file_path = format!("{}/bytesto4t.v2.json", config_path);
    let app_data = app_handle.state::<Storage>().inner();
    let app_config = app_data.config.lock().map_err(|e| e.to_string())?;
    write_config_file(&config_file_path, &app_config)?;

    Ok(())
}

#[tauri::command]
pub fn set_config_theme(theme: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut config = app_data.config.lock().map_err(|e| e.to_string())?;
    config.theme = Some(theme.to_string());
    Ok(())
}

#[tauri::command]
pub fn set_config_colorscheme(colorscheme: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut config = app_data.config.lock().map_err(|e| e.to_string())?;
    config.colorscheme = Some(colorscheme.to_string());
    Ok(())
}

#[tauri::command]
pub fn add_config_recent_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut config = app_data.config.lock().map_err(|e| e.to_string())?;
    let recent_files = config.recent_files.get_or_insert_with(Vec::new);

    if !recent_files.contains(&file_path.to_string()) {
        recent_files.push(file_path.to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn get_config_theme(app_data: State<Storage>) -> Result<String, String> {
    let config = app_data.config.lock().map_err(|e| e.to_string())?;
    let theme = config.theme.as_ref().ok_or("theme not set")?;
    Ok(theme.to_string())
}

#[tauri::command]
pub fn get_config_colorscheme(app_data: State<Storage>) -> Result<String, String> {
    let config = app_data.config.lock().map_err(|e| e.to_string())?;
    let colorscheme = config.colorscheme.as_ref().ok_or("colorscheme not set")?;
    Ok(colorscheme.to_string())
}

#[tauri::command]
pub fn get_config_recent_files(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let config = app_data.config.lock().map_err(|e| e.to_string())?;
    let recent_files = config.recent_files.as_ref().ok_or("recent_files not set")?;
    Ok(recent_files.clone())
}

#[tauri::command]
pub fn remove_config_recent_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut config = app_data.config.lock().map_err(|e| e.to_string())?;
    if let Some(recent_files) = &mut config.recent_files {
        recent_files.retain(|file| file != file_path);
    }
    Ok(())
}
