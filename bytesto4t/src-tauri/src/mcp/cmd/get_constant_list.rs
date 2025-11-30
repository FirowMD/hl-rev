use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetConstantListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetConstantListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let constants = bytecode.constants.as_ref().ok_or_else(|| McpError::Validation("constants not loaded".to_string()))?;
        let mut constant_list = Vec::new();
        for (index, c) in constants.iter().enumerate() {
            constant_list.push(format!("{:?}@{}", c, index));
        }

        Ok(CallToolResult::text(constant_list.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_constant_list".to_string(),
            Some("Get list of constants".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetConstantListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}