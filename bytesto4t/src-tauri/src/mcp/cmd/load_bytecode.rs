use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct LoadBytecodeHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for LoadBytecodeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .app_data
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;

        let bc = hlbc::Bytecode::from_file(Path::new(file_path))
            .map_err(|e| McpError::Internal(e.to_string()))?;
        app_data.target_file_path = file_path.to_string();
        app_data.bytecode = Some(bc);

        Ok(CallToolResult::text("loaded"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "load_bytecode".to_string(),
            Some("Load bytecode file into app".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string" }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
            LoadBytecodeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}