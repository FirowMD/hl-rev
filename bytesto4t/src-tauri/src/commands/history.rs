use tauri::State;
use crate::app_data::{Storage, HistoryItem};

#[tauri::command]
pub async fn add_history_item(
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
pub async fn get_history_items(
    app_data: State<'_, Storage>,
) -> Result<Vec<HistoryItem>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let history = app_data.history_items.lock().map_err(|e| e.to_string())?;
    Ok(history.clone())
}