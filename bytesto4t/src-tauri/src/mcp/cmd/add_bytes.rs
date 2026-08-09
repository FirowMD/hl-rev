use crate::app_data::Storage;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct AddBytesHandler {
    pub app_handle: AppHandle,
}

#[derive(serde::Deserialize)]
struct Input {
    data: Vec<u8>,
}

#[async_trait]
impl ToolHandler for AddBytesHandler {
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
        positions.push(blob.len());
        blob.extend(input.data);
        support::rebuild_runtime_indexes(&mut candidate)?;
        *bytecode = candidate;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "create_bytes".to_string(),
            Some("Append one entry to a v5 bytes pool".to_string()),
            json!({
                "type": "object",
                "properties": {"data": {"type": "array", "items": {"type": "integer", "minimum": 0, "maximum": 255}}},
                "required": ["data"],
                "additionalProperties": false
            }),
            AddBytesHandler { app_handle },
        )
        .await?;
    Ok(())
}
