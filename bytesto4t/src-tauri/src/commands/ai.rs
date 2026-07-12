use crate::ai_decomp::AIDecompilation;
use crate::app_data::Storage;
use crate::commands::config::get_secure_openrouter_key;
use chrono;
use dirs;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn decompile_with_openrouter(
    model: String,
    prompt: String,
    disassembly: String,
) -> Result<String, String> {
    if model.trim().is_empty() {
        return Err("AI model is not configured".to_string());
    }
    if disassembly.trim().is_empty() {
        return Err("No disassembly is available to decompile".to_string());
    }

    let api_key = get_secure_openrouter_key()?
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| "OpenRouter API key is not configured".to_string())?;

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": format!("{}\n\n{}", prompt, disassembly),
            }
        ],
    });

    let response = reqwest::Client::new()
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .header("HTTP-Referer", "app://bytesto4t")
        .header("X-Title", "ByteSto4t")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("OpenRouter request failed: {e}"))?;

    let status = response.status();
    let data = response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse OpenRouter response: {e}"))?;

    if !status.is_success() {
        let message = data
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("OpenRouter returned an error");
        return Err(format!(
            "OpenRouter request failed with {status}: {message}"
        ));
    }

    data.get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Invalid OpenRouter response format: {data}"))
}

#[tauri::command]
pub fn save_ai_decompilation(
    function_name: &str,
    result: &str,
    model: &str,
    app_data: State<Storage>,
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
            .map_err(|e| format!("Failed to serialize decompilation: {}", e))?,
    )
    .map_err(|e| format!("Failed to write decompilation file: {}", e))?;

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.ai_decompilations = Some(load_decompilations()?);

    Ok(())
}

#[tauri::command]
pub fn get_ai_decompilation(
    function_name: &str,
    _app_data: State<Storage>,
) -> Result<Option<AIDecompilation>, String> {
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
pub fn get_ai_decompilations(_app_data: State<Storage>) -> Result<Vec<AIDecompilation>, String> {
    let decompilations = load_decompilations()?;
    Ok(decompilations.into_values().collect())
}

#[tauri::command]
pub fn get_ai_decompiled_functions(_app_data: State<Storage>) -> Result<Vec<String>, String> {
    let decompilations = load_decompilations()?;
    Ok(decompilations.into_keys().collect())
}

#[tauri::command]
pub fn update_replaced_decompilations(app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.ai_decompilations = Some(load_decompilations()?);
    Ok(())
}

#[tauri::command]
pub fn remove_ai_decompilation(
    function_name: &str,
    app_data: State<Storage>,
) -> Result<(), String> {
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
pub fn remove_all_decompilations(app_data: State<Storage>) -> Result<(), String> {
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

    for entry in
        fs::read_dir(dir).map_err(|e| format!("Failed to read decompilations directory: {}", e))?
    {
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
    let mut path = dirs::data_dir().ok_or("Could not determine data directory")?;
    path.push("bytesto4t_decompilations");

    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create decompilations directory: {}", e))?;
    }

    Ok(path)
}
