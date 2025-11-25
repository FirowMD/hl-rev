use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::fmt::EnhancedFmt;
use hlbc::opcodes::Opcode;

#[derive(Clone)]
pub struct GetReferencesHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetReferencesHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = arguments
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::Validation("Missing 'index'".to_string()))? as usize;
        let typ_str = arguments
            .get("typ")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'typ'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let out = match typ_str {
            "function" => {
                if index >= bytecode.functions.len() {
                    return Err(McpError::Validation("Function index out of bounds".to_string()));
                }
                let findex = bytecode.functions[index].findex;
                let mut refs = String::new();
                bytecode
                    .functions
                    .iter()
                    .enumerate()
                    .flat_map(|(_i, f)| std::iter::repeat(f).zip(f.find_fun_refs()))
                    .for_each(|(f, (op_idx, op, fun))| {
                        if fun.0 == findex.0 {
                            refs.push_str(&format!(
                                "{} at {}: {}\n",
                                f.display_header::<EnhancedFmt>(&bytecode),
                                op_idx,
                                op.name()
                            ));
                        }
                    });
                refs
            }
            "string" => {
                if index >= bytecode.strings.len() {
                    return Err(McpError::Validation("String index out of bounds".to_string()));
                }
                let mut refs = String::new();
                bytecode
                    .functions
                    .iter()
                    .flat_map(|f| f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op)))
                    .for_each(|(f, op_idx, op)| match op {
                        Opcode::String { ptr, .. } => {
                            if ptr.0 == index {
                                refs.push_str(&format!(
                                    "{} at {}: {}\n",
                                    f.display_header::<EnhancedFmt>(&bytecode),
                                    op_idx,
                                    op.name()
                                ));
                            }
                        }
                        _ => {}
                    });
                refs
            }
            "global" => {
                if index >= bytecode.globals.len() {
                    return Err(McpError::Validation("Global index out of bounds".to_string()));
                }
                let mut refs = String::new();
                if let Some(constants) = &bytecode.constants {
                    for (i, c) in constants.iter().enumerate() {
                        if c.global.0 == index {
                            refs.push_str(&format!("Constant@{}\n", i));
                        }
                    }
                }
                bytecode
                    .functions
                    .iter()
                    .flat_map(|f| f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op)))
                    .for_each(|(f, op_idx, op)| match op {
                        Opcode::GetGlobal { global, .. } | Opcode::SetGlobal { global, .. } => {
                            if global.0 == index {
                                refs.push_str(&format!(
                                    "{} at {}: {}\n",
                                    f.display_header::<EnhancedFmt>(&bytecode),
                                    op_idx,
                                    op.name()
                                ));
                            }
                        }
                        _ => {}
                    });
                refs
            }
            _ => return Err(McpError::Validation(format!("Unsupported item type: {}", typ_str))),
        };

        Ok(CallToolResult::text(out))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_references".to_string(),
            Some("Get references for item".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "integer" }, "typ": { "type": "string" } },
                "required": ["index", "typ"],
                "additionalProperties": false
            }),
            GetReferencesHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
