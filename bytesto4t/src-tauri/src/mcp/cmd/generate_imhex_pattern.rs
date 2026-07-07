use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use crate::structgen;

#[derive(Clone)]
pub struct GenerateImhexPatternHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GenerateImhexPatternHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let type_index = arguments
            .get("type_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::Validation("Missing 'type_index'".to_string()))? as usize;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if type_index >= bytecode.types.len() {
            return Err(McpError::Validation("Type index out of bounds".to_string()));
        }

        let pattern = structgen::generate_imhex_pattern(bytecode, type_index);
        Ok(CallToolResult::text(pattern))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "generate_imhex_pattern".to_string(),
            Some("Generate ImHex pattern for a type".to_string()),
            json!({
                "type": "object",
                "properties": {"type_index": {"type": "integer"}},
                "required": ["type_index"],
                "additionalProperties": false
            }),
            GenerateImhexPatternHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}