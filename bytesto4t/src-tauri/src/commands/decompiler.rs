use tauri::State;
use hlbc::Resolve;
use hlbc::fmt::EnhancedFmt;
use hlbc::types::Type;
use hlbc::opcodes::Opcode;
use hlbc_decompiler::{decompile_function, decompile_class};
use crate::app_data::Storage;

#[tauri::command]
pub fn get_decompiled_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;

    match app_item.typ.as_str() {
        "function" => {
            let functions = &bytecode.functions;
            if index >= functions.len() {
                return Err("Function index out of bounds".to_string());
            }
            let function = &functions[index];
            let decompiled = decompile_function(&bytecode, &function);
            Ok(format!("{}", decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))))
        }
        "class" => {
            let types = &bytecode.types;
            if index >= types.len() {
                return Err("Type index out of bounds".to_string());
            }
            let type_obj = &types[index];
            match type_obj {
                Type::Obj(obj) => {
                    let decompiled = decompile_class(&bytecode, obj);
                    Ok(format!("{}", decompiled.display(&bytecode, &hlbc_decompiler::fmt::FormatOptions::new(2))))
                }
                _ => Err("Type is not an object".to_string()),
            }
        }
        _ => Err(format!("Unsupported item type: {}", app_item.typ)),
    }
}

#[tauri::command]
pub fn get_inspector_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;
    let item_type = &app_item.typ;

    let info = match item_type.as_str() {
        "function" => {
            let functions = &bytecode.functions;
            if index >= functions.len() {
                return Err("Function index out of bounds".to_string());
            }
            let f = &functions[index];
            let mut info = format!("{}{}\n", 
                f.name.display::<EnhancedFmt>(&bytecode), 
                f.findex);
            
            let findex = f.findex;
            info.push_str("\n\nReferences:");
            
            functions
                .iter()
                .enumerate()
                .flat_map(|(i, f)| std::iter::repeat((i, f)).zip(f.find_fun_refs()))
                .for_each(|((_src_idx, f), (op_idx, op, fun))| {
                    if fun.0 == findex.0 {
                        info.push_str(&format!(
                            "\n{} at {}: {}",
                            f.display_header::<EnhancedFmt>(&bytecode),
                            op_idx,
                            op.name()
                        ));
                    }
                });
            
            info
        }
        "class" => {
            let types = &bytecode.types;
            if index >= types.len() {
                return Err("Type index out of bounds".to_string());
            }
            let type_obj = &types[index];
            let info = match type_obj {
                Type::Fun(_fun) => format!("{}\n", 
                    type_obj.display::<EnhancedFmt>(&bytecode)),
                Type::Obj(_obj) => format!("{}\n", 
                    type_obj.display::<EnhancedFmt>(&bytecode)),
                _ => format!("{}", type_obj.display::<EnhancedFmt>(&bytecode))
            };
            
            info
        }
        "file" => {
            let debug_files = bytecode.debug_files.as_ref().ok_or("debug_files not loaded")?;
            if index >= debug_files.len() {
                return Err("File index out of bounds".to_string());
            }
            let mut info = format!("File: {}\n\nFunctions:", debug_files[index]);
            
            for (_i, f) in bytecode.functions.iter().enumerate() {
                if let Some(debug_info) = f.debug_info.as_ref() {
                    if let Some(last_file_idx) = debug_info.last().map(|(file_idx, _)| file_idx) {
                        if *last_file_idx == index {
                            info.push_str(&format!(
                                "\n{}", 
                                f.display_header::<EnhancedFmt>(&bytecode)
                            ));
                        }
                    }
                }
            }
            
            info
        }
        "global" => {
            let globals = &bytecode.globals;
            if index >= globals.len() {
                return Err("Global index out of bounds".to_string());
            }
            let mut info = format!("{}", globals[index].display::<EnhancedFmt>(&bytecode));
            
            info.push_str("\n\nReferences:");
            
            if let Some(constants) = &bytecode.constants {
                for (i, c) in constants.iter().enumerate() {
                    if c.global.0 == index {
                        info.push_str(&format!("\nConstant@{}: {:?}", i, c));
                    }
                }
            }
            
            bytecode
                .functions
                .iter()
                .flat_map(|f| {
                    f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op))
                })
                .for_each(|(f, op_idx, op)| match op {
                    Opcode::GetGlobal { global, .. } | Opcode::SetGlobal { global, .. } => {
                        if global.0 == index {
                            info.push_str(&format!(
                                "\n{} at {}: {}",
                                f.display_header::<EnhancedFmt>(&bytecode),
                                op_idx,
                                op.name()
                            ));
                        }
                    }
                    _ => {}
                });
            
            info
        }
        "constant" => {
            let constants = bytecode.constants.as_ref().ok_or("constants not loaded")?;
            if index >= constants.len() {
                return Err("Constant index out of bounds".to_string());
            }
            format!("{:?}", constants[index])
        }
        "string" => {
            let strings = &bytecode.strings;
            if index >= strings.len() {
                return Err("String index out of bounds".to_string());
            }
            let mut info = format!("String@{}: {:?}", index, strings[index]);
            
            info.push_str("\n\nReferences:");
            
            bytecode
                .functions
                .iter()
                .flat_map(|f| {
                    f.ops.iter().enumerate().map(move |(op_idx, op)| (f, op_idx, op))
                })
                .for_each(|(f, op_idx, op)| match op {
                    Opcode::String { ptr, .. } => {
                        if ptr.0 == index {
                            info.push_str(&format!(
                                "\n{} at {}: {}",
                                f.display_header::<EnhancedFmt>(&bytecode),
                                op_idx,
                                op.name()
                            ));
                        }
                    }
                    _ => {}
                });
            
            info
        }
        "int" => {
            let ints = &bytecode.ints;
            if index >= ints.len() {
                return Err("Int index out of bounds".to_string());
            }
            let info = format!("Int@{}: {}", index, ints[index]);
            info
        }
        "float" => {
            let floats = &bytecode.floats;
            if index >= floats.len() {
                return Err("Float index out of bounds".to_string());
            }
            let info = format!("Float@{}: {}", index, floats[index]);
            info
        }
        "native" => {
            let natives = &bytecode.natives;
            if index >= natives.len() {
                return Err("Native index out of bounds".to_string());
            }
            let native = &natives[index];
            let info = format!("{}\n", 
                native.display::<EnhancedFmt>(&bytecode));
            
            info
        }
        "bytes" => {
            match &bytecode.bytes {
                Some((bytes_data, indices)) => {
                    if index >= indices.len() {
                        return Err("Bytes index out of bounds".to_string());
                    }
                    
                    let start_pos = indices[index];
                    let end_pos = indices.get(index + 1).copied().unwrap_or(bytes_data.len());
                    let byte_slice = &bytes_data[start_pos..end_pos];
                    
                    let mut info = format!("Bytes@{}: {} bytes\n\n", index, byte_slice.len());
                    
                    for (line_idx, chunk) in byte_slice.chunks(16).enumerate() {
                        let offset = line_idx * 16;
                        info.push_str(&format!("{:08x}  ", offset));
                        
                        for (i, byte) in chunk.iter().enumerate() {
                            if i == 8 {
                                info.push(' ');
                            }
                            info.push_str(&format!("{:02x} ", byte));
                        }
                        
                        let remaining = 16 - chunk.len();
                        for i in 0..remaining {
                            if chunk.len() + i == 8 {
                                info.push(' ');
                            }
                            info.push_str("   ");
                        }
                        
                        info.push_str(" |");
                        
                        for byte in chunk {
                            if *byte >= 32 && *byte <= 126 {
                                info.push(*byte as char);
                            } else {
                                info.push('.');
                            }
                        }
                        
                        info.push_str("|\n");
                    }
                    
                    info
                }
                None => return Err("No bytes data available".to_string())
            }
        }
        _ => return Err(format!("Unsupported item type: {}", item_type)),
    };

    Ok(info)
}

