use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::fmt::EnhancedFmt;
use hlbc::opcodes::Opcode;
use hlbc::types::Type;

#[derive(Clone)]
pub struct GetInspectorInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetInspectorInfoHandler {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let state = self.app_handle.state::<Storage>();
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let app_item = app_data.selected_item.as_ref().ok_or_else(|| McpError::Validation("No item selected".to_string()))?;
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let index: usize = app_item.index.parse().map_err(|_| McpError::Validation("Invalid index format".to_string()))?;
        let item_type = &app_item.typ;

        let out = match item_type.as_str() {
            "function" => {
                if index >= bytecode.functions.len() {
                    return Err(McpError::Validation("Function index out of bounds".to_string()));
                }
                let f = &bytecode.functions[index];
                let mut info = format!("{}{}\n", f.name.display::<EnhancedFmt>(&bytecode), f.findex);
                let findex = f.findex;
                info.push_str("\n\nReferences:");
                bytecode
                    .functions
                    .iter()
                    .enumerate()
                    .flat_map(|(i, f)| std::iter::repeat((i, f)).zip(f.find_fun_refs()))
                    .for_each(|((_src_idx, f), (op_idx, op, fun))| {
                        if fun.0 == findex.0 {
                            info.push_str(&format!(
                                "\n{} at {}: {}",
                                f.display_header::<EnhancedFmt>(&bytecode),
                                op_idx,
                                op.name()
                            ));
                        }
                    });
                info
            }
            "class" => {
                if index >= bytecode.types.len() {
                    return Err(McpError::Validation("Type index out of bounds".to_string()));
                }
                let type_obj = &bytecode.types[index];
                match type_obj {
                    Type::Fun(_fun) => format!("{}\n", type_obj.display::<EnhancedFmt>(&bytecode)),
                    Type::Obj(_obj) => format!("{}\n", type_obj.display::<EnhancedFmt>(&bytecode)),
                    _ => format!("{}", type_obj.display::<EnhancedFmt>(&bytecode)),
                }
            }
            "file" => {
                let debug_files = bytecode.debug_files.as_ref().ok_or_else(|| McpError::Validation("debug_files not loaded".to_string()))?;
                if index >= debug_files.len() {
                    return Err(McpError::Validation("File index out of bounds".to_string()));
                }
                let mut info = format!("File: {}\n\nFunctions:", debug_files[index]);
                for f in &bytecode.functions {
                    if let Some(debug_info) = f.debug_info.as_ref() {
                        if let Some(last_file_idx) = debug_info.last().map(|(file_idx, _)| file_idx) {
                            if *last_file_idx == index {
                                info.push_str(&format!("\n{}", f.display_header::<EnhancedFmt>(&bytecode)));
                            }
                        }
                    }
                }
                info
            }
            "global" => {
                if index >= bytecode.globals.len() {
                    return Err(McpError::Validation("Global index out of bounds".to_string()));
                }
                let mut info = format!("{}", bytecode.globals[index].display::<EnhancedFmt>(&bytecode));
                info.push_str("\n\nReferences:");
                if let Some(constants) = &bytecode.constants {
                    for (i, c) in constants.iter().enumerate() {
                        if c.global.0 == index {
                            info.push_str(&format!("\nConstant@{}: {:?}", i, c));
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
                                info.push_str(&format!(
                                    "\n{} at {}: {}",
                                    f.display_header::<EnhancedFmt>(&bytecode),
                                    op_idx,
                                    op.name()
                                ));
                            }
                        }
                        _ => {}
                    });
                info
            }
            "constant" => {
                let constants = bytecode.constants.as_ref().ok_or_else(|| McpError::Validation("constants not loaded".to_string()))?;
                if index >= constants.len() {
                    return Err(McpError::Validation("Constant index out of bounds".to_string()));
                }
                format!("{:?}", constants[index])
            }
            "string" => {
                if index >= bytecode.strings.len() {
                    return Err(McpError::Validation("String index out of bounds".to_string()));
                }
                let mut info = format!("String@{}: {:?}", index, bytecode.strings[index]);
                info.push_str("\n\nReferences:");
                bytecode
                    .functions
                    .iter()
                    .flat_map(|f| f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op)))
                    .for_each(|(f, op_idx, op)| match op {
                        Opcode::String { ptr, .. } => {
                            if ptr.0 == index {
                                info.push_str(&format!(
                                    "\n{} at {}: {}",
                                    f.display_header::<EnhancedFmt>(&bytecode),
                                    op_idx,
                                    op.name()
                                ));
                            }
                        }
                        _ => {}
                    });
                info
            }
            "int" => {
                if index >= bytecode.ints.len() {
                    return Err(McpError::Validation("Int index out of bounds".to_string()));
                }
                format!("Int@{}: {}", index, bytecode.ints[index])
            }
            "float" => {
                if index >= bytecode.floats.len() {
                    return Err(McpError::Validation("Float index out of bounds".to_string()));
                }
                format!("Float@{}: {}", index, bytecode.floats[index])
            }
            "native" => {
                if index >= bytecode.natives.len() {
                    return Err(McpError::Validation("Native index out of bounds".to_string()));
                }
                format!("{}\n", bytecode.natives[index].display::<EnhancedFmt>(&bytecode))
            }
            "bytes" => {
                match &bytecode.bytes {
                    Some((bytes_data, indices)) => {
                        if index >= indices.len() {
                            return Err(McpError::Validation("Bytes index out of bounds".to_string()));
                        }
                        let start_pos = indices[index];
                        let end_pos = indices.get(index + 1).copied().unwrap_or(bytes_data.len());
                        let byte_slice = &bytes_data[start_pos..end_pos];
                        let mut info = format!("Bytes@{}: {} bytes\n\n", index, byte_slice.len());
                        for (line_idx, chunk) in byte_slice.chunks(16).enumerate() {
                            let offset = line_idx * 16;
                            info.push_str(&format!("{:08x}  ", offset));
                            for (i, byte) in chunk.iter().enumerate() {
                                if i == 8 { info.push(' '); }
                                info.push_str(&format!("{:02x} ", byte));
                            }
                            let remaining = 16 - chunk.len();
                            for i in 0..remaining {
                                if chunk.len() + i == 8 { info.push(' '); }
                                info.push_str("   ");
                            }
                            info.push_str(" |");
                            for byte in chunk { if *byte >= 32 && *byte <= 126 { info.push(*byte as char); } else { info.push('.'); } }
                            info.push_str("|\n");
                        }
                        info
                    }
                    None => return Err(McpError::Validation("No bytes data available".to_string())),
                }
            }
            _ => return Err(McpError::Validation(format!("Unsupported item type: {}", item_type))),
        };

        Ok(CallToolResult::text(out))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "get_inspector_info".to_string(),
            Some("Get inspector info for selected item".to_string()),
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            GetInspectorInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}