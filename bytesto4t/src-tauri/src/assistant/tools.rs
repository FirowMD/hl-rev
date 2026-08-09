use prism_mcp_rs::prelude::{CallToolResult, ContentBlock, ToolHandler};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::AppHandle;

const MAX_TOOL_OUTPUT_CHARS: usize = 100_000;

#[derive(Clone, Debug, Serialize)]
pub struct ToolActivity {
    pub name: String,
    pub arguments: Value,
    pub success: bool,
    pub summary: String,
}

pub fn definitions() -> Vec<Value> {
    let indexed_types = [
        "function", "class", "type", "file", "global", "constant", "string", "int", "float",
        "native", "bytes",
    ];
    vec![
        tool(
            "get_dashboard_info",
            "Return bytecode version and pool sizes for the loaded HashLink module.",
            json!({"type":"object","properties":{},"additionalProperties":false}),
        ),
        tool(
            "get_function_list",
            "List functions as name, shared function id (findex), and zero-based vector index. Use the vector index with inspection tools.",
            json!({"type":"object","properties":{},"additionalProperties":false}),
        ),
        tool(
            "get_type_list",
            "List all HashLink types with their zero-based vector indexes.",
            json!({"type":"object","properties":{},"additionalProperties":false}),
        ),
        tool(
            "get_inspector_info",
            "Inspect an indexed bytecode item and include references to it. This is the general lookup tool.",
            json!({
                "type":"object",
                "properties":{
                    "index": index_schema(),
                    "typ":{"type":"string","enum":indexed_types}
                },
                "required":["index","typ"],
                "additionalProperties":false
            }),
        ),
        tool(
            "get_decompiled_info",
            "Decompile a function or object class into Haxe-like source. The index is the zero-based vector index.",
            json!({
                "type":"object",
                "properties":{
                    "index": index_schema(),
                    "typ":{"type":"string","enum":["function","class"]}
                },
                "required":["index","typ"],
                "additionalProperties":false
            }),
        ),
        tool(
            "get_disassembler_info",
            "Return HashLink bytecode disassembly for a function, object class, or enum class.",
            json!({
                "type":"object",
                "properties":{
                    "index": index_schema(),
                    "typ":{"type":"string","enum":["function","class"]}
                },
                "required":["index","typ"],
                "additionalProperties":false
            }),
        ),
        tool(
            "get_references",
            "Return references to an indexed bytecode item.",
            json!({
                "type":"object",
                "properties":{
                    "index": index_schema(),
                    "typ":{"type":"string","enum":indexed_types}
                },
                "required":["index","typ"],
                "additionalProperties":false
            }),
        ),
        tool(
            "find_functions_using_type",
            "List functions whose signature or bytecode uses a type vector index.",
            json!({
                "type":"object",
                "properties":{"type_index":index_schema()},
                "required":["type_index"],
                "additionalProperties":false
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "name": name,
        "description": description,
        "parameters": parameters,
        "strict": false
    })
}

fn index_schema() -> Value {
    json!({
        "oneOf": [
            {"type":"integer","minimum":0},
            {"type":"string","pattern":"^[0-9]+$"}
        ]
    })
}

fn argument_map(arguments: &Value) -> Result<HashMap<String, Value>, String> {
    arguments
        .as_object()
        .cloned()
        .map(|values| values.into_iter().collect())
        .ok_or_else(|| "Tool arguments must be a JSON object.".to_string())
}

pub async fn execute(
    app_handle: AppHandle,
    name: &str,
    arguments: Value,
) -> (String, ToolActivity) {
    let result = match argument_map(&arguments) {
        Ok(arguments) => dispatch(app_handle, name, arguments)
            .await
            .map(tool_result_text),
        Err(error) => Err(error),
    };

    let (output, success) = match result {
        Ok(output) => (truncate(output), true),
        Err(error) => (format!("Tool error: {error}"), false),
    };
    let summary = output
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(180)
        .collect();
    let activity = ToolActivity {
        name: name.to_string(),
        arguments,
        success,
        summary,
    };
    (output, activity)
}

async fn dispatch(
    app_handle: AppHandle,
    name: &str,
    arguments: HashMap<String, Value>,
) -> Result<CallToolResult, String> {
    let result = match name {
        "get_dashboard_info" => {
            crate::mcp::cmd::get_dashboard_info::GetDashboardInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_function_list" => {
            crate::mcp::cmd::get_function_list::GetFunctionListHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_type_list" => {
            crate::mcp::cmd::get_type_list::GetTypeListHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_inspector_info" => {
            crate::mcp::cmd::get_inspector_info::GetInspectorInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_decompiled_info" => {
            crate::mcp::cmd::get_decompiled_info::GetDecompiledInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_disassembler_info" => {
            crate::mcp::cmd::get_disassembler_info::GetDisassemblerInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_references" => {
            crate::mcp::cmd::get_references::GetReferencesHandler { app_handle }
                .call(arguments)
                .await
        }
        "find_functions_using_type" => {
            crate::mcp::cmd::find_functions_using_type::FindFunctionsUsingTypeHandler { app_handle }
                .call(arguments)
                .await
        }
        _ => return Err(format!("Unknown assistant tool: {name}")),
    };
    result.map_err(|error| error.to_string())
}

fn tool_result_text(result: CallToolResult) -> String {
    let mut sections = Vec::new();
    if let Some(structured) = result.structured_content {
        sections.push(structured.to_string());
    }
    for block in result.content {
        if let ContentBlock::Text { text, .. } = block {
            sections.push(text);
        }
    }
    if sections.is_empty() {
        "Tool returned no text.".to_string()
    } else {
        sections.join("\n")
    }
}

fn truncate(value: String) -> String {
    if value.chars().count() <= MAX_TOOL_OUTPUT_CHARS {
        return value;
    }
    let mut output: String = value.chars().take(MAX_TOOL_OUTPUT_CHARS).collect();
    output.push_str("\n[Output truncated by bytesto4t]");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assistant_tools_are_read_only() {
        let names: Vec<_> = definitions()
            .into_iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        assert!(names.iter().all(|name| {
            !name.starts_with("create_")
                && !name.starts_with("update_")
                && !name.starts_with("delete_")
                && !name.starts_with("save_")
        }));
    }

    #[test]
    fn output_truncation_is_bounded() {
        let output = truncate("x".repeat(MAX_TOOL_OUTPUT_CHARS + 10));
        assert!(output.ends_with("[Output truncated by bytesto4t]"));
    }
}
