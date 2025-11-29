use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::{Native, RefFun, RefString, RefType, Type};

#[derive(Clone)]
pub struct AddNativeHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct NewNativeInput {
    pub lib: String,
    pub name: String,
    pub signature_type: usize,
    pub findex: Option<usize>,
}

#[async_trait]
impl ToolHandler for AddNativeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: NewNativeInput = serde_json::from_value(serde_json::to_value(arguments)
            .map_err(|e| McpError::Validation(e.to_string()))?)
            .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let lib_idx: usize = input.lib.parse().map_err(|_| McpError::Validation("Invalid lib string index".to_string()))?;
        let name_idx: usize = input.name.parse().map_err(|_| McpError::Validation("Invalid name string index".to_string()))?;

        if lib_idx >= bytecode.strings.len() {
            return Err(McpError::Validation(format!("Lib string index {} out of bounds", lib_idx)));
        }
        if name_idx >= bytecode.strings.len() {
            return Err(McpError::Validation(format!("Name string index {} out of bounds", name_idx)));
        }
        if input.signature_type >= bytecode.types.len() {
            return Err(McpError::Validation(format!("Type index {} out of bounds", input.signature_type)));
        }
        match &bytecode.types[input.signature_type] {
            Type::Fun(_) | Type::Method(_) => {}
            other => {
                return Err(McpError::Validation(format!("Invalid native signature type at index {}: {:?}", input.signature_type, other)));
            }
        }

        let lib = RefString(lib_idx);
        let name = RefString(name_idx);
        let t = RefType(input.signature_type);
        let findex = RefFun(input.findex.unwrap_or(bytecode.natives.len()));

        bytecode.natives.push(Native { lib, name, t, findex });
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "create_native".to_string(),
            Some("Create a new native".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "lib": {"type": "string"},
                    "name": {"type": "string"},
                    "signature_type": {"type": "integer"},
                    "findex": {"type": "integer"}
                },
                "required": ["lib", "name", "signature_type"],
                "additionalProperties": false
            }),
            AddNativeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
