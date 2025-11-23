use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetFloatListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetFloatListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut float_list = Vec::new();
        for (index, v) in bytecode.floats.iter().enumerate() {
            float_list.push(format!("{}@{}", v, index));
        }

        Ok(CallToolResult::text(float_list.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_float_list".to_string(),
            Some("Get list of floats".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetFloatListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}