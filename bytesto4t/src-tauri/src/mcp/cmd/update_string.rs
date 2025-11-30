use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct UpdateStringHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateStringInput {
    pub index: usize,
    pub value: String,
}

#[async_trait]
impl ToolHandler for UpdateStringHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: UpdateStringInput = serde_json::from_value(serde_json::to_value(arguments)
            .map_err(|e| McpError::Validation(e.to_string()))?)
            .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if input.index >= bytecode.strings.len() {
            return Err(McpError::Validation(format!("String index {} out of bounds", input.index)));
        }

        bytecode.strings[input.index] = hlbc::Str::from(input.value);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "update_string".to_string(),
            Some("Update an existing string".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" }, "value": { "type": "string" } },
                "required": ["index", "value"],
                "additionalProperties": false
            }),
            UpdateStringHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}