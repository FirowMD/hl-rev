use crate::app_data::Storage;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct ExportFunctionJsonHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for ExportFunctionJsonHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let function = if arguments.contains_key("index") {
            let index = support::required_index(&arguments, "index")?;
            bytecode.functions.get(index).ok_or_else(|| {
                McpError::Validation(format!("Function vector index {} out of bounds", index))
            })?
        } else {
            let findex = support::required_index(&arguments, "function_index")?;
            bytecode
                .functions
                .iter()
                .find(|function| function.findex.0 == findex)
                .ok_or_else(|| {
                    McpError::Validation(format!("Function with findex {} not found", findex))
                })?
        };
        let json_content = function
            .to_json()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let mut file =
            std::fs::File::create(file_path).map_err(|e| McpError::Internal(e.to_string()))?;
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
                    "index": {"type": "integer", "minimum": 0, "description": "Function vector index"},
                    "function_index": {
                        "oneOf": [{"type": "integer", "minimum": 0}, {"type": "string", "pattern": "^[0-9]+$"}],
                        "description": "Legacy function findex argument"
                    },
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"],
                "oneOf": [
                    {"required": ["index"], "not": {"required": ["function_index"]}},
                    {"required": ["function_index"], "not": {"required": ["index"]}}
                ],
                "additionalProperties": false
            }),
            ExportFunctionJsonHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
