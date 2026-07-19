use crate::app_data::Storage;
use crate::bytecode_refs;
use crate::mcp::cmd::support;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct GetReferencesHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetReferencesHandler {
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

        let references = bytecode_refs::references_for_item(bytecode, item_type, index)
            .map_err(McpError::Validation)?;
        let output = if references.is_empty() {
            "No references found.".to_string()
        } else {
            references.join("\n")
        };
        Ok(CallToolResult::text(output))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    server
        .add_tool(
            "get_references".to_string(),
            Some("Get references to an indexed bytecode item".to_string()),
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
            GetReferencesHandler { app_handle },
        )
        .await?;
    Ok(())
}
