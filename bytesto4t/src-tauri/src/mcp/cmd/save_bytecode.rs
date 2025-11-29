use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct SaveBytecodeHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for SaveBytecodeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut stripped = bytecode.clone();
        stripped.strip();

        let mut file = std::fs::File::create(file_path).map_err(|e| McpError::Internal(e.to_string()))?;
        stripped.serialize(&mut file).map_err(|e| McpError::Internal(e.to_string()))?;

        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "save_bytecode".to_string(),
            Some("Save stripped bytecode to file".to_string()),
            json!({
                "type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"],
                "additionalProperties": false
            }),
            SaveBytecodeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}

