use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::fmt::EnhancedFmt;

#[derive(Clone)]
pub struct GetTypeListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetTypeListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut type_names = Vec::new();
        for (index, t) in bytecode.types.iter().enumerate() {
            type_names.push(t.display::<EnhancedFmt>(&bytecode).to_string() + "@" + &index.to_string());
        }

        Ok(CallToolResult::text(type_names.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_type_list".to_string(),
            Some("Get list of types".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetTypeListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}