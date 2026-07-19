use crate::app_data::Storage;
use crate::bytecode_refs;
use crate::mcp::cmd::support;
use hlbc::types::{Native, RefFun, RefString, RefType, Type};
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct UpdateNativeHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateNativeInput {
    pub index: usize,
    pub lib: String,
    pub name: String,
    pub signature_type: usize,
    pub findex: Option<usize>,
}

#[async_trait]
impl ToolHandler for UpdateNativeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: UpdateNativeInput = serde_json::from_value(
            serde_json::to_value(arguments).map_err(|e| McpError::Validation(e.to_string()))?,
        )
        .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_mut()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if input.index >= bytecode.natives.len() {
            return Err(McpError::Validation(format!(
                "Native index {} out of bounds",
                input.index
            )));
        }

        let lib_idx: usize = input
            .lib
            .parse()
            .map_err(|_| McpError::Validation("Invalid lib string index".to_string()))?;
        let name_idx: usize = input
            .name
            .parse()
            .map_err(|_| McpError::Validation("Invalid name string index".to_string()))?;

        if lib_idx >= bytecode.strings.len() {
            return Err(McpError::Validation(format!(
                "Lib string index {} out of bounds",
                lib_idx
            )));
        }
        if name_idx >= bytecode.strings.len() {
            return Err(McpError::Validation(format!(
                "Name string index {} out of bounds",
                name_idx
            )));
        }
        if input.signature_type >= bytecode.types.len() {
            return Err(McpError::Validation(format!(
                "Type index {} out of bounds",
                input.signature_type
            )));
        }
        match &bytecode.types[input.signature_type] {
            Type::Fun(_) | Type::Method(_) => {}
            other => {
                return Err(McpError::Validation(format!(
                    "Invalid native signature type at index {}: {:?}",
                    input.signature_type, other
                )));
            }
        }

        let lib = RefString(lib_idx);
        let name = RefString(name_idx);
        let t = RefType(input.signature_type);
        let old_findex = bytecode.natives[input.index].findex;
        let findex = input.findex.unwrap_or(old_findex.0);
        support::ensure_findex_in_dense_range(bytecode, findex, false)?;
        support::ensure_findex_available(bytecode, findex, None, Some(input.index))?;
        if findex != old_findex.0 {
            return Err(McpError::Validation(
                "Changing a native findex is not supported; create a replacement native instead"
                    .to_string(),
            ));
        }
        let findex = RefFun(findex);

        let native = Native {
            lib,
            name,
            t,
            findex,
        };
        bytecode_refs::validate_native_refs(bytecode, &native, "updated native")
            .map_err(McpError::Validation)?;
        bytecode.natives[input.index] = native;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "update_native".to_string(),
            Some("Update an existing native".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer"},
                    "lib": {"type": "string", "description": "Decimal string-pool index"},
                    "name": {"type": "string", "description": "Decimal string-pool index"},
                    "signature_type": {"type": "integer"},
                    "findex": {"type": "integer"}
                },
                "required": ["index", "lib", "name", "signature_type"],
                "additionalProperties": false
            }),
            UpdateNativeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
