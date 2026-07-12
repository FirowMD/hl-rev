use crate::app_data::Storage;
use crate::bytecode_refs;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct RemoveNativeHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for RemoveNativeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = arguments
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::Validation("Missing 'index'".to_string()))?
            as usize;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if index >= bytecode.natives.len() {
            return Err(McpError::Validation(
                "Native index out of bounds".to_string(),
            ));
        }

        let findex = bytecode.natives[index].findex;
        bytecode_refs::ensure_tail_delete(
            "Native",
            index,
            bytecode.natives.len(),
            bytecode_refs::function_references(bytecode, findex),
        )
        .map_err(McpError::Validation)?;
        bytecode.natives.remove(index);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "delete_native".to_string(),
            Some("Delete a native by index".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" } },
                "required": ["index"],
                "additionalProperties": false
            }),
            RemoveNativeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
