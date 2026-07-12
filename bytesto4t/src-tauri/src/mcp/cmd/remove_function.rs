use crate::app_data::Storage;
use crate::bytecode_refs;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct RemoveFunctionHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for RemoveFunctionHandler {
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

        if index >= bytecode.functions.len() {
            return Err(McpError::Validation(
                "Function index out of bounds".to_string(),
            ));
        }
        let findex = bytecode.functions[index].findex;
        bytecode_refs::ensure_tail_delete(
            "Function",
            index,
            bytecode.functions.len(),
            bytecode_refs::function_references(bytecode, findex),
        )
        .map_err(McpError::Validation)?;
        bytecode.functions.remove(index);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "delete_function".to_string(),
            Some("Delete a function".to_string()),
            json!({
                "type": "object",
                "properties": {"index": {"type": "integer"}},
                "required": ["index"],
                "additionalProperties": false
            }),
            RemoveFunctionHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
