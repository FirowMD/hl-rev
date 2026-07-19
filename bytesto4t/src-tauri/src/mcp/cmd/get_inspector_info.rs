use crate::app_data::Storage;
use crate::bytecode_refs;
use crate::mcp::cmd::support;
use hlbc::fmt::EnhancedFmt;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct GetInspectorInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetInspectorInfoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index = support::required_index(&arguments, "index")?;
        let item_type = arguments
            .get("typ")
            .and_then(Value::as_str)
            .ok_or_else(|| McpError::Validation("Missing 'typ'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut output = match item_type {
            "function" => bytecode
                .functions
                .get(index)
                .map(|function| format!("{}", function.display_header::<EnhancedFmt>(bytecode)))
                .ok_or_else(|| {
                    McpError::Validation(format!("Function index {} out of bounds", index))
                })?,
            "class" | "type" => bytecode
                .types
                .get(index)
                .map(|ty| format!("{}", ty.display::<EnhancedFmt>(bytecode)))
                .ok_or_else(|| {
                    McpError::Validation(format!("Type index {} out of bounds", index))
                })?,
            "file" => bytecode
                .debug_files
                .as_ref()
                .ok_or_else(|| McpError::Validation("debug_files not loaded".to_string()))?
                .get(index)
                .map(|file| format!("File@{}: {}", index, file))
                .ok_or_else(|| {
                    McpError::Validation(format!("File index {} out of bounds", index))
                })?,
            "global" => bytecode
                .globals
                .get(index)
                .map(|global| {
                    format!(
                        "Global@{}: {}",
                        index,
                        global.display::<EnhancedFmt>(bytecode)
                    )
                })
                .ok_or_else(|| {
                    McpError::Validation(format!("Global index {} out of bounds", index))
                })?,
            "constant" => bytecode
                .constants
                .as_ref()
                .ok_or_else(|| McpError::Validation("constants not loaded".to_string()))?
                .get(index)
                .map(|constant| format!("Constant@{}: {:?}", index, constant))
                .ok_or_else(|| {
                    McpError::Validation(format!("Constant index {} out of bounds", index))
                })?,
            "string" => bytecode
                .strings
                .get(index)
                .map(|string| format!("String@{}: {:?}", index, string))
                .ok_or_else(|| {
                    McpError::Validation(format!("String index {} out of bounds", index))
                })?,
            "int" => bytecode
                .ints
                .get(index)
                .map(|value| format!("Int@{}: {}", index, value))
                .ok_or_else(|| {
                    McpError::Validation(format!("Int index {} out of bounds", index))
                })?,
            "float" => bytecode
                .floats
                .get(index)
                .map(|value| format!("Float@{}: {}", index, value))
                .ok_or_else(|| {
                    McpError::Validation(format!("Float index {} out of bounds", index))
                })?,
            "native" => bytecode
                .natives
                .get(index)
                .map(|native| {
                    format!(
                        "Native@{}: {}",
                        index,
                        native.display::<EnhancedFmt>(bytecode)
                    )
                })
                .ok_or_else(|| {
                    McpError::Validation(format!("Native index {} out of bounds", index))
                })?,
            "bytes" => format_bytes(bytecode, index)?,
            _ => {
                return Err(McpError::Validation(format!(
                    "Unsupported item type: {}",
                    item_type
                )))
            }
        };

        let references = bytecode_refs::references_for_item(bytecode, item_type, index)
            .map_err(McpError::Validation)?;
        output.push_str("\n\nReferences:");
        if references.is_empty() {
            output.push_str(" none");
        } else {
            for reference in references {
                output.push('\n');
                output.push_str(&reference);
            }
        }

        Ok(CallToolResult::text(output))
    }
}

fn format_bytes(bytecode: &hlbc::Bytecode, index: usize) -> McpResult<String> {
    let (bytes, offsets) = bytecode
        .bytes
        .as_ref()
        .ok_or_else(|| McpError::Validation("No bytes data available".to_string()))?;
    let start = *offsets
        .get(index)
        .ok_or_else(|| McpError::Validation(format!("Bytes index {} out of bounds", index)))?;
    let end = offsets.get(index + 1).copied().unwrap_or(bytes.len());
    let slice = bytes.get(start..end).ok_or_else(|| {
        McpError::Internal(format!(
            "Invalid byte range {}..{} for bytes index {}",
            start, end, index
        ))
    })?;

    let mut output = format!("Bytes@{}: {} bytes\n", index, slice.len());
    for (line_index, chunk) in slice.chunks(16).enumerate() {
        output.push_str(&format!("\n{:08x}  ", line_index * 16));
        for (column, byte) in chunk.iter().enumerate() {
            if column == 8 {
                output.push(' ');
            }
            output.push_str(&format!("{:02x} ", byte));
        }
        for column in chunk.len()..16 {
            if column == 8 {
                output.push(' ');
            }
            output.push_str("   ");
        }
        output.push_str(" |");
        for byte in chunk {
            output.push(if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            });
        }
        output.push('|');
    }
    Ok(output)
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "get_inspector_info".to_string(),
            Some("Inspect an indexed bytecode item and its references".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": {
                        "oneOf": [
                            {"type": "integer", "minimum": 0},
                            {"type": "string", "pattern": "^[0-9]+$"}
                        ]
                    },
                    "typ": {
                        "type": "string",
                        "enum": ["function", "class", "type", "file", "global", "constant", "string", "int", "float", "native", "bytes"]
                    }
                },
                "required": ["index", "typ"],
                "additionalProperties": false
            }),
            GetInspectorInfoHandler { app_handle },
        )
        .await?;
    Ok(())
}
