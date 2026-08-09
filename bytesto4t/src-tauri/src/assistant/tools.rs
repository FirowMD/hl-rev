use crate::app_data::Storage;
use prism_mcp_rs::prelude::{CallToolResult, ContentBlock, McpError, McpResult, ToolHandler};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

const MAX_TOOL_OUTPUT_CHARS: usize = 100_000;

#[derive(Clone, Debug, Serialize)]
pub struct ToolActivity {
    pub name: String,
    pub arguments: Value,
    pub success: bool,
    pub summary: String,
}

pub fn definitions(allow_bytecode_edits: bool) -> Vec<Value> {
    let indexed_types = [
        "function", "class", "type", "file", "global", "constant", "string", "int", "float",
        "native", "bytes",
    ];
    let mut definitions = vec![
        tool(
            "get_dashboard_info",
            "Return bytecode version and pool sizes for the loaded HashLink module.",
            empty_schema(),
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
                    "index": legacy_index_schema(),
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
                    "index": legacy_index_schema(),
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
                    "index": legacy_index_schema(),
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
                    "index": legacy_index_schema(),
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
                "properties":{"type_index":legacy_index_schema()},
                "required":["type_index"],
                "additionalProperties":false
            }),
        ),
    ];

    definitions.extend([
        indexed_tool(
            "get_function_full_info",
            "Return lossless function JSON. For update_function, pass numeric name as a decimal string and preserve the remaining fields.",
        ),
        indexed_tool(
            "get_type_full_info",
            "Return lossless externally tagged type JSON. Translate it to update_type's type_kind schema; object own_fields map to fields and numeric names become decimal strings.",
        ),
        indexed_tool(
            "get_string_full_info",
            "Return one string-pool entry by vector index.",
        ),
        indexed_tool(
            "get_int_full_info",
            "Return one integer-pool entry by vector index.",
        ),
        indexed_tool(
            "get_float_full_info",
            "Return one float-pool entry by vector index.",
        ),
        indexed_tool(
            "get_bytes_full_info",
            "Return one v5 bytes-pool entry by vector index.",
        ),
        indexed_tool(
            "get_global_full_info",
            "Return one global-pool entry; the returned index value is update_global's global_type.",
        ),
        indexed_tool(
            "get_constant_full_info",
            "Return one constant-table entry by vector index.",
        ),
        indexed_tool(
            "get_native_full_info",
            "Return native JSON. For update_native, t maps to signature_type and numeric lib/name values become decimal strings.",
        ),
    ]);

    definitions.push(tool(
        "validate_bytecode",
        "Rebuild and validate the loaded in-memory bytecode without changing or saving it.",
        empty_schema(),
    ));
    if allow_bytecode_edits {
        definitions.extend(function_tools());
        definitions.extend(type_tools());
        definitions.extend(pool_tools());
        definitions.push(tool(
            "save_bytecode",
            "Open a native save dialog, then validate and serialize the loaded bytecode to the path selected by the user.",
            empty_schema(),
        ));
    }
    definitions
}

fn empty_schema() -> Value {
    json!({"type":"object","properties":{},"additionalProperties":false})
}

fn indexed_tool(name: &str, description: &str) -> Value {
    tool(
        name,
        description,
        json!({
            "type":"object",
            "properties":{"index":index_schema()},
            "required":["index"],
            "additionalProperties":false
        }),
    )
}

fn function_tools() -> Vec<Value> {
    vec![
        tool(
            "create_function",
            "Append a function. Opcode JSON uses the externally tagged hlbc::Opcode representation.",
            function_schema(false),
        ),
        tool(
            "update_function",
            "Replace a complete function at a vector index. Read it first; ops replaces the whole opcode array and control-flow offsets are not repaired.",
            function_schema(true),
        ),
        indexed_tool(
            "delete_function",
            "Delete an unreferenced tail function and compact shared function identifiers.",
        ),
    ]
}

