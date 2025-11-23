use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::{Storage, AppItem};

#[derive(Clone)]
pub struct SetSelectedItemHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for SetSelectedItemHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let app_item: AppItem = serde_json::from_value(serde_json::to_value(arguments)
            .map_err(|e| McpError::Validation(e.to_string()))?)
            .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        if app_item.typ == "function" {
            let idx = app_item.index.parse::<usize>().map_err(|_| McpError::Validation("Invalid function index".to_string()))?;
            if let Some(bc) = app_data.bytecode.as_ref() {
                if idx >= bc.functions.len() {
                    return Err(McpError::Validation(format!("Function index out of bounds: {}", idx)));
                }
            }
        }
        app_data.selected_item = Some(app_item);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "set_selected_item".to_string(),
            Some("Set selected item".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": { "type": "string" },
                    "typ": { "type": "string" }
                },
                "required": ["index", "typ"],
                "additionalProperties": false
            }),
            SetSelectedItemHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}