use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::fmt::EnhancedFmt;

#[derive(Clone)]
pub struct GetDashboardInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetDashboardInfoHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let version = bytecode.version;
        let int_n = bytecode.ints.len();
        let float_n = bytecode.floats.len();
        let string_n = bytecode.strings.len();
        let byte_n = bytecode.bytes.as_ref().map(|(bytes, _)| bytes.len()).unwrap_or(0);
        let file_n = bytecode.debug_files.as_ref().map(|files| files.len()).unwrap_or(0);
        let type_n = bytecode.types.len();
        let global_n = bytecode.globals.len();
        let native_n = bytecode.natives.len();
        let function_n = bytecode.functions.len();
        let constant_n = bytecode.constants.as_ref().map(|constants| constants.len()).unwrap_or(0);
        let entrypoint = bytecode.entrypoint.display::<EnhancedFmt>(&bytecode);

        let msg = format!(
            "File path: {}\nVersion: {}\nInts: {}\nFloats: {}\nStrings: {}\nBytes: {}\nFiles: {}\nTypes: {}\nGlobals: {}\nNatives: {}\nFunctions: {}\nConstants: {}\nEntrypoint: {}",
            app_data.target_file_path,
            version,
            int_n,
            float_n,
            string_n,
            byte_n,
            file_n,
            type_n,
            global_n,
            native_n,
            function_n,
            constant_n,
            entrypoint
        );

        Ok(CallToolResult::text(msg))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_dashboard_info".to_string(),
            Some("Get dashboard summary info".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetDashboardInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}