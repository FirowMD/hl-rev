use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::fmt::EnhancedFmt;
use hlbc::Resolve;

#[derive(Clone)]
pub struct GetGlobalListHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetGlobalListHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut globals = Vec::new();
        for (index, g) in bytecode.globals.iter().enumerate() {
            let ty = bytecode.get(*g);
            globals.push(ty.display::<EnhancedFmt>(&bytecode).to_string() + "@" + &index.to_string());
        }

        Ok(CallToolResult::text(globals.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_global_list".to_string(),
            Some("Get list of globals".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetGlobalListHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}