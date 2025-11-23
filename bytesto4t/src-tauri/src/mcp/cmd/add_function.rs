use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::Storage;
use hlbc::types::{Type, Function, RefType, RefFun, RefString};
use hlbc::opcodes::Opcode;

#[derive(Clone)]
pub struct AddFunctionHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct NewFunctionInput {
    pub name: String,
    pub t: usize,
    pub findex: Option<usize>,
    pub ops: Vec<Value>,
    pub regs: Vec<usize>,
    pub parent: Option<usize>,
    pub debug_info: Option<Vec<(usize, usize)>>,
    pub assigns: Option<Vec<(usize, usize)>>,
    pub is_constructor: Option<bool>,
}

#[async_trait]
impl ToolHandler for AddFunctionHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: NewFunctionInput = serde_json::from_value(serde_json::to_value(arguments)
            .map_err(|e| McpError::Validation(e.to_string()))?)
            .map_err(|e| McpError::Validation(e.to_string()))?;

        let state = self.app_handle.state::<Storage>();
        let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let bytecode = app_data.bytecode.as_mut().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;

        let mut input = input;
        if let Some(true) = input.is_constructor {
            input.name = "new".to_string();
        }

        let name_idx = input.name.parse::<usize>().map_err(|_| McpError::Validation("Function name must be a string index".to_string()))?;
        if name_idx == 0 {
            return Err(McpError::Validation("Function name index 0 is reserved".to_string()));
        }
        if name_idx >= bytecode.strings.len() {
            return Err(McpError::Validation(format!("Function name index {} is invalid", name_idx)));
        }
        let name_ref = RefString(name_idx);

        let type_idx = input.t;
        let parent_ref = input.parent.map(RefType);
        if type_idx >= bytecode.types.len() {
            return Err(McpError::Validation(format!("Function type index {} is out of bounds", type_idx)));
        }
        match &bytecode.types[type_idx] {
            Type::Fun(_) | Type::Method(_) => {}
            other => {
                return Err(McpError::Validation(format!(
                    "Invalid function type at index {}: {:?}",
                    type_idx, other
                )));
            }
        }

        let regs_vec: Vec<RefType> = input.regs.into_iter().map(RefType).collect();
        let mut ops_vec = Vec::new();
        for op_json in input.ops {
            let opcode: Opcode = serde_json::from_value(op_json)
                .map_err(|e| McpError::Validation(e.to_string()))?;
            ops_vec.push(opcode);
        }
        let assigns_converted = input.assigns.as_ref().map(|asns| {
            asns.iter().map(|(name_idx, slot_idx)| (RefString(*name_idx as usize), *slot_idx)).collect::<Vec<_>>()
        });

        let f = Function {
            t: RefType(type_idx),
            findex: RefFun(
                input.findex.unwrap_or_else(|| {
                    bytecode.functions.iter().map(|f| f.findex.0).max().unwrap_or(0) + 1
                })
            ),
            regs: regs_vec,
            ops: ops_vec,
            debug_info: input.debug_info.clone(),
            assigns: assigns_converted,
            name: name_ref,
            parent: parent_ref,
        };
        bytecode.functions.push(f);

        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "add_function".to_string(),
            Some("Add a new function (create_function)".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "t": {"type": "integer"},
                    "findex": {"type": "integer"},
                    "ops": {"type": "array"},
                    "regs": {"type": "array", "items": {"type": "integer"}},
                    "parent": {"type": "integer"},
                    "debug_info": {"type": "array"},
                    "assigns": {"type": "array"},
                    "is_constructor": {"type": "boolean"}
                },
                "required": ["name", "t", "ops", "regs"],
                "additionalProperties": false
            }),
            AddFunctionHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}