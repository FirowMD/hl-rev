use crate::app_data::Storage;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct UpdateBytesHandler {
    pub app_handle: AppHandle,
}

#[derive(serde::Deserialize)]
struct Input {
    index: usize,
    data: Vec<u8>,
}

#[async_trait]
impl ToolHandler for UpdateBytesHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: Input = serde_json::from_value(
            serde_json::to_value(arguments).map_err(|e| McpError::Validation(e.to_string()))?,
        )
        .map_err(|e| McpError::Validation(e.to_string()))?;
        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let mut candidate = bytecode.clone();
        let (blob, positions) = candidate.bytes.as_mut().ok_or_else(|| {
            McpError::Validation("Bytecode versions before v5 have no bytes pool".to_string())
        })?;
        let start = *positions
            .get(input.index)
            .ok_or_else(|| McpError::Validation("Bytes index out of bounds".to_string()))?;
        let end = positions
            .get(input.index + 1)
            .copied()
            .unwrap_or(blob.len());
        let old_len = end - start;
        let new_len = input.data.len();
        blob.splice(start..end, input.data);
        for position in positions.iter_mut().skip(input.index + 1) {
            *position = if new_len >= old_len {
                position
                    .checked_add(new_len - old_len)
                    .ok_or_else(|| McpError::Validation("Bytes offset overflow".to_string()))?
            } else {
                position
                    .checked_sub(old_len - new_len)
                    .ok_or_else(|| McpError::Validation("Bytes offset underflow".to_string()))?
            };
        }
        support::rebuild_runtime_indexes(&mut candidate)?;
        *bytecode = candidate;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "update_bytes".to_string(),
            Some("Replace one entry in a v5 bytes pool".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer", "minimum": 0},
                    "data": {"type": "array", "items": {"type": "integer", "minimum": 0, "maximum": 255}}
                },
                "required": ["index", "data"],
                "additionalProperties": false
            }),
            UpdateBytesHandler { app_handle },
        )
        .await?;
    Ok(())
}
