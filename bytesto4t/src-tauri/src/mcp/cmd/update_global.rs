use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::RefType;

#[derive(Clone)]
pub struct UpdateGlobalHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateGlobalInput {
    pub index: usize,
    pub global_type: usize,
}

#[async_trait]
impl ToolHandler for UpdateGlobalHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: UpdateGlobalInput = serde_json::from_value(serde_json::to_value(arguments)
            .map_err(|e| McpError::Validation(e.to_string()))?)
            .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if input.index >= bytecode.globals.len() {
            return Err(McpError::Validation(format!("Global index {} out of bounds", input.index)));
        }
        if input.global_type >= bytecode.types.len() {
            return Err(McpError::Validation(format!("Type index {} out of bounds", input.global_type)));
        }

        bytecode.globals[input.index] = RefType(input.global_type);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "update_global".to_string(),
            Some("Update an existing global".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" }, "global_type": { "type": "integer" } },
                "required": ["index", "global_type"],
                "additionalProperties": false
            }),
            UpdateGlobalHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}