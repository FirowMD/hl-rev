use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::AppHandle;

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
        let bytes = std::fs::read(path).map_err(|e| McpError::Internal(e.to_string()))?;
        let json_str = serde_json::to_string(&bytes).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(CallToolResult::text(json_str))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "read_binary_file".to_string(),
            Some("Read file bytes".to_string()),
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