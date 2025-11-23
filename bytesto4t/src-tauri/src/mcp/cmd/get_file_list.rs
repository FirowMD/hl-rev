use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetFileListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetFileListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let debug_files = bytecode
            .debug_files
            .as_ref()
            .ok_or_else(|| McpError::Validation("debug_files not loaded".to_string()))?;

        let mut file_names = Vec::new();
        for (index, file) in debug_files.iter().enumerate() {
            file_names.push(file.to_string() + "@" + &index.to_string());
        }

        Ok(CallToolResult::text(file_names.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_file_list".to_string(),
            Some("Get list of debug files".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetFileListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}