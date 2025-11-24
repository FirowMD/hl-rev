use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::app_data::{Storage, AppItem};
use hlbc::fmt::EnhancedFmt;
use hlbc::Resolve;
use hlbc::types::Type;

#[derive(Clone)]
pub struct GetDisassemblerInfoHandler {
    pub app_handle: AppHandle,
}

#[async_trait]
impl ToolHandler for GetDisassemblerInfoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let index_str = arguments
            .get("index")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'index'".to_string()))?;
        let typ_str = arguments
            .get("typ")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'typ'".to_string()))?;

        let state = self.app_handle.state::<Storage>();
        {
            let mut app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
            let app_item = AppItem { index: index_str.to_string(), typ: typ_str.to_string() };
            app_data.selected_item = Some(app_item);
        }
        let app_data = state.app_data.lock().map_err(|e| McpError::Internal(e.to_string()))?;
        let app_item = app_data.selected_item.as_ref().unwrap().clone();
        let bytecode = app_data.bytecode.as_ref().ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        let index: usize = app_item.index.parse().map_err(|_| McpError::Validation("Invalid index format".to_string()))?;

        let out = match app_item.typ.as_str() {
            "function" => {
                if index >= bytecode.functions.len() {
                    return Err(McpError::Validation("Function index out of bounds".to_string()));
                }
                let function = &bytecode.functions[index];
                format!("{}", function.display::<EnhancedFmt>(&bytecode))
            }
            "class" => {
                if index >= bytecode.types.len() {
                    return Err(McpError::Validation("Type index out of bounds".to_string()));
                }
                match &bytecode.types[index] {
                    Type::Obj(obj) => {
                        let mut s = format!("{}", (&bytecode.types[index]).display::<EnhancedFmt>(&bytecode));
                        if let Some(sup) = obj.super_ {
                            s += &format!("\nextends {}", sup.display::<EnhancedFmt>(&bytecode));
                        }
                        s += &format!("\nglobal: {}", obj.global.0);
                        s += "\nfields:";
                        for f in &obj.own_fields {
                            s += &format!("\n  {}: {}", f.name.display::<EnhancedFmt>(&bytecode), f.t.display::<EnhancedFmt>(&bytecode));
                        }
                        s += "\nprotos:";
                        for p in &obj.protos {
                            s += &format!("\n  {}: {} ({})", p.name.display::<EnhancedFmt>(&bytecode), bytecode.get(p.findex).display_header::<EnhancedFmt>(&bytecode), p.pindex);
                        }
                        s += "\nbindings:";
                        for (fi, fun) in &obj.bindings {
                            s += &format!("\n  {}: {}", fi.display::<EnhancedFmt>(&bytecode, &bytecode.types[index]), fun.display_header::<EnhancedFmt>(&bytecode));
                        }
                        s
                    }
                    Type::Enum { global, constructs, .. } => {
                        let mut s = format!("global: {}", global.0);
                        s += "\nconstructs:";
                        for c in constructs {
                            s += &format!("\n  {}:", c.name(&bytecode));
                            for (i, p) in c.params.iter().enumerate() {
                                s += &format!("\n    {i}: {}", p.display::<EnhancedFmt>(&bytecode));
                            }
                        }
                        s
                    }
                    _ => return Err(McpError::Validation("Type is not an object or enum".to_string())),
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
            "get_disassembler_info".to_string(),
            Some("Get disassembler info for item".to_string()),
            json!({
                "type": "object",
                "properties": { "index": { "type": "string" }, "typ": { "type": "string" } },
                "required": ["index", "typ"],
                "additionalProperties": false
            }),
            GetDisassemblerInfoHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
