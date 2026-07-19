use crate::app_data::Storage;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct RemoveConstantHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for RemoveConstantHandler {
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

        let constants = bytecode
            .constants
            .as_mut()
            .ok_or_else(|| McpError::Validation("No constants defined".to_string()))?;
        if index >= constants.len() {
            return Err(McpError::Validation(
                "Constant index out of bounds".to_string(),
            ));
        }
        let mut candidate = bytecode.clone();
        let candidate_constants = candidate.constants.as_mut().ok_or_else(|| {
            McpError::Internal("Constants disappeared while preparing the deletion".to_string())
        })?;
        candidate_constants.remove(index);
        support::rebuild_runtime_indexes(&mut candidate)?;
        *bytecode = candidate;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "delete_constant".to_string(),
            Some("Delete a constant by index".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" } },
                "required": ["index"],
                "additionalProperties": false
            }),
            RemoveConstantHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