fn function_schema(updating: bool) -> Value {
    let mut properties = serde_json::Map::from_iter([
        (
            "name".to_string(),
            json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal string-pool index; ignored when is_constructor is true"}),
        ),
        ("t".to_string(), index_schema()),
        ("findex".to_string(), index_schema()),
        (
            "ops".to_string(),
            json!({"type":"array","items":{"type":"object"}}),
        ),
        (
            "regs".to_string(),
            json!({"type":"array","items":index_schema()}),
        ),
        ("parent".to_string(), index_schema()),
        ("debug_info".to_string(), json!({"type":"array"})),
        ("assigns".to_string(), json!({"type":"array"})),
        ("is_constructor".to_string(), json!({"type":"boolean"})),
    ]);
    let mut required = vec![json!("name"), json!("t"), json!("ops"), json!("regs")];
    if updating {
        properties.insert("index".to_string(), index_schema());
        required.insert(0, json!("index"));
    }
    json!({
        "type":"object",
        "properties":properties,
        "required":required,
        "additionalProperties":false
    })
}

fn type_tools() -> Vec<Value> {
    vec![
        tool(
            "create_type",
            "Append a HashLink type. Names in compound types are decimal string-pool indexes.",
            type_schema(false),
        ),
        tool(
            "update_type",
            "Replace a complete type at a vector index. Read the type first and preserve every required compound-type member.",
            type_schema(true),
        ),
        indexed_tool("delete_type", "Delete an unreferenced tail type."),
    ]
}

fn type_schema(updating: bool) -> Value {
    let field = json!({
        "type":"object",
        "properties":{"name":{"type":"string","pattern":"^[0-9]+$"},"field_type":index_schema()},
        "required":["name","field_type"],
        "additionalProperties":false
    });
    let proto = json!({
        "type":"object",
        "properties":{"name":{"type":"string","pattern":"^[0-9]+$"},"findex":index_schema(),"pindex":{"type":"integer"}},
        "required":["name","findex","pindex"],
        "additionalProperties":false
    });
    let construct = json!({
        "type":"object",
        "properties":{"name":{"type":"string","pattern":"^[0-9]+$"},"params":{"type":"array","items":index_schema()}},
        "required":["name","params"],
        "additionalProperties":false
    });
    let binding = json!({
        "type":"object",
        "properties":{"field":index_schema(),"findex":index_schema()},
        "required":["field","findex"],
        "additionalProperties":false
    });
    let mut properties = serde_json::Map::from_iter([
        (
            "type_kind".to_string(),
            json!({"type":"string","enum":["void","ui8","ui16","i32","i64","f32","f64","bool","bytes","dyn","fun","obj","array","type","ref","virtual","dynobj","abstract","enum","null","method","struct","packed","guid"]}),
        ),
        (
            "name".to_string(),
            json!({"type":"string","pattern":"^[0-9]+$"}),
        ),
        ("super_type".to_string(), index_schema()),
        ("global".to_string(), index_schema()),
        ("inner_type".to_string(), index_schema()),
        (
            "args".to_string(),
            json!({"type":"array","items":index_schema()}),
        ),
        ("ret".to_string(), index_schema()),
        ("fields".to_string(), json!({"type":"array","items":field})),
        ("protos".to_string(), json!({"type":"array","items":proto})),
        (
            "constructs".to_string(),
            json!({"type":"array","items":construct}),
        ),
        (
            "bindings".to_string(),
            json!({"type":"array","items":binding}),
        ),
    ]);
    let mut required = vec![json!("type_kind")];
    if updating {
        properties.insert("index".to_string(), index_schema());
        required.insert(0, json!("index"));
    }
    json!({
        "type":"object",
        "properties":properties,
        "required":required,
        "additionalProperties":false
    })
}

