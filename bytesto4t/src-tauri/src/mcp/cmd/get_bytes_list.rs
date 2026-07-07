use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetBytesListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetBytesListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let out = match &bytecode.bytes {
            Some((bytes_data, indices)) => {
                let mut bytes_list = Vec::new();
                for (index, &start_pos) in indices.iter().enumerate() {
                    let end_pos = indices.get(index + 1).copied().unwrap_or(bytes_data.len());
                    let byte_slice = &bytes_data[start_pos..end_pos];
                    let hex_str = byte_slice.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join(" ");
                    let display_str = if hex_str.len() > 50 { format!("{} ... ({} bytes)", &hex_str[..50], byte_slice.len()) } else { format!("{} ({} bytes)", hex_str, byte_slice.len()) };
                    bytes_list.push(format!("{}@{}", display_str, index));
                }
                bytes_list.join("\n")
            }
            None => String::new(),
        };
        Ok(CallToolResult::text(out))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_bytes_list".to_string(),
            Some("Get list of bytes entries".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetBytesListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}