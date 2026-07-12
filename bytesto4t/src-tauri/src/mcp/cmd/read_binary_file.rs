use crate::app_data::Storage;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct ReadBinaryFileHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ReadBinaryFileHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'path'".to_string()))?;
        let requested_path =
            std::fs::canonicalize(path).map_err(|e| McpError::Internal(e.to_string()))?;

        let target_file_path = {
            let state = self.app_handle.state::<Storage>();
            let app_data = state
                .bytecode
                .lock()
                .map_err(|e| McpError::Internal(e.to_string()))?;

            if app_data.target_file_path.is_empty() || app_data.bytecode.is_none() {
                return Err(McpError::Validation("bytecode not loaded".to_string()));
            }

            app_data.target_file_path.clone()
        };

        let loaded_path = std::fs::canonicalize(&target_file_path)
            .map_err(|e| McpError::Internal(e.to_string()))?;

        if requested_path != loaded_path {
            return Err(McpError::Validation(
                "read_binary_file can only read the currently loaded bytecode file".to_string(),
            ));
        }

        let bytes = std::fs::read(requested_path).map_err(|e| McpError::Internal(e.to_string()))?;
        let json_str =
            serde_json::to_string(&bytes).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(CallToolResult::text(json_str))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "read_binary_file".to_string(),
            Some("Read bytes from the currently loaded bytecode file".to_string()),
            json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"],
                "additionalProperties": false
            }),
            ReadBinaryFileHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
