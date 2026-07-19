use crate::app_data::Storage;
use crate::mcp::cmd::support;
use hlbc::types::Type;
use hlbc_decompiler::{decompile_class, decompile_function};
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct GetDecompiledInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetDecompiledInfoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = support::required_index(&arguments, "index")?;
        let typ_str = arguments
            .get("typ")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'typ'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let out = match typ_str {
            "function" => {
                if index >= bytecode.functions.len() {
                    return Err(McpError::Validation(
                        "Function index out of bounds".to_string(),
                    ));
                }
                let function = &bytecode.functions[index];
                let decompiled = decompile_function(&bytecode, &function);
                format!(
                    "{}",
                    decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))
                )
            }
            "class" => {
                if index >= bytecode.types.len() {
                    return Err(McpError::Validation("Type index out of bounds".to_string()));
                }
                match &bytecode.types[index] {
                    Type::Obj(obj) => {
                        let decompiled = decompile_class(&bytecode, obj);
                        format!(
                            "{}",
                            decompiled
                                .display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))
                        )
                    }
                    _ => return Err(McpError::Validation("Type is not an object".to_string())),
                }
            }
            _ => {
                return Err(McpError::Validation(format!(
                    "Unsupported item type: {}",
                    typ_str
                )))
            }
        };

        Ok(CallToolResult::text(out))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_decompiled_info".to_string(),
            Some("Get decompiled info for item".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": {"oneOf": [{"type": "integer", "minimum": 0}, {"type": "string", "pattern": "^[0-9]+$"}]},
                    "typ": {"type": "string", "enum": ["function", "class"]}
                },
                "required": ["index", "typ"],
                "additionalProperties": false
            }),
            GetDecompiledInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
