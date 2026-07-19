use hlbc::types::Function;
use hlbc::Bytecode;
use prism_mcp_rs::prelude::{McpError, McpResult};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;

pub fn required_index(arguments: &HashMap<String, Value>, key: &str) -> McpResult<usize> {
    let value = arguments
        .get(key)
        .ok_or_else(|| McpError::Validation(format!("Missing '{}'", key)))?;

    let parsed = match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok()),
        Value::String(value) => value.parse::<usize>().ok(),
        _ => None,
    };

    parsed.ok_or_else(|| McpError::Validation(format!("'{}' must be a non-negative integer", key)))
}

pub fn next_findex(bytecode: &Bytecode) -> McpResult<usize> {
    let maximum = bytecode
        .functions
        .iter()
        .map(|function| function.findex.0)
        .chain(bytecode.natives.iter().map(|native| native.findex.0))
        .max();
    match maximum {
        Some(maximum) => maximum
            .checked_add(1)
            .ok_or_else(|| McpError::Validation("No function index is available".to_string())),
        None => Ok(0),
    }
}

pub fn ensure_findex_available(
    bytecode: &Bytecode,
    findex: usize,
    except_function: Option<usize>,
    except_native: Option<usize>,
) -> McpResult<()> {
    let function_conflict = bytecode
        .functions
        .iter()
        .enumerate()
        .any(|(index, function)| Some(index) != except_function && function.findex.0 == findex);
    let native_conflict = bytecode
        .natives
        .iter()
        .enumerate()
        .any(|(index, native)| Some(index) != except_native && native.findex.0 == findex);

    if function_conflict || native_conflict {
        Err(McpError::Validation(format!(
            "Function index {} is already in use",
            findex
        )))
    } else {
        Ok(())
    }
}

pub fn ensure_findex_in_dense_range(
    bytecode: &Bytecode,
    findex: usize,
    adding_entry: bool,
) -> McpResult<()> {
    let current_len = bytecode.functions.len() + bytecode.natives.len();
    let upper_bound = if adding_entry {
        current_len
    } else {
        current_len.saturating_sub(1)
    };
    if findex <= upper_bound {
        Ok(())
    } else {
        Err(McpError::Validation(format!(
            "Function index {} is outside the dense range 0..={}",
            findex, upper_bound
        )))
    }
}

pub fn constructor_name_index(bytecode: &Bytecode) -> McpResult<usize> {
    bytecode
        .strings
        .iter()
        .position(|value| value.as_str() == "new")
        .ok_or_else(|| {
            McpError::Validation(
                "Constructor name 'new' is missing from the string pool; create it first"
                    .to_string(),
            )
        })
}

pub fn normalize_function_metadata(bytecode: &Bytecode, function: &mut Function) -> McpResult<()> {
    if let Some(debug_files) = &bytecode.debug_files {
        match &function.debug_info {
            Some(debug_info) if debug_info.len() != function.ops.len() => {
                return Err(McpError::Validation(format!(
                    "debug_info has {} entries but the function has {} opcodes",
                    debug_info.len(),
                    function.ops.len()
                )))
            }
            None if !function.ops.is_empty() && debug_files.is_empty() => {
                return Err(McpError::Validation(
                    "Cannot synthesize debug info because the debug file pool is empty".to_string(),
                ))
            }
            None => {
                function.debug_info = Some(vec![(0, 0); function.ops.len()]);
            }
            Some(_) => {}
        }
        function.assigns.get_or_insert_with(Vec::new);
    } else {
        if function
            .debug_info
            .as_ref()
            .is_some_and(|debug_info| !debug_info.is_empty())
            || function
                .assigns
                .as_ref()
                .is_some_and(|assigns| !assigns.is_empty())
        {
            return Err(McpError::Validation(
                "debug_info and assigns require a bytecode debug section".to_string(),
            ));
        }
        function.debug_info = None;
        function.assigns = None;
    }
    Ok(())
}

pub fn rebuild_runtime_indexes(bytecode: &mut Bytecode) -> McpResult<()> {
    let bytecode_to_rebuild = bytecode.clone();
    let worker = std::thread::Builder::new()
        .name("bytesto4t-bytecode-rebuild".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || -> hlbc::Result<Bytecode> {
            let function_metadata = bytecode_to_rebuild
                .functions
                .iter()
                .map(|function| (function.findex, (function.name, function.parent)))
                .collect::<HashMap<_, _>>();
            let mut encoded = Vec::new();
            bytecode_to_rebuild.serialize(&mut encoded)?;
            let mut rebuilt = Bytecode::deserialize(Cursor::new(encoded))?;
            for function in &mut rebuilt.functions {
                if let Some((name, parent)) = function_metadata.get(&function.findex) {
                    function.name = *name;
                    function.parent = *parent;
                }
            }
            Ok(rebuilt)
        })
        .map_err(|error| McpError::Internal(format!("Failed to start rebuild worker: {error}")))?;
    let rebuilt = worker
        .join()
        .map_err(|_| McpError::Internal("Bytecode rebuild worker panicked".to_string()))?
        .map_err(|error| McpError::Internal(error.to_string()))?;
    *bytecode = rebuilt;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::required_index;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    #[test]
    fn required_index_accepts_integer_and_legacy_string() {
        let mut arguments = HashMap::<String, Value>::new();
        arguments.insert("index".to_string(), json!(42));
        assert_eq!(required_index(&arguments, "index").unwrap(), 42);

        arguments.insert("index".to_string(), json!("17"));
        assert_eq!(required_index(&arguments, "index").unwrap(), 17);
    }

    #[test]
    fn required_index_rejects_negative_and_fractional_values() {
        let mut arguments = HashMap::<String, Value>::new();
        arguments.insert("index".to_string(), json!(-1));
        assert!(required_index(&arguments, "index").is_err());

        arguments.insert("index".to_string(), json!(1.5));
        assert!(required_index(&arguments, "index").is_err());
    }
}
