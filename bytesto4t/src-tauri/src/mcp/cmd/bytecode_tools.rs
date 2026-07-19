use crate::app_data::Storage;
use hlbc::Bytecode;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
struct GetTargetFileInfoHandler {
    app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetTargetFileInfoHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let target_path = {
            let app_data = state
                .bytecode
                .lock()
                .map_err(|error| McpError::Internal(error.to_string()))?;
            app_data
                .bytecode
                .as_ref()
                .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
            app_data.target_file_path.clone()
        };

        let path = Path::new(&target_path);
        let metadata =
            std::fs::metadata(path).map_err(|error| McpError::Internal(error.to_string()))?;
        let result = json!({
            "path": target_path,
            "name": path.file_name().and_then(|name| name.to_str()).unwrap_or("unknown"),
            "size": metadata.len()
        });
        Ok(CallToolResult::text(result.to_string()))
    }
}

#[derive(Clone)]
struct MergeBytecodeHandler {
    app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for MergeBytecodeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(Value::as_str)
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;
        let other = Bytecode::from_file(Path::new(file_path))
            .map_err(|error| McpError::Validation(error.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let current = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let merged = current
            .clone()
            .merge_with(other)
            .map_err(|error| McpError::Validation(error.to_string()))?;
        app_data.bytecode = Some(merged);
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "get_target_file_info".to_string(),
            Some("Get the loaded bytecode file path, name, and size".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetTargetFileInfoHandler {
                app_handle: app_handle.clone(),
            },
        )
        .await?;
    server
        .add_tool(
            "merge_bytecode".to_string(),
            Some("Merge another HashLink bytecode file into the loaded bytecode".to_string()),
            json!({
                "type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"],
                "additionalProperties": false
            }),
            MergeBytecodeHandler { app_handle },
        )
        .await?;
    Ok(())
}
