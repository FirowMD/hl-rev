use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetBytesFullInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetBytesFullInfoHandler {
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

        match &bytecode.bytes {
            Some((bytes_data, indices)) => {
                if index >= indices.len() {
                    return Err(McpError::Validation("Bytes index out of bounds".to_string()));
                }
                let start_pos = indices[index];
                let end_pos = indices.get(index + 1).copied().unwrap_or(bytes_data.len());
                let byte_slice = &bytes_data[start_pos..end_pos];
                let json_str = serde_json::to_string(byte_slice).map_err(|e| McpError::Internal(e.to_string()))?;
                Ok(CallToolResult::text(json_str))
            }
            None => Err(McpError::Validation("No bytes data available".to_string())),
        }
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_bytes_full_info".to_string(),
            Some("Get full bytes for an entry".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" } },
                "required": ["index"],
                "additionalProperties": false
            }),
            GetBytesFullInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}