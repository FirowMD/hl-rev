use crate::app_data::Storage;
use crate::bytecode_refs;
use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefFun, RefString, RefType};
use std::io::BufRead;
use tauri::State;

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
        function_names.push(
            function.name(&bytecode).to_string()
                + &function.findex.to_string()
                + "@"
                + &index.to_string(),
        );
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
pub fn create_function(
    mut input: NewFunctionInput,
    app_data: State<Storage>,
) -> Result<(), String> {
    let mut bytecode_store = app_data.bytecode.lock().map_err(|e| e.to_string())?;
    let bytecode = bytecode_store
        .bytecode
        .as_mut()
        .ok_or("bytecode not loaded")?;

    // Overwrite name to "new" if is_constructor is Some(true)
    if let Some(true) = input.is_constructor {
        input.name = "new".to_string();
    }

    // expect name to be string index (usize)
    let name_idx = input
        .name
        .parse::<usize>()
        .map_err(|_| "Function name must be a string index")?;
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

    let f = Function {
        t: RefType(type_idx),
        findex: RefFun(input.findex.unwrap_or_else(|| {
            bytecode
                .functions
                .iter()
                .map(|f| f.findex.0)
                .max()
                .unwrap_or(0)
                + 1
        })),
        regs: regs_vec,
        ops: ops_vec,
        debug_info: input.debug_info.clone(),
        assigns: assigns_converted,
        name: name_ref,
        parent: parent_ref,
    };
    bytecode_refs::validate_function_refs_with_pending_fun(
        bytecode,
        &f,
        "new function",
        false,
        Some(f.findex),
    )?;
    bytecode.functions.push(f);
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
    bytecode_refs::ensure_tail_delete(
        "Function",
        index,
        bytecode.functions.len(),
        bytecode_refs::function_references(bytecode, findex),
    )?;
    bytecode.functions.remove(index);
    Ok(())
}

#[tauri::command]
pub fn update_function(
    index: usize,
    mut input: NewFunctionInput,
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
    if let Some(true) = input.is_constructor {
        input.name = "new".to_string();
    }

    let name_idx = input
        .name
        .parse::<usize>()
        .map_err(|_| "Function name for update_function must be a string index")?;
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
    let f = Function {
        t: RefType(type_idx),
        findex: RefFun(input.findex.unwrap_or(bytecode.functions[index].findex.0)),
        regs: regs_vec,
        ops: ops_vec,
        debug_info: input.debug_info.clone(),
        assigns: assigns_converted,
        name: name_ref,
        parent: parent_ref,
    };
    bytecode_refs::validate_function_refs(bytecode, &f, "updated function", false)?;
    bytecode.functions[index] = f;
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

    Ok(functions[index].name(&bytecode).to_string()
        + &functions[index].findex.to_string()
        + "@"
        + &index.to_string())
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
