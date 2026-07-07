use tauri::State;
use hlbc::types::{Type, Function, RefType, RefFun, RefString};
use hlbc::opcodes::Opcode;
use std::io::BufRead;
use crate::app_data::Storage;

#[tauri::command]
pub fn get_function_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    let mut function_names = Vec::new();
    let mut index = 0;
    for function in functions {
        function_names.push(function.name(&bytecode).to_string() + &function.findex.to_string() +
            "@" + &index.to_string());
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
pub fn list_functions_with_constructors(app_data: State<Storage>) -> Result<Vec<FunctionConstructorInfo>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let mut result = Vec::new();
    for (index, f) in bytecode.functions.iter().enumerate() {
        let fname = f.name(&bytecode).to_string();
        let is_ctor = fname == "new" || fname.to_lowercase().starts_with("ctor");
        let sig = format!("{:?}", f.ty(&bytecode));
        result.push(FunctionConstructorInfo {
            index,
            name: fname,
            signature: sig,
            is_constructor: is_ctor
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
pub fn create_function(mut input: NewFunctionInput, app_data: State<Storage>) -> Result<(), String> {
    println!("DEBUG: Received create_function input: {:?}", input);
    println!(
        "name={:?}, t={:?}, findex={:?}, regs={:?}, ops(count)={:?}, parent={:?}, debug_info={:?}, assigns={:?}, is_constructor={:?}",
        input.name, input.t, input.findex, input.regs.len(), input.ops.len(), input.parent, input.debug_info, input.assigns, input.is_constructor
    );

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;

    // Overwrite name to "new" if is_constructor is Some(true)
    if let Some(true) = input.is_constructor {
        input.name = "new".to_string();
    }

    // expect name to be string index (usize)
    let name_idx = input.name.parse::<usize>().map_err(|_| "Function name must be a string index")?;
    if name_idx == 0 {
        return Err("Function name index 0 is reserved for 'no name' and cannot be used.".to_string());
    }
    if name_idx >= bytecode.strings.len() {
        return Err(format!("Function name index {} is not valid (should be in 1..{})", name_idx, bytecode.strings.len()-1));
    }
    let name_ref = RefString(name_idx);

    let type_idx = input.t;
    let parent_ref = input.parent.map(RefType);

    if type_idx >= bytecode.types.len() {
        return Err(format!(
            "Function type index {} is out of bounds (max is {}).",
            type_idx,
            bytecode.types.len().saturating_sub(1)
        ));
    }
    match &bytecode.types[type_idx] {
        Type::Fun(_) | Type::Method(_) => {
            // allowed
        }
        other => {
            return Err(format!(
                "Invalid function type at index {}: expected function signature (Type::Fun or Type::Method), but got {:?}",
                type_idx, other
            ));
        }
    }

    let regs_vec: Vec<RefType> = input.regs.into_iter().map(RefType).collect();

    let mut ops_vec = Vec::with_capacity(input.ops.len());
    for op_json in input.ops {
        let opcode: Opcode = serde_json::from_value(op_json)
            .map_err(|e| format!("Error parsing opcode: {}", e))?;
        ops_vec.push(opcode);
    }

    let assigns_converted = input.assigns.as_ref().map(|asns| {
        asns
            .iter()
            .map(|(name_idx, slot_idx)| (RefString(*name_idx as usize), *slot_idx))
            .collect::<Vec<_>>()
    });

    let f = Function {
        t: RefType(type_idx),
        findex: RefFun(
            input.findex.unwrap_or_else(|| {
                bytecode.functions
                    .iter()
                    .map(|f| f.findex.0)
                    .max()
                    .unwrap_or(0) + 1
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
    Ok(())
}

#[tauri::command]
pub fn delete_function(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    if index >= bytecode.functions.len() {
        return Err(format!("Function index {} out of bounds (max: {})", index, bytecode.functions.len().saturating_sub(1)));
    }
    bytecode.functions.remove(index);
    Ok(())
}

#[tauri::command]
pub fn update_function(index: usize, mut input: NewFunctionInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    if index >= bytecode.functions.len() {
        return Err(format!("Function index {} out of bounds (max: {})", index, bytecode.functions.len().saturating_sub(1)));
    }
    if let Some(true) = input.is_constructor {
        input.name = "new".to_string();
    }

    let name_idx = input.name.parse::<usize>().map_err(|_| "Function name for update_function must be a string index")?;
    if name_idx == 0 {
        return Err("Function name index 0 is reserved for 'no name' and cannot be used.".to_string());
    }
    if name_idx >= bytecode.strings.len() {
        return Err(format!("Function name index {} is not valid (should be in 1..{})", name_idx, bytecode.strings.len() - 1));
    }
    let name_ref = RefString(name_idx);
    let type_idx = input.t;
    let parent_ref = input.parent.map(RefType);
    if type_idx >= bytecode.types.len() {
        return Err(format!(
            "Function type index {} is out of bounds (max is {}).",
            type_idx, bytecode.types.len().saturating_sub(1)
        ));
    }
    match &bytecode.types[type_idx] {
        Type::Fun(_) | Type::Method(_) => { }
        other => {
            return Err(format!(
                "Invalid function type at index {}: expected function signature (Type::Fun or Type::Method), but got {:?}",
                type_idx, other
            ));
        }
    }
    let regs_vec = input.regs.into_iter().map(RefType).collect();
    let mut ops_vec = Vec::with_capacity(input.ops.len());
    for op_json in input.ops {
        let opcode: Opcode = serde_json::from_value(op_json)
            .map_err(|e| format!("Error parsing opcode: {}", e))?;
        ops_vec.push(opcode);
    }
    let assigns_converted = input.assigns.as_ref().map(|asns| {
        asns
            .iter()
            .map(|(name_idx, slot_idx)| (RefString(*name_idx as usize), *slot_idx))
            .collect::<Vec<_>>()
    });
    let f = Function {
        t: RefType(type_idx),
        findex: RefFun(input.findex.unwrap_or(0)),
        regs: regs_vec,
        ops: ops_vec,
        debug_info: input.debug_info.clone(),
        assigns: assigns_converted,
        name: name_ref,
        parent: parent_ref,
    };
    bytecode.functions[index] = f;
    Ok(())
}

#[tauri::command]
pub fn get_function_full_info(index: usize, app_data: State<Storage>) -> Result<hlbc::types::Function, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    if index >= bytecode.functions.len() {
        return Err(format!("Function index {} out of bounds (max: {})", index, bytecode.functions.len().saturating_sub(1)));
    }
    Ok(bytecode.functions[index].clone())
}

#[tauri::command]
pub fn get_function_name_by_index(index: usize, app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let functions = &bytecode.functions;
    
    if index >= functions.len() {
        return Err("Function index out of bounds".to_string());
    }

    Ok(functions[index].name(&bytecode).to_string() + &functions[index].findex.to_string() +
    "@" + &index.to_string())
}

#[tauri::command]
pub fn load_function_addresses_from_file(file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut function_addresses = Vec::new();
    let file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        function_addresses.push(line.map_err(|e| e.to_string())?);
    }

    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    app_data.function_addresses = Some(function_addresses);

    Ok(())
}

#[tauri::command]
pub fn get_function_addresses(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let function_addresses = app_data.function_addresses.as_ref().ok_or("function_addresses not loaded")?;
    Ok(function_addresses.clone())
}