#[tauri::command]
pub fn get_disassembler_info(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;

    match app_item.typ.as_str() {
        "function" => {
            let functions = &bytecode.functions;
            if index >= functions.len() {
                return Err("Function index out of bounds".to_string());
            }
            let function = &functions[index];
            Ok(format!("{}", function.display::<EnhancedFmt>(&bytecode)))
        }
        "class" => {
            let types = &bytecode.types;
            if index >= types.len() {
                return Err("Type index out of bounds".to_string());
            }
            let type_obj = &types[index];
            match type_obj {
                Type::Obj(obj) => {
                    let mut disassembled_type = format!("{}", type_obj.display::<EnhancedFmt>(&bytecode));
                    if let Some(sup) = obj.super_ {
                        disassembled_type += &format!("\nextends {}", sup.display::<EnhancedFmt>(&bytecode));
                    }
                    disassembled_type += &format!("\nglobal: {}", obj.global.0);
                    disassembled_type += "\nfields:";
                    for f in &obj.own_fields {
                        disassembled_type += &format!("\n  {}: {}", f.name.display::<EnhancedFmt>(&bytecode), f.t.display::<EnhancedFmt>(&bytecode));
                    }
                    disassembled_type += "\nprotos:";
                    for p in &obj.protos {
                        disassembled_type += &format!("\n  {}: {} ({})", p.name.display::<EnhancedFmt>(&bytecode), bytecode.get(p.findex).display_header::<EnhancedFmt>(&bytecode), p.pindex);
                    }
                    disassembled_type += "\nbindings:";
                    for (fi, fun) in &obj.bindings {
                        disassembled_type += &format!("\n  {}: {}", fi.display::<EnhancedFmt>(&bytecode, type_obj), fun.display_header::<EnhancedFmt>(&bytecode));
                    }
                    Ok(disassembled_type)
                }
                Type::Enum { global, constructs, .. } => {
                    let mut disassembled_type = format!("global: {}", global.0);
                    disassembled_type += "\nconstructs:";
                    for c in constructs {
                        disassembled_type += &format!("\n  {}:", c.name(&bytecode));
                        for (i, p) in c.params.iter().enumerate() {
                            disassembled_type += &format!("\n    {i}: {}", p.display::<EnhancedFmt>(&bytecode));
                        }
                    }
                    Ok(disassembled_type)
                }
                _ => Err("Type is not an object or enum".to_string()),
            }
        }
        _ => Err(format!("Unsupported item type: {}", app_item.typ)),
    }
}