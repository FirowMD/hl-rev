use crate::app_data::Storage;
use crate::bytecode_refs;
use crate::mcp::cmd::support;
use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefFun, RefString, RefType};
use std::io::BufRead;
use tauri::State;

pub(crate) fn format_function_entry(name: &str, findex: usize, index: usize) -> String {
    format!("{name}@{findex}@{index}")
}

#[tauri::command]
pub fn get_function_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_ref()
        .ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    let mut function_names = Vec::new();
    let mut index = 0;
    for function in functions {
        function_names.push(format_function_entry(
            &function.name(&bytecode).to_string(),
            function.findex.0,
            index,
        ));
        index += 1;
    }

    Ok(function_names)
}

#[derive(serde::Serialize)]
pub struct FunctionConstructorInfo {
    pub index: usize,
    pub name: String,
    pub signature: String,
    pub is_constructor: bool,
}

#[tauri::command]
pub fn list_functions_with_constructors(
    app_data: State<Storage>,
) -> Result<Vec<FunctionConstructorInfo>, String> {
    let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_ref()
        .ok_or("bytecode not loaded")?;
    let mut result = Vec::new();
    for (index, f) in bytecode.functions.iter().enumerate() {
        let fname = f.name(&bytecode).to_string();
        let is_ctor = fname == "new" || fname.to_lowercase().starts_with("ctor");
        let sig = format!("{:?}", f.ty(&bytecode));
        result.push(FunctionConstructorInfo {
            index,
            name: fname,
            signature: sig,
            is_constructor: is_ctor,
        });
    }
    Ok(result)
}

#[derive(Debug, serde::Deserialize)]
pub struct NewFunctionInput {
    pub name: String,
    pub t: usize,
    pub findex: Option<usize>,
    pub ops: Vec<serde_json::Value>,
    pub regs: Vec<usize>,
    pub parent: Option<usize>,
    pub debug_info: Option<Vec<(usize, usize)>>,
    pub assigns: Option<Vec<(usize, usize)>>,
    pub is_constructor: Option<bool>,
}

#[tauri::command]
pub fn create_function(input: NewFunctionInput, app_data: State<Storage>) -> Result<(), String> {
    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_mut()
        .ok_or("bytecode not loaded")?;

    let name_idx = if input.is_constructor == Some(true) {
        support::constructor_name_index(bytecode).map_err(|error| error.to_string())?
    } else {
        input
            .name
            .parse::<usize>()
            .map_err(|_| "Function name must be a string index")?
    };
    bytecode_refs::ensure_string_index(bytecode, name_idx, "function name", false)?;
    let name_ref = RefString(name_idx);

    let type_idx = input.t;
    let parent_ref = input.parent.map(RefType);

    bytecode_refs::ensure_function_type_index(bytecode, type_idx, "function type")?;

    let regs_vec: Vec<RefType> = input.regs.into_iter().map(RefType).collect();

    let mut ops_vec = Vec::with_capacity(input.ops.len());
    for op_json in input.ops {
        let opcode: Opcode =
            serde_json::from_value(op_json).map_err(|e| format!("Error parsing opcode: {}", e))?;
        ops_vec.push(opcode);
    }

    let assigns_converted = input.assigns.as_ref().map(|asns| {
        asns.iter()
            .map(|(name_idx, slot_idx)| (RefString(*name_idx as usize), *slot_idx))
            .collect::<Vec<_>>()
    });

    let findex = input
        .findex
        .unwrap_or(bytecode.functions.len() + bytecode.natives.len());
    support::ensure_findex_in_dense_range(bytecode, findex, true).map_err(|e| e.to_string())?;
    support::ensure_findex_available(bytecode, findex, None, None).map_err(|e| e.to_string())?;
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
    support::normalize_function_metadata(bytecode, &mut f).map_err(|e| e.to_string())?;
    bytecode_refs::validate_function_refs_with_pending_fun(
        bytecode,
        &f,
        "new function",
        false,
        Some(f.findex),
    )?;
    let mut candidate = bytecode.clone();
    candidate.functions.push(f);
    support::rebuild_runtime_indexes(&mut candidate).map_err(|e| e.to_string())?;
    *bytecode = candidate;
    Ok(())
}

