use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::{RefType, FunPtr};
use hlbc::analysis::find_functions_using_type;

#[derive(Clone)]
pub struct FindFunctionsUsingTypeHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for FindFunctionsUsingTypeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let type_index = arguments
            .get("type_index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::Validation("Missing 'type_index'".to_string()))? as usize;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        if type_index >= bytecode.types.len() {
            return Err(McpError::Validation("Type index out of bounds".to_string()));
        }
        let target_type = RefType(type_index);
        let function_refs = find_functions_using_type(bytecode, target_type);
        let mut function_names = Vec::new();
        for func_ref in function_refs {
            if let Some(func_ptr) = bytecode.safe_get_ref_fun(func_ref) {
                match func_ptr {
                    FunPtr::Fun(function) => {
                        let full_name = function.name(bytecode).to_string() + &function.findex.to_string();
                        if let Some(vec_index) = bytecode.functions.iter().position(|f| f.findex == func_ref) {
                            function_names.push(format!("{}@{}", full_name, vec_index));
                        }
                    }
                    FunPtr::Native(native) => {
                        let full_name = native.name(bytecode).to_string() + &native.findex.to_string();
                        function_names.push(format!("{}@native_{}", full_name, func_ref.0));
                    }
                }
            }
        }
        Ok(CallToolResult::text(function_names.join("\n")))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "find_functions_using_type".to_string(),
            Some("Find functions using a specific type".to_string()),
            json!({
                "type": "object",
                "properties": {"type_index": {"type": "integer"}},
                "required": ["type_index"],
                "additionalProperties": false
            }),
            FindFunctionsUsingTypeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}