use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::{ConstantDef, RefGlobal};

#[derive(Clone)]
pub struct UpdateConstantHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateConstantInput {
    pub index: usize,
    pub global: usize,
    pub fields: Vec<usize>,
}

#[async_trait]
impl ToolHandler for UpdateConstantHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: UpdateConstantInput = serde_json::from_value(serde_json::to_value(arguments)
            .map_err(|e| McpError::Validation(e.to_string()))?)
            .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let constants = bytecode.constants.as_mut().ok_or_else(|| McpError::Validation("No constants defined".to_string()))?;
        if input.index >= constants.len() {
            return Err(McpError::Validation(format!("Constant index {} out of bounds", input.index)));
        }
        constants[input.index] = ConstantDef { global: RefGlobal(input.global), fields: input.fields };
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "update_constant".to_string(),
            Some("Update an existing constant".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" }, "global": { "type": "integer" }, "fields": { "type": "array", "items": { "type": "integer" } } },
                "required": ["index", "global", "fields"],
                "additionalProperties": false
            }),
            UpdateConstantHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}