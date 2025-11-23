use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetConstantFullInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetConstantFullInfoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = arguments
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::Validation("Missing 'index'".to_string()))? as usize;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let constants = bytecode.constants.as_ref().ok_or_else(|| McpError::Validation("No constants defined".to_string()))?;
        if index >= constants.len() {
            return Err(McpError::Validation("Constant index out of bounds".to_string()));
        }
        let c = &constants[index];
        let out = json!({ "global": c.global.0, "fields": c.fields });
        Ok(CallToolResult::text(out.to_string()))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_constant_full_info".to_string(),
            Some("Get full info for a constant".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" } },
                "required": ["index"],
                "additionalProperties": false
            }),
            GetConstantFullInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}