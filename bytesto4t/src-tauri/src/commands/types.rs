use tauri::State;
use hlbc::types::{Type, RefType, RefString, RefGlobal, RefFun, TypeFun, TypeObj, ObjField, ObjProto, EnumConstruct, Native, ConstantDef};
use hlbc::fmt::EnhancedFmt;
use std::io::{Write, BufRead};
use std::collections::HashMap;
use crate::app_data::Storage;
use crate::structgen;

#[derive(Debug, serde::Deserialize)]
pub struct NewTypeInput {
    pub type_kind: String,
    pub name: Option<String>,
    pub super_type: Option<usize>,
    pub global: Option<usize>,
    pub inner_type: Option<usize>,
    pub args: Option<Vec<usize>>,
    pub ret: Option<usize>,
    pub fields: Option<Vec<NewFieldInput>>,
    pub protos: Option<Vec<NewProtoInput>>,
    pub constructs: Option<Vec<NewEnumConstructInput>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewFieldInput {
    pub name: String,
    pub field_type: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewProtoInput {
    pub name: String,
    pub findex: usize,
    pub pindex: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewEnumConstructInput {
    pub name: String,
    pub params: Vec<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewGlobalInput {
    pub global_type: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewNativeInput {
    pub lib: String,
    pub name: String,
    pub signature_type: usize,
    pub findex: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewConstantInput {
    pub global: usize,
    pub fields: Vec<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewStringInput {
    pub value: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewIntInput {
    pub value: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewFloatInput {
    pub value: f64,
}

#[tauri::command]
pub fn get_type_list(app_data: State<Storage>) -> Result<Vec<String>, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let types = &bytecode.types;
    let mut type_names = Vec::new();
    let mut index = 0;
    for t in types {
        type_names.push(t.display::<EnhancedFmt>(&bytecode).to_string() +
            "@" + &index.to_string());
        index += 1;
    }

    Ok(type_names)
}

#[tauri::command]
pub fn create_type(input: NewTypeInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;

    let new_type = match input.type_kind.as_str() {
        "void" => Type::Void,
        "ui8" => Type::UI8,
        "ui16" => Type::UI16,
        "i32" => Type::I32,
        "i64" => Type::I64,
        "f32" => Type::F32,
        "f64" => Type::F64,
        "bool" => Type::Bool,
        "bytes" => Type::Bytes,
        "dyn" => Type::Dyn,
        "array" => Type::Array,
        "type" => Type::Type,
        "dynobj" => Type::DynObj,
        "ref" => {
            let inner = input.inner_type.ok_or("ref type requires inner_type")?;
            Type::Ref(RefType(inner))
        }
        "null" => {
            let inner = input.inner_type.ok_or("null type requires inner_type")?;
            Type::Null(RefType(inner))
        }
        "packed" => {
            let inner = input.inner_type.ok_or("packed type requires inner_type")?;
            Type::Packed(RefType(inner))
        }
        "fun" | "method" => {
            let args = input.args.unwrap_or_default().into_iter().map(RefType).collect();
            let ret = RefType(input.ret.ok_or("function type requires return type")?);
            let type_fun = TypeFun { args, ret };
            if input.type_kind == "fun" {
                Type::Fun(type_fun)
            } else {
                Type::Method(type_fun)
            }
        }
        "obj" | "struct" => {
            let name_str = input.name.ok_or("object type requires name")?;
            let name_idx: usize = name_str.parse().map_err(|_| "Invalid name index")?;
            let name = RefString(name_idx);
            let super_ = input.super_type.map(RefType);
            let global = RefGlobal(input.global.unwrap_or(0));
            
            let own_fields: Result<Vec<_>, String> = input.fields.unwrap_or_default()
                .into_iter()
                .map(|f| {
                    let field_name_idx: usize = f.name.parse().map_err(|_| "Invalid field name index")?;
                    Ok(ObjField {
                        name: RefString(field_name_idx),
                        t: RefType(f.field_type),
                    })
                })
                .collect();
            let own_fields = own_fields?;
            
            let protos: Result<Vec<_>, String> = input.protos.unwrap_or_default()
                .into_iter()
                .map(|p| {
                    let proto_name_idx: usize = p.name.parse().map_err(|_| "Invalid proto name index")?;
                    Ok(ObjProto {
                        name: RefString(proto_name_idx),
                        findex: RefFun(p.findex),
                        pindex: p.pindex as i32,
                    })
                })
                .collect();
            let protos = protos?;
            
            let type_obj = TypeObj {
                name,
                super_,
                global,
                own_fields,
                fields: Vec::new(),
                protos,
                bindings: HashMap::new(),
            };
            
            if input.type_kind == "obj" {
                Type::Obj(type_obj)
            } else {
                Type::Struct(type_obj)
            }
        }
        "enum" => {
            let name_str = input.name.ok_or("enum type requires name")?;
            let name_idx: usize = name_str.parse().map_err(|_| "Invalid name index")?;
            let name = RefString(name_idx);
            let global = RefGlobal(input.global.unwrap_or(0));
            
            let constructs: Result<Vec<_>, String> = input.constructs.unwrap_or_default()
                .into_iter()
                .map(|c| {
                    let construct_name_idx: usize = c.name.parse().map_err(|_| "Invalid construct name index")?;
                    Ok(EnumConstruct {
                        name: RefString(construct_name_idx),
                        params: c.params.into_iter().map(RefType).collect(),
                    })
                })
                .collect();
            let constructs = constructs?;
            
            Type::Enum { name, global, constructs }
        }
        "abstract" => {
            let name_str = input.name.ok_or("abstract type requires name")?;
            let name_idx: usize = name_str.parse().map_err(|_| "Invalid name index")?;
            let name = RefString(name_idx);
            Type::Abstract { name }
        }
        "virtual" => {
            let fields: Result<Vec<_>, String> = input.fields.unwrap_or_default()
                .into_iter()
                .map(|f| {
                    let field_name_idx: usize = f.name.parse().map_err(|_| "Invalid field name index")?;
                    Ok(ObjField {
                        name: RefString(field_name_idx),
                        t: RefType(f.field_type),
                    })
                })
                .collect();
            let fields = fields?;
            
            Type::Virtual { fields }
        }
        _ => return Err(format!("Unknown type kind: {}", input.type_kind)),
    };

    bytecode.types.push(new_type);
    Ok(())
}

#[tauri::command]
pub fn update_type(index: usize, input: NewTypeInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.types.len() {
        return Err(format!("Type index {} out of bounds", index));
    }

    let updated_type = match input.type_kind.as_str() {
        "void" => Type::Void,
        "ui8" => Type::UI8,
        "ui16" => Type::UI16,
        "i32" => Type::I32,
        "i64" => Type::I64,
        "f32" => Type::F32,
        "f64" => Type::F64,
        "bool" => Type::Bool,
        "bytes" => Type::Bytes,
        "dyn" => Type::Dyn,
        "array" => Type::Array,
        "type" => Type::Type,
        "dynobj" => Type::DynObj,
        "ref" => {
            let inner = input.inner_type.ok_or("ref type requires inner_type")?;
            Type::Ref(RefType(inner))
        }
        "null" => {
            let inner = input.inner_type.ok_or("null type requires inner_type")?;
            Type::Null(RefType(inner))
        }
        "packed" => {
            let inner = input.inner_type.ok_or("packed type requires inner_type")?;
            Type::Packed(RefType(inner))
        }
        "fun" | "method" => {
            let args = input.args.unwrap_or_default().into_iter().map(RefType).collect();
            let ret = RefType(input.ret.ok_or("function type requires return type")?);
            let type_fun = TypeFun { args, ret };
            if input.type_kind == "fun" {
                Type::Fun(type_fun)
            } else {
                Type::Method(type_fun)
            }
        }
        "obj" | "struct" => {
            let name_str = input.name.ok_or("object type requires name")?;
            let name_idx: usize = name_str.parse().map_err(|_| "Invalid name index")?;
            let name = RefString(name_idx);
            let super_ = input.super_type.map(RefType);
            let global = RefGlobal(input.global.unwrap_or(0));
            
            let own_fields: Result<Vec<_>, String> = input.fields.unwrap_or_default()
                .into_iter()
                .map(|f| {
                    let field_name_idx: usize = f.name.parse().map_err(|_| "Invalid field name index")?;
                    Ok(ObjField {
                        name: RefString(field_name_idx),
                        t: RefType(f.field_type),
                    })
                })
                .collect();
            let own_fields = own_fields?;
            
            let protos: Result<Vec<_>, String> = input.protos.unwrap_or_default()
                .into_iter()
                .map(|p| {
                    let proto_name_idx: usize = p.name.parse().map_err(|_| "Invalid proto name index")?;
                    Ok(ObjProto {
                        name: RefString(proto_name_idx),
                        findex: RefFun(p.findex),
                        pindex: p.pindex as i32,
                    })
                })
                .collect();
            let protos = protos?;
            
            let type_obj = TypeObj {
                name,
                super_,
                global,
                own_fields,
                fields: Vec::new(),
                protos,
                bindings: HashMap::new(),
            };
            
            if input.type_kind == "obj" {
                Type::Obj(type_obj)
            } else {
                Type::Struct(type_obj)
            }
        }
        "enum" => {
            let name_str = input.name.ok_or("enum type requires name")?;
            let name_idx: usize = name_str.parse().map_err(|_| "Invalid name index")?;
            let name = RefString(name_idx);
            let global = RefGlobal(input.global.unwrap_or(0));
            
            let constructs: Result<Vec<_>, String> = input.constructs.unwrap_or_default()
                .into_iter()
                .map(|c| {
                    let construct_name_idx: usize = c.name.parse().map_err(|_| "Invalid construct name index")?;
                    Ok(EnumConstruct {
                        name: RefString(construct_name_idx),
                        params: c.params.into_iter().map(RefType).collect(),
                    })
                })
                .collect();
            let constructs = constructs?;
            
            Type::Enum { name, global, constructs }
        }
        "abstract" => {
            let name_str = input.name.ok_or("abstract type requires name")?;
            let name_idx: usize = name_str.parse().map_err(|_| "Invalid name index")?;
            let name = RefString(name_idx);
            Type::Abstract { name }
        }
        "virtual" => {
            let fields: Result<Vec<_>, String> = input.fields.unwrap_or_default()
                .into_iter()
                .map(|f| {
                    let field_name_idx: usize = f.name.parse().map_err(|_| "Invalid field name index")?;
                    Ok(ObjField {
                        name: RefString(field_name_idx),
                        t: RefType(f.field_type),
                    })
                })
                .collect();
            let fields = fields?;
            
            Type::Virtual { fields }
        }
        _ => return Err(format!("Unknown type kind: {}", input.type_kind)),
    };

    bytecode.types[index] = updated_type;
    Ok(())
}

#[tauri::command]
pub fn delete_type(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.types.len() {
        return Err(format!("Type index {} out of bounds", index));
    }
    
    bytecode.types.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_type_full_info(index: usize, app_data: State<Storage>) -> Result<Type, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.types.len() {
        return Err(format!("Type index {} out of bounds", index));
    }
    
    Ok(bytecode.types[index].clone())
}

#[tauri::command]
pub fn import_type_json(json_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    let json_file = std::fs::File::open(json_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(json_file);
    let json_content: String = reader.lines()
        .map(|line| line.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    let ty = Type::from_json(json_content.as_str()).map_err(|e| e.to_string())?;
    bytecode.add_type(ty);
    Ok(())
}

#[tauri::command]
pub fn export_type_json(type_index: &str, file_path: &str, app_data: State<Storage>) -> Result<(), String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    let types = &bytecode.types;
    let index: usize = type_index.parse().map_err(|_| "Invalid index format")?;

    if index >= types.len() {
        return Err("Type index out of bounds".to_string());
    }

    let type_obj = &types[index];
    let json_content = type_obj.to_json().map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(file_path).map_err(|e| e.to_string())?;
    writeln!(file, "{}", json_content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn generate_imhex_pattern(app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let app_item = app_data.selected_item.as_ref().ok_or("No item selected")?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;

    match app_item.typ.as_str() {
        "class" => {
            let index: usize = app_item.index.parse().map_err(|_| "Invalid index format")?;
            if index >= bytecode.types.len() {
                return Err("Type index out of bounds".to_string());
            }
            
            Ok(structgen::generate_imhex_pattern(bytecode, index))
        },
        _ => Err(format!("Cannot generate ImHex pattern for item type: {}", app_item.typ))
    }
}

#[tauri::command]
pub fn create_global(input: NewGlobalInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    bytecode.globals.push(RefType(input.global_type));
    Ok(())
}

#[tauri::command]
pub fn update_global(index: usize, input: NewGlobalInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.globals.len() {
        return Err(format!("Global index {} out of bounds", index));
    }
    
    bytecode.globals[index] = RefType(input.global_type);
    Ok(())
}

#[tauri::command]
pub fn delete_global(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.globals.len() {
        return Err(format!("Global index {} out of bounds", index));
    }
    
    bytecode.globals.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_global_full_info(index: usize, app_data: State<Storage>) -> Result<RefType, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.globals.len() {
        return Err(format!("Global index {} out of bounds", index));
    }
    
    Ok(bytecode.globals[index])
}

#[tauri::command]
pub fn create_native(input: NewNativeInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    let lib_idx: usize = input.lib.parse().map_err(|_| "Invalid lib string index")?;
    let name_idx: usize = input.name.parse().map_err(|_| "Invalid name string index")?;
    
    let lib = RefString(lib_idx);
    let name = RefString(name_idx);
    let t = RefType(input.signature_type);
    let findex = RefFun(input.findex.unwrap_or(bytecode.natives.len()));
    
    bytecode.natives.push(Native { lib, name, t, findex });
    Ok(())
}

#[tauri::command]
pub fn update_native(index: usize, input: NewNativeInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.natives.len() {
        return Err(format!("Native index {} out of bounds", index));
    }
    
    let lib_idx: usize = input.lib.parse().map_err(|_| "Invalid lib string index")?;
    let name_idx: usize = input.name.parse().map_err(|_| "Invalid name string index")?;
    
    let lib = RefString(lib_idx);
    let name = RefString(name_idx);
    let t = RefType(input.signature_type);
    let findex = RefFun(input.findex.unwrap_or(bytecode.natives[index].findex.0));
    
    bytecode.natives[index] = Native { lib, name, t, findex };
    Ok(())
}

#[tauri::command]
pub fn delete_native(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.natives.len() {
        return Err(format!("Native index {} out of bounds", index));
    }
    
    bytecode.natives.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_native_full_info(index: usize, app_data: State<Storage>) -> Result<Native, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.natives.len() {
        return Err(format!("Native index {} out of bounds", index));
    }
    
    Ok(bytecode.natives[index].clone())
}

#[tauri::command]
pub fn create_constant(input: NewConstantInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    let constants = bytecode.constants.get_or_insert_with(Vec::new);
    
    constants.push(ConstantDef {
        global: RefGlobal(input.global),
        fields: input.fields,
    });
    
    Ok(())
}

#[tauri::command]
pub fn update_constant(index: usize, input: NewConstantInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    let constants = bytecode.constants.as_mut().ok_or("No constants defined")?;
    
    if index >= constants.len() {
        return Err(format!("Constant index {} out of bounds", index));
    }
    
    constants[index] = ConstantDef {
        global: RefGlobal(input.global),
        fields: input.fields,
    };
    
    Ok(())
}

#[tauri::command]
pub fn delete_constant(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    let constants = bytecode.constants.as_mut().ok_or("No constants defined")?;
    
    if index >= constants.len() {
        return Err(format!("Constant index {} out of bounds", index));
    }
    
    constants.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_constant_full_info(index: usize, app_data: State<Storage>) -> Result<ConstantDef, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    let constants = bytecode.constants.as_ref().ok_or("No constants defined")?;
    
    if index >= constants.len() {
        return Err(format!("Constant index {} out of bounds", index));
    }
    
    Ok(constants[index].clone())
}

#[tauri::command]
pub fn create_string(input: NewStringInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    bytecode.strings.push(hlbc::Str::from(input.value));
    Ok(())
}

#[tauri::command]
pub fn update_string(index: usize, input: NewStringInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.strings.len() {
        return Err(format!("String index {} out of bounds", index));
    }
    
    bytecode.strings[index] = hlbc::Str::from(input.value);
    Ok(())
}

#[tauri::command]
pub fn delete_string(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.strings.len() {
        return Err(format!("String index {} out of bounds", index));
    }
    
    if index == 0 {
        return Err("Cannot delete reserved string at index 0".to_string());
    }
    
    bytecode.strings.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_string_full_info(index: usize, app_data: State<Storage>) -> Result<String, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.strings.len() {
        return Err(format!("String index {} out of bounds", index));
    }
    
    Ok(bytecode.strings[index].to_string())
}

#[tauri::command]
pub fn create_int(input: NewIntInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    bytecode.ints.push(input.value);
    Ok(())
}

#[tauri::command]
pub fn update_int(index: usize, input: NewIntInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.ints.len() {
        return Err(format!("Int index {} out of bounds", index));
    }
    
    bytecode.ints[index] = input.value;
    Ok(())
}

#[tauri::command]
pub fn delete_int(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.ints.len() {
        return Err(format!("Int index {} out of bounds", index));
    }
    
    bytecode.ints.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_int_full_info(index: usize, app_data: State<Storage>) -> Result<i32, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.ints.len() {
        return Err(format!("Int index {} out of bounds", index));
    }
    
    Ok(bytecode.ints[index])
}

#[tauri::command]
pub fn create_float(input: NewFloatInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    bytecode.floats.push(input.value);
    Ok(())
}

#[tauri::command]
pub fn update_float(index: usize, input: NewFloatInput, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.floats.len() {
        return Err(format!("Float index {} out of bounds", index));
    }
    
    bytecode.floats[index] = input.value;
    Ok(())
}

#[tauri::command]
pub fn delete_float(index: usize, app_data: State<Storage>) -> Result<(), String> {
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_mut().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.floats.len() {
        return Err(format!("Float index {} out of bounds", index));
    }
    
    bytecode.floats.remove(index);
    Ok(())
}

#[tauri::command]
pub fn get_float_full_info(index: usize, app_data: State<Storage>) -> Result<f64, String> {
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    let bytecode = app_data.bytecode.as_ref().ok_or("bytecode not loaded")?;
    
    if index >= bytecode.floats.len() {
        return Err(format!("Float index {} out of bounds", index));
    }
    
    Ok(bytecode.floats[index])
}