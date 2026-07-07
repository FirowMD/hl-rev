use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetFunctionListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetFunctionListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut function_names = Vec::new();
        for (index, function) in bytecode.functions.iter().enumerate() {
            function_names.push(function.name(&bytecode).to_string() + &function.findex.to_string() + "@" + &index.to_string());
        }

        Ok(CallToolResult::text(function_names.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_function_list".to_string(),
            Some("Get list of functions".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetFunctionListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}