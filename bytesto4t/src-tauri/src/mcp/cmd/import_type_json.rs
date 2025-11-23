use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::BufRead;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::Type;

#[derive(Clone)]
pub struct ImportTypeJsonHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ImportTypeJsonHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let json_path = arguments
            .get("json_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'json_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_mut().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let json_file = std::fs::File::open(json_path).map_err(|e| McpError::Internal(e.to_string()))?;
        let reader = std::io::BufReader::new(json_file);
        let json_content: String = reader
            .lines()
            .map(|line| line.map_err(|e| McpError::Internal(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        let ty = Type::from_json(json_content.as_str()).map_err(|e| McpError::Internal(e.to_string()))?;
        bytecode.add_type(ty);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "import_type_json".to_string(),
            Some("Import type from JSON file".to_string()),
            json!({
                "type": "object",
                "properties": {"json_path": {"type": "string"}},
                "required": ["json_path"],
                "additionalProperties": false
            }),
            ImportTypeJsonHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}