#[tauri::command]
pub fn delete_function(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_mut()
        .ok_or("bytecode not loaded")?;
    if index >= bytecode.functions.len() {
        return Err(format!(
            "Function index {} out of bounds (max: {})",
            index,
            bytecode.functions.len().saturating_sub(1)
        ));
    }
    let findex = bytecode.functions[index].findex;
    let self_reference_prefix = format!("function[{}].ops[", index);
    let references = bytecode_refs::function_references(bytecode, findex)
        .into_iter()
        .filter(|reference| !reference.starts_with(&self_reference_prefix))
        .collect();
    bytecode_refs::ensure_tail_delete("Function", index, bytecode.functions.len(), references)?;
    let mut candidate = bytecode.clone();
    candidate.functions.remove(index);
    bytecode_refs::compact_function_indexes_after_delete(&mut candidate, findex);
    support::rebuild_runtime_indexes(&mut candidate).map_err(|e| e.to_string())?;
    *bytecode = candidate;
    Ok(())
}

#[tauri::command]
pub fn update_function(
    index: usize,
    input: NewFunctionInput,
    app_data: State<Storage>,
) -> Result<(), String> {
    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_mut()
        .ok_or("bytecode not loaded")?;
    if index >= bytecode.functions.len() {
        return Err(format!(
            "Function index {} out of bounds (max: {})",
            index,
            bytecode.functions.len().saturating_sub(1)
        ));
    }
    let name_idx = if input.is_constructor == Some(true) {
        support::constructor_name_index(bytecode).map_err(|error| error.to_string())?
    } else {
        input
            .name
            .parse::<usize>()
            .map_err(|_| "Function name for update_function must be a string index")?
    };
    bytecode_refs::ensure_string_index(bytecode, name_idx, "function name", false)?;
    let name_ref = RefString(name_idx);
    let type_idx = input.t;
    let parent_ref = input.parent.map(RefType);
    bytecode_refs::ensure_function_type_index(bytecode, type_idx, "function type")?;
    let regs_vec = input.regs.into_iter().map(RefType).collect();
    let mut ops_vec = Vec::with_capacity(input.ops.len());
    for op_json in input.ops {
        let opcode: Opcode =
            serde_json::from_value(op_json).map_err(|e| format!("Error parsing opcode: {}", e))?;
        ops_vec.push(opcode);
    }
    let assigns_converted = input.assigns.as_ref().map(|asns| {
        asns.iter()
            .map(|(name_idx, slot_idx)| (RefString(*name_idx as usize), *slot_idx))
            .collect::<Vec<_>>()
    });
    let old_findex = bytecode.functions[index].findex;
    let findex = input.findex.unwrap_or(old_findex.0);
    if findex != old_findex.0 {
        return Err("Changing a function findex is not supported".to_string());
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
    support::normalize_function_metadata(bytecode, &mut f).map_err(|e| e.to_string())?;
    bytecode_refs::validate_function_refs(bytecode, &f, "updated function", false)?;
    let mut candidate = bytecode.clone();
    candidate.functions[index] = f;
    support::rebuild_runtime_indexes(&mut candidate).map_err(|e| e.to_string())?;
    *bytecode = candidate;
    Ok(())
}

#[tauri::command]
pub fn get_function_full_info(
    index: usize,
    app_data: State<Storage>,
) -> Result<hlbc::types::Function, String> {
    let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_ref()
        .ok_or("bytecode not loaded")?;
    if index >= bytecode.functions.len() {
        return Err(format!(
            "Function index {} out of bounds (max: {})",
            index,
            bytecode.functions.len().saturating_sub(1)
        ));
    }
    Ok(bytecode.functions[index].clone())
}

#[tauri::command]
pub fn get_function_name_by_index(
    index: usize,
    app_data: State<Storage>,
) -> Result<String, String> {
    let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_ref()
        .ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;

    if index >= functions.len() {
        return Err("Function index out of bounds".to_string());
    }

    Ok(format_function_entry(
        &functions[index].name(&bytecode).to_string(),
        functions[index].findex.0,
        index,
    ))
}

#[tauri::command]
pub fn load_function_addresses_from_file(
    file_path: &str,
    app_data: State<Storage>,
) -> Result<(), String> {
    let mut function_addresses = Vec::new();
    let file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        function_addresses.push(line.map_err(|e| e.to_string())?);
    }

    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    bytecode_store.function_addresses = Some(function_addresses);

    Ok(())
}

#[tauri::command]
pub fn get_function_addresses(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let function_addresses = bytecode_store
        .function_addresses
        .as_ref()
        .ok_or("function_addresses not loaded")?;
    Ok(function_addresses.clone())
}
