use crate::app_data::Storage;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct ExportTypeJsonHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ExportTypeJsonHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = support::required_index(&arguments, "type_index")?;
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let types = &bytecode.types;

        if index >= types.len() {
            return Err(McpError::Validation("Type index out of bounds".to_string()));
        }

        let type_obj = &types[index];
        let json_content = type_obj
            .to_json()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let mut file =
            std::fs::File::create(file_path).map_err(|e| McpError::Internal(e.to_string()))?;
        writeln!(file, "{}", json_content).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "export_type_json".to_string(),
            Some("Export type to JSON file".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "type_index": {"oneOf": [{"type": "integer", "minimum": 0}, {"type": "string", "pattern": "^[0-9]+$"}]},
                    "file_path": {"type": "string"}
                },
                "required": ["type_index", "file_path"],
                "additionalProperties": false
            }),
            ExportTypeJsonHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
