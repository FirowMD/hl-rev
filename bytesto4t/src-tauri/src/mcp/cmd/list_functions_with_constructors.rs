use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct ListFunctionsWithConstructorsHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ListFunctionsWithConstructorsHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut result = Vec::new();
        for (index, f) in bytecode.functions.iter().enumerate() {
            let fname = f.name(&bytecode).to_string();
            let is_ctor = fname == "new" || fname.to_lowercase().starts_with("ctor");
            let sig = format!("{:?}", f.ty(&bytecode));
            result.push(serde_json::json!({
                "index": index,
                "name": fname,
                "signature": sig,
                "is_constructor": is_ctor
            }));
        }
        Ok(CallToolResult::text(serde_json::to_string(&result).map_err(|e| McpError::Internal(e.to_string()))?))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "list_functions_with_constructors".to_string(),
            Some("List functions with constructor flag".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            ListFunctionsWithConstructorsHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}