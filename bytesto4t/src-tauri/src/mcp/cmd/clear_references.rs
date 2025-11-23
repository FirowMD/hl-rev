use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct ClearReferencesHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ClearReferencesHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        app_data.references = None;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "clear_references".to_string(),
            Some("Clear saved references".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            ClearReferencesHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}