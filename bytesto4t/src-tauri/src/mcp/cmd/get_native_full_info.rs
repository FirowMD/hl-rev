use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct GetNativeFullInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetNativeFullInfoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = arguments
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::Validation("Missing 'index'".to_string()))? as usize;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if index >= bytecode.natives.len() {
            return Err(McpError::Validation("Native index out of bounds".to_string()));
        }

        let native = &bytecode.natives[index];
        let out = json!({
            "name": native.name.0,
            "lib": native.lib.0,
            "t": native.t.0,
            "findex": native.findex.0
        });
        Ok(CallToolResult::text(out.to_string()))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_native_full_info".to_string(),
            Some("Get full native info".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" } },
                "required": ["index"],
                "additionalProperties": false
            }),
            GetNativeFullInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}