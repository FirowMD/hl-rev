use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::Type;
use hlbc_decompiler::{decompile_function, decompile_class};

#[derive(Clone)]
pub struct GetDecompiledInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetDecompiledInfoHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let app_item = app_data.selected_item.as_ref().ok_or_else(|| McpError::Validation("No item selected".to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let index: usize = app_item.index.parse().map_err(|_| McpError::Validation("Invalid index format".to_string()))?;

        let out = match app_item.typ.as_str() {
            "function" => {
                if index >= bytecode.functions.len() {
                    return Err(McpError::Validation("Function index out of bounds".to_string()));
                }
                let function = &bytecode.functions[index];
                let decompiled = decompile_function(&bytecode, &function);
                format!("{}", decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2)))
            }
            "class" => {
                if index >= bytecode.types.len() {
                    return Err(McpError::Validation("Type index out of bounds".to_string()));
                }
                match &bytecode.types[index] {
                    Type::Obj(obj) => {
                        let decompiled = decompile_class(&bytecode, obj);
                        format!("{}", decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2)))
                    }
                    _ => return Err(McpError::Validation("Type is not an object".to_string())),
                }
            }
            _ => return Err(McpError::Validation(format!("Unsupported item type: {}", app_item.typ))),
        };

        Ok(CallToolResult::text(out))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_decompiled_info".to_string(),
            Some("Get decompiled info for selected item".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetDecompiledInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}