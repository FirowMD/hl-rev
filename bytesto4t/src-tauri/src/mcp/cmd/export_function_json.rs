use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;

#[derive(Clone)]
pub struct ExportFunctionJsonHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ExportFunctionJsonHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let function_index = arguments
            .get("function_index")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'function_index'".to_string()))?;
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let functions = &bytecode.functions;

        let mut function = None;
        for f in functions {
            if f.findex.to_string() == function_index {
                function = Some(f);
                break;
            }
        }
        if function.is_none() {
            return Err(McpError::Validation(format!("Function with index {} not found", function_index)));
        }
        let json_content = function.unwrap().to_json().map_err(|e| McpError::Internal(e.to_string()))?;
        let mut file = std::fs::File::create(file_path).map_err(|e| McpError::Internal(e.to_string()))?;
        writeln!(file, "{}", json_content).map_err(|e| McpError::Internal(e.to_string()))?;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "export_function_json".to_string(),
            Some("Export function to JSON file".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "function_index": {"type": "string"},
                    "file_path": {"type": "string"}
                },
                "required": ["function_index", "file_path"],
                "additionalProperties": false
            }),
            ExportFunctionJsonHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}