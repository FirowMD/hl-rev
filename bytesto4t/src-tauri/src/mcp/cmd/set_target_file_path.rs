use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct SetTargetFilePathHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for SetTargetFilePathHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        app_data.target_file_path = file_path.to_string();
        let path = Path::new(&app_data.target_file_path);
        app_data.bytecode = Some(hlbc::Bytecode::from_file(path).map_err(|e| McpError::Internal(e.to_string()))?);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "set_target_file_path".to_string(),
            Some("Set target file path and load bytecode".to_string()),
            json!({
                "type": "object",
                "properties": { "file_path": { "type": "string" } },
                "required": ["file_path"],
                "additionalProperties": false
            }),
            SetTargetFilePathHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}