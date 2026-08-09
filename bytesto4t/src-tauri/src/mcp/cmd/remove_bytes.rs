use crate::app_data::Storage;
use crate::bytecode_refs;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct RemoveBytesHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for RemoveBytesHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = support::required_index(&arguments, "index")?;
        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let count = bytecode
            .bytes
            .as_ref()
            .ok_or_else(|| {
                McpError::Validation("Bytecode versions before v5 have no bytes pool".to_string())
            })?
            .1
            .len();
        bytecode_refs::ensure_tail_delete(
            "Bytes",
            index,
            count,
            bytecode_refs::bytes_references(bytecode, index),
        )
        .map_err(McpError::Validation)?;
        let mut candidate = bytecode.clone();
        let (blob, positions) = candidate.bytes.as_mut().expect("checked bytes pool");
        let start = positions.pop().expect("checked bytes index");
        blob.truncate(start);
        support::rebuild_runtime_indexes(&mut candidate)?;
        *bytecode = candidate;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "delete_bytes".to_string(),
            Some("Delete the unreferenced tail entry of a v5 bytes pool".to_string()),
            json!({
                "type": "object",
                "properties": {"index": {"type": "integer", "minimum": 0}},
                "required": ["index"],
                "additionalProperties": false
            }),
            RemoveBytesHandler { app_handle },
        )
        .await?;
    Ok(())
}
