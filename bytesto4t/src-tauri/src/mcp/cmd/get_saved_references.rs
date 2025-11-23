use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetSavedReferencesHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetSavedReferencesHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let resp = app_data.references.as_ref().map(|r| (r.element_index, r.references.clone()));
        let json_str = serde_json::to_string(&resp).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(CallToolResult::text(json_str))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_saved_references".to_string(),
            Some("Get saved references".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetSavedReferencesHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}