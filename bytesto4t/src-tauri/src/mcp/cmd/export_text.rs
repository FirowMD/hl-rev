use crate::app_data::Storage;
use crate::mcp::cmd::support;
use hlbc::fmt::EnhancedFmt;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy)]
enum ListKind {
    Functions,
    Types,
    Files,
}

#[derive(Clone)]
struct SaveListHandler {
    app_handle: AppHandle,
    kind: ListKind,
}

#[async_trait]
impl ToolHandler for SaveListHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = required_path(&arguments)?;
        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let lines = match self.kind {
            ListKind::Functions => bytecode
                .functions
                .iter()
                .enumerate()
                .map(|(index, function)| {
                    format!("{}{}@{}", function.name(bytecode), function.findex, index)
                })
                .collect::<Vec<_>>(),
            ListKind::Types => bytecode
                .types
                .iter()
                .enumerate()
                .map(|(index, ty)| format!("{}@{}", ty.display::<EnhancedFmt>(bytecode), index))
                .collect::<Vec<_>>(),
            ListKind::Files => bytecode
                .debug_files
                .as_ref()
                .ok_or_else(|| McpError::Validation("debug_files not loaded".to_string()))?
                .iter()
                .enumerate()
                .map(|(index, file)| format!("{}@{}", file, index))
                .collect::<Vec<_>>(),
        };
        std::fs::write(file_path, lines.join("\n"))
            .map_err(|error| McpError::Internal(error.to_string()))?;
        Ok(CallToolResult::text(format!(
            "Saved {} entries",
            lines.len()
        )))
    }
}

#[derive(Clone)]
struct SaveDisassembledCodeHandler {
    app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for SaveDisassembledCodeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let file_path = required_path(&arguments)?;
        let index = support::required_index(&arguments, "index")?;
        let state = self.app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        let bytecode = app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let function = bytecode.functions.get(index).ok_or_else(|| {
            McpError::Validation(format!("Function index {} out of bounds", index))
        })?;
        std::fs::write(
            file_path,
            format!("{}\n", function.display::<EnhancedFmt>(bytecode)),
        )
        .map_err(|error| McpError::Internal(error.to_string()))?;
        Ok(CallToolResult::text("ok"))
    }
}

fn required_path(arguments: &HashMap<String, Value>) -> McpResult<&str> {
    arguments
        .get("file_path")
        .and_then(Value::as_str)
        .ok_or_else(|| McpError::Validation("Missing 'file_path'".to_string()))
}

fn path_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"file_path": {"type": "string"}},
        "required": ["file_path"],
        "additionalProperties": false
    })
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    for (name, description, kind) in [
        (
            "save_function_list",
            "Save the function list to a text file",
            ListKind::Functions,
        ),
        (
            "save_type_list",
            "Save the type list to a text file",
            ListKind::Types,
        ),
        (
            "save_file_list",
            "Save the debug file list to a text file",
            ListKind::Files,
        ),
    ] {
        server
            .add_tool(
                name.to_string(),
                Some(description.to_string()),
                path_schema(),
                SaveListHandler {
                    app_handle: app_handle.clone(),
                    kind,
                },
            )
            .await?;
    }

    server
        .add_tool(
            "save_disassembled_code".to_string(),
            Some("Save one function's disassembly by vector index".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "index": {
                        "oneOf": [
                            {"type": "integer", "minimum": 0},
                            {"type": "string", "pattern": "^[0-9]+$"}
                        ]
                    }
                },
                "required": ["file_path", "index"],
                "additionalProperties": false
            }),
            SaveDisassembledCodeHandler { app_handle },
        )
        .await?;
    Ok(())
}
