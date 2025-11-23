use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::{Storage, AppItem};

#[derive(Clone)]
pub struct GetSelectedItemHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetSelectedItemHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let selected = app_data.selected_item.clone();
        let json_str = serde_json::to_string(&selected).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(CallToolResult::text(json_str))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_selected_item".to_string(),
            Some("Get selected item".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetSelectedItemHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}