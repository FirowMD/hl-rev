use crate::app_data::Storage;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::BufRead;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
struct LoadFunctionAddressesHandler {
    app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for LoadFunctionAddressesHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = arguments
            .get("file_path")
            .and_then(Value::as_str)
            .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))?;
        let file = std::fs::File::open(file_path)
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let addresses = std::io::BufReader::new(file)
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let count = addresses.len();

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        app_data.function_addresses = Some(addresses);
        Ok(CallToolResult::text(format!("Loaded {} addresses", count)))
    }
}

#[derive(Clone)]
struct GetFunctionAddressesHandler {
    app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetFunctionAddressesHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let addresses = app_data
            .function_addresses
            .as_ref()
            .ok_or_else(|| McpError::Validation("function_addresses not loaded".to_string()))?;
        let output = serde_json::to_string(addresses)
            .map_err(|error| McpError::Internal(error.to_string()))?;
        Ok(CallToolResult::text(output))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "load_function_addresses".to_string(),
            Some("Load function addresses from a line-delimited text file".to_string()),
            json!({
                "type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"],
                "additionalProperties": false
            }),
            LoadFunctionAddressesHandler {
                app_handle: app_handle.clone(),
            },
        )
        .await?;
    server
        .add_tool(
            "get_function_addresses".to_string(),
            Some("Get the loaded function address list".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetFunctionAddressesHandler { app_handle },
        )
        .await?;
    Ok(())
}