fn pool_tools() -> Vec<Value> {
    let index_value_schema = |value_schema: Value| {
        json!({
            "type":"object",
            "properties":{"index":index_schema(),"value":value_schema},
            "required":["index","value"],
            "additionalProperties":false
        })
    };
    let create_value_schema = |value_schema: Value| {
        json!({
            "type":"object",
            "properties":{"value":value_schema},
            "required":["value"],
            "additionalProperties":false
        })
    };
    let bytes_schema = |updating: bool| {
        let mut properties = serde_json::Map::from_iter([(
            "data".to_string(),
            json!({"type":"array","items":{"type":"integer","minimum":0,"maximum":255}}),
        )]);
        let mut required = vec![json!("data")];
        if updating {
            properties.insert("index".to_string(), index_schema());
            required.insert(0, json!("index"));
        }
        json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    };
    let native_schema = |updating: bool| {
        let mut properties = serde_json::Map::from_iter([
            (
                "lib".to_string(),
                json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal string-pool index"}),
            ),
            (
                "name".to_string(),
                json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal string-pool index"}),
            ),
            ("signature_type".to_string(), index_schema()),
            ("findex".to_string(), index_schema()),
        ]);
        let mut required = vec![json!("lib"), json!("name"), json!("signature_type")];
        if updating {
            properties.insert("index".to_string(), index_schema());
            required.insert(0, json!("index"));
        }
        json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    };
    let global_schema = |updating: bool| {
        let mut properties =
            serde_json::Map::from_iter([("global_type".to_string(), index_schema())]);
        let mut required = vec![json!("global_type")];
        if updating {
            properties.insert("index".to_string(), index_schema());
            required.insert(0, json!("index"));
        }
        json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    };
    let constant_schema = |updating: bool| {
        let mut properties = serde_json::Map::from_iter([
            ("global".to_string(), index_schema()),
            (
                "fields".to_string(),
                json!({"type":"array","items":index_schema()}),
            ),
        ]);
        let mut required = vec![json!("global"), json!("fields")];
        if updating {
            properties.insert("index".to_string(), index_schema());
            required.insert(0, json!("index"));
        }
        json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    };

    vec![
        tool("create_string", "Append a string-pool value.", create_value_schema(json!({"type":"string"}))),
        tool("update_string", "Replace a string-pool value. Check references first because strings are commonly shared.", index_value_schema(json!({"type":"string"}))),
        indexed_tool("delete_string", "Delete an unreferenced tail string."),
        tool("create_int", "Append a signed 32-bit integer-pool value.", create_value_schema(json!({"type":"integer","minimum":-2147483648_i64,"maximum":2147483647_i64}))),
        tool("update_int", "Replace a signed 32-bit integer-pool value. Check references first because pool values can be shared.", index_value_schema(json!({"type":"integer","minimum":-2147483648_i64,"maximum":2147483647_i64}))),
        indexed_tool("delete_int", "Delete an unreferenced tail integer."),
        tool("create_float", "Append a float-pool value.", create_value_schema(json!({"type":"number"}))),
        tool("update_float", "Replace a float-pool value. Check references first because pool values can be shared.", index_value_schema(json!({"type":"number"}))),
        indexed_tool("delete_float", "Delete an unreferenced tail float."),
        tool("create_bytes", "Append one entry to a v5 bytes pool.", bytes_schema(false)),
        tool("update_bytes", "Replace one entry in a v5 bytes pool.", bytes_schema(true)),
        indexed_tool("delete_bytes", "Delete an unreferenced tail v5 bytes entry."),
        tool("create_global", "Append a global with a type vector index.", global_schema(false)),
        tool("update_global", "Replace a global's type vector index.", global_schema(true)),
        indexed_tool("delete_global", "Delete an unreferenced tail global."),
        tool("create_constant", "Append a v4+ constant definition.", constant_schema(false)),
        tool("update_constant", "Replace a complete v4+ constant definition.", constant_schema(true)),
        indexed_tool("delete_constant", "Delete a v4+ constant definition."),
        tool("create_native", "Append a native function declaration.", native_schema(false)),
        tool("update_native", "Replace a complete native declaration; its shared findex cannot change.", native_schema(true)),
        indexed_tool("delete_native", "Delete an unreferenced tail native and compact shared function identifiers."),
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
    json!({"type":"integer","minimum":0})
}

fn legacy_index_schema() -> Value {
    json!({
        "oneOf": [
            {"type":"integer","minimum":0},
            {"type":"string","pattern":"^[0-9]+$"}
        ]
    })
}

pub fn is_mutating(name: &str) -> bool {
    name.starts_with("create_")
        || name.starts_with("update_")
        || name.starts_with("delete_")
        || name == "save_bytecode"
}

pub fn changes_loaded_bytecode(name: &str) -> bool {
    name.starts_with("create_") || name.starts_with("update_") || name.starts_with("delete_")
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
    allow_bytecode_edits: bool,
) -> (String, ToolActivity) {
    let result = if is_mutating(name) && !allow_bytecode_edits {
        Err("Bytecode editing is disabled in Assistant settings.".to_string())
    } else {
        match argument_map(&arguments) {
            Ok(arguments) => dispatch(app_handle, name, arguments)
                .await
                .map(tool_result_text),
            Err(error) => Err(error),
        }
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
        "get_function_full_info" => {
            crate::mcp::cmd::get_function_full_info::GetFunctionFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_type_full_info" => {
            crate::mcp::cmd::get_type_full_info::GetTypeFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_string_full_info" => {
            crate::mcp::cmd::get_string_full_info::GetStringFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_int_full_info" => {
            crate::mcp::cmd::get_int_full_info::GetIntFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_float_full_info" => {
            crate::mcp::cmd::get_float_full_info::GetFloatFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_bytes_full_info" => {
            crate::mcp::cmd::get_bytes_full_info::GetBytesFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_global_full_info" => {
            crate::mcp::cmd::get_global_full_info::GetGlobalFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_constant_full_info" => {
            crate::mcp::cmd::get_constant_full_info::GetConstantFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "get_native_full_info" => {
            crate::mcp::cmd::get_native_full_info::GetNativeFullInfoHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_function" => {
            crate::mcp::cmd::add_function::AddFunctionHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_function" => {
            crate::mcp::cmd::update_function::UpdateFunctionHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_function" => {
            crate::mcp::cmd::remove_function::RemoveFunctionHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_type" => {
            crate::mcp::cmd::add_type::AddTypeHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_type" => {
            crate::mcp::cmd::update_type::UpdateTypeHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_type" => {
            crate::mcp::cmd::remove_type::RemoveTypeHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_string" => {
            crate::mcp::cmd::add_string::AddStringHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_string" => {
            crate::mcp::cmd::update_string::UpdateStringHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_string" => {
            crate::mcp::cmd::remove_string::RemoveStringHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_int" => {
            crate::mcp::cmd::add_int::AddIntHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_int" => {
            crate::mcp::cmd::update_int::UpdateIntHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_int" => {
            crate::mcp::cmd::remove_int::RemoveIntHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_float" => {
            crate::mcp::cmd::add_float::AddFloatHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_float" => {
            crate::mcp::cmd::update_float::UpdateFloatHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_float" => {
            crate::mcp::cmd::remove_float::RemoveFloatHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_bytes" => {
            crate::mcp::cmd::add_bytes::AddBytesHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_bytes" => {
            crate::mcp::cmd::update_bytes::UpdateBytesHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_bytes" => {
            crate::mcp::cmd::remove_bytes::RemoveBytesHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_global" => {
            crate::mcp::cmd::add_global::AddGlobalHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_global" => {
            crate::mcp::cmd::update_global::UpdateGlobalHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_global" => {
            crate::mcp::cmd::remove_global::RemoveGlobalHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_constant" => {
            crate::mcp::cmd::add_constant::AddConstantHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_constant" => {
            crate::mcp::cmd::update_constant::UpdateConstantHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_constant" => {
            crate::mcp::cmd::remove_constant::RemoveConstantHandler { app_handle }
                .call(arguments)
                .await
        }
        "create_native" => {
            crate::mcp::cmd::add_native::AddNativeHandler { app_handle }
                .call(arguments)
                .await
        }
        "update_native" => {
            crate::mcp::cmd::update_native::UpdateNativeHandler { app_handle }
                .call(arguments)
                .await
        }
        "delete_native" => {
            crate::mcp::cmd::remove_native::RemoveNativeHandler { app_handle }
                .call(arguments)
                .await
        }
        "validate_bytecode" => validate_bytecode(&app_handle),
        "save_bytecode" => save_bytecode_with_dialog(&app_handle).await,
        _ => return Err(format!("Unknown assistant tool: {name}")),
    };
    result.map_err(|error| error.to_string())
}

fn validate_bytecode(app_handle: &AppHandle) -> McpResult<CallToolResult> {
    let state = app_handle.state::<Storage>();
    let app_data = state
        .bytecode
        .lock()
        .map_err(|error| McpError::Internal(error.to_string()))?;
    let mut candidate = app_data
        .bytecode
        .as_ref()
        .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?
        .clone();
    candidate
        .rebuild_derived_data()
        .map_err(|error| McpError::Validation(error.to_string()))?;
    candidate
        .validate()
        .map_err(|error| McpError::Validation(error.to_string()))?;
    Ok(CallToolResult::text("ok"))
}

async fn save_bytecode_with_dialog(app_handle: &AppHandle) -> McpResult<CallToolResult> {
    let target_path = {
        let state = app_handle.state::<Storage>();
        let app_data = state
            .bytecode
            .lock()
            .map_err(|error| McpError::Internal(error.to_string()))?;
        app_data
            .bytecode
            .as_ref()
            .ok_or_else(|| McpError::Validation("bytecode not loaded".to_string()))?;
        app_data.target_file_path.clone()
    };
    let target = Path::new(&target_path);
    let mut dialog = app_handle
        .dialog()
        .file()
        .set_title("Save patched HashLink bytecode");
    if let Some(parent) = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        dialog = dialog.set_directory(parent);
    }
    if let Some(file_name) = patched_file_name(target) {
        dialog = dialog.set_file_name(file_name);
    }

    let (sender, receiver) = tokio::sync::oneshot::channel();
    dialog.save_file(move |selection| {
        let _ = sender.send(selection);
    });
    let selection = receiver
        .await
        .map_err(|_| McpError::Internal("Save dialog closed unexpectedly".to_string()))?
        .ok_or_else(|| McpError::Validation("Save cancelled by user".to_string()))?;
    let path = selection
        .into_path()
        .map_err(|error| McpError::Validation(error.to_string()))?;
    let file_path = path
        .to_str()
        .ok_or_else(|| McpError::Validation("Selected save path is not valid UTF-8".to_string()))?;
    let arguments = HashMap::from([("file_path".to_string(), json!(file_path))]);
    crate::mcp::cmd::save_bytecode::SaveBytecodeHandler {
        app_handle: app_handle.clone(),
    }
    .call(arguments)
    .await
}

fn patched_file_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    Some(
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) => format!("{stem}.patched.{extension}"),
            None => format!("{stem}.patched"),
        },
    )
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
    use std::collections::HashSet;

    #[test]
    fn assistant_exposes_a_complete_patch_workflow() {
        let names: Vec<_> = definitions(true)
            .into_iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        for required in [
            "get_function_full_info",
            "update_function",
            "create_string",
            "create_int",
            "create_float",
            "validate_bytecode",
            "save_bytecode",
        ] {
            assert!(
                names.iter().any(|name| name == required),
                "missing {required}"
            );
        }
        assert_eq!(
            names.len(),
            names.iter().collect::<HashSet<_>>().len(),
            "assistant tool names must be unique"
        );
    }

    #[test]
    fn every_bytecode_mutation_tool_is_classified() {
        for name in definitions(true)
            .into_iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
        {
            if name.starts_with("create_")
                || name.starts_with("update_")
                || name.starts_with("delete_")
            {
                assert!(is_mutating(&name));
                assert!(changes_loaded_bytecode(&name));
            }
        }
        assert!(is_mutating("save_bytecode"));
        assert!(!changes_loaded_bytecode("save_bytecode"));
    }

    #[test]
    fn mutation_tools_are_hidden_when_editing_is_disabled() {
        assert!(definitions(false).into_iter().all(|tool| tool
            .get("name")
            .and_then(Value::as_str)
            .is_none_or(|name| !is_mutating(name))));
    }

    #[test]
    fn patched_save_name_preserves_the_extension() {
        assert_eq!(
            patched_file_name(Path::new("game.hl")).as_deref(),
            Some("game.patched.hl")
        );
        assert_eq!(
            patched_file_name(Path::new("hlboot.dat")).as_deref(),
            Some("hlboot.patched.dat")
        );
    }

    #[test]
    fn output_truncation_is_bounded() {
        let output = truncate("x".repeat(MAX_TOOL_OUTPUT_CHARS + 10));
        assert!(output.ends_with("[Output truncated by bytesto4t]"));
    }
}
