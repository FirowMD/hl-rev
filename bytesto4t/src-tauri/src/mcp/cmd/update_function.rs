use crate::app_data::Storage;
use crate::bytecode_refs;
use crate::mcp::cmd::support;
use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefFun, RefString, RefType, Type};
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct UpdateFunctionHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateFunctionInput {
    pub index: usize,
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
impl ToolHandler for UpdateFunctionHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: UpdateFunctionInput = serde_json::from_value(
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

        if input.index >= bytecode.functions.len() {
            return Err(McpError::Validation(
                "Function index out of bounds".to_string(),
            ));
        }

        let name_idx = if input.is_constructor == Some(true) {
            support::constructor_name_index(bytecode)?
        } else {
            input.name.parse::<usize>().map_err(|_| {
                McpError::Validation("Function name must be a string index".to_string())
            })?
        };
        if name_idx == 0 {
            return Err(McpError::Validation(
                "Function name index 0 is reserved".to_string(),
            ));
        }
        if name_idx >= bytecode.strings.len() {
            return Err(McpError::Validation(format!(
                "Function name index {} is invalid",
                name_idx
            )));
        }
        let name_ref = RefString(name_idx);

        let type_idx = input.t;
        let parent_ref = input.parent.map(RefType);
        if type_idx >= bytecode.types.len() {
            return Err(McpError::Validation(format!(
                "Function type index {} is out of bounds",
                type_idx
            )));
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
            let opcode: Opcode =
                serde_json::from_value(op_json).map_err(|e| McpError::Validation(e.to_string()))?;
            ops_vec.push(opcode);
        }
        let assigns_converted = input.assigns.as_ref().map(|asns| {
            asns.iter()
                .map(|(name_idx, slot_idx)| (RefString(*name_idx), *slot_idx))
                .collect::<Vec<_>>()
        });

        let old_findex = bytecode.functions[input.index].findex;
        let findex = input.findex.unwrap_or(old_findex.0);
        support::ensure_findex_in_dense_range(bytecode, findex, false)?;
        support::ensure_findex_available(bytecode, findex, Some(input.index), None)?;
        if findex != old_findex.0 {
            return Err(McpError::Validation(
                "Changing a function findex is not supported; create a replacement function instead"
                    .to_string(),
            ));
        }

        let mut f = Function {
            t: RefType(type_idx),
            findex: RefFun(findex),
            regs: regs_vec,
            ops: ops_vec,
            debug_info: input.debug_info.clone(),
            assigns: assigns_converted,
            name: name_ref,
            parent: parent_ref,
        };
        support::normalize_function_metadata(bytecode, &mut f)?;
        bytecode_refs::validate_function_refs(bytecode, &f, "updated function", false)
            .map_err(McpError::Validation)?;
        bytecode.functions[input.index] = f;

        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "update_function".to_string(),
            Some("Update an existing function".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer"},
                    "name": {"type": "string", "description": "Decimal string-pool index; ignored when is_constructor is true"},
                    "t": {"type": "integer"},
                    "findex": {"type": "integer"},
                    "ops": {"type": "array"},
                    "regs": {"type": "array", "items": {"type": "integer"}},
                    "parent": {"type": "integer"},
                    "debug_info": {"type": "array"},
                    "assigns": {"type": "array"},
                    "is_constructor": {"type": "boolean"}
                },
                "required": ["index", "name", "t", "ops", "regs"],
                "additionalProperties": false
            }),
            UpdateFunctionHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
