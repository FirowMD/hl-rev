use crate::app_data::Storage;
use crate::bytecode_refs;
use hlbc::types::{
    EnumConstruct, ObjField, ObjProto, RefField, RefFun, RefGlobal, RefString, RefType, Type,
    TypeFun, TypeObj,
};
use prism_mcp_rs::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct UpdateTypeHandler {
    pub app_handle: AppHandle,
}

#[derive(Debug, serde::Deserialize)]
struct NewFieldInput {
    pub name: String,
    pub field_type: usize,
}

#[derive(Debug, serde::Deserialize)]
struct NewProtoInput {
    pub name: String,
    pub findex: usize,
    pub pindex: i32,
}

#[derive(Debug, serde::Deserialize)]
struct NewEnumConstructInput {
    pub name: String,
    pub params: Vec<usize>,
}

#[derive(Debug, serde::Deserialize)]
struct NewBindingInput {
    pub field: usize,
    pub findex: usize,
}

#[derive(Debug, serde::Deserialize)]
struct UpdateTypeInput {
    pub index: usize,
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
    pub bindings: Option<Vec<NewBindingInput>>,
}

#[async_trait]
impl ToolHandler for UpdateTypeHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let input: UpdateTypeInput = serde_json::from_value(
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

        if input.index >= bytecode.types.len() {
            return Err(McpError::Validation("Type index out of bounds".to_string()));
        }
        let existing_bindings = bytecode.types[input.index]
            .get_type_obj()
            .map(|object| object.bindings.clone())
            .unwrap_or_default();

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
            "guid" => Type::Guid,
            "ref" => {
                let inner = input.inner_type.ok_or_else(|| {
                    McpError::Validation("ref type requires inner_type".to_string())
                })?;
                Type::Ref(RefType(inner))
            }
            "null" => {
                let inner = input.inner_type.ok_or_else(|| {
                    McpError::Validation("null type requires inner_type".to_string())
                })?;
                Type::Null(RefType(inner))
            }
            "packed" => {
                let inner = input.inner_type.ok_or_else(|| {
                    McpError::Validation("packed type requires inner_type".to_string())
                })?;
                Type::Packed(RefType(inner))
            }
            "fun" | "method" => {
                let args = input
                    .args
                    .unwrap_or_default()
                    .into_iter()
                    .map(RefType)
                    .collect();
                let ret = RefType(input.ret.ok_or_else(|| {
                    McpError::Validation("function type requires return type".to_string())
                })?);
                let type_fun = TypeFun { args, ret };
                if input.type_kind == "fun" {
                    Type::Fun(type_fun)
                } else {
                    Type::Method(type_fun)
                }
            }
            "obj" | "struct" => {
                let name_str = input
                    .name
                    .ok_or_else(|| McpError::Validation("object type requires name".to_string()))?;
                let name_idx: usize = name_str
                    .parse()
                    .map_err(|_| McpError::Validation("Invalid name index".to_string()))?;
                let name = RefString(name_idx);
                let super_ = input.super_type.map(RefType);
                let global = RefGlobal(input.global.unwrap_or(0));

                let own_fields: Result<Vec<_>, McpError> = input
                    .fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(|f| {
                        let field_name_idx: usize = f.name.parse().map_err(|_| {
                            McpError::Validation("Invalid field name index".to_string())
                        })?;
                        Ok(ObjField {
                            name: RefString(field_name_idx),
                            t: RefType(f.field_type),
                        })
                    })
                    .collect();
                let own_fields = own_fields?;

                let protos: Result<Vec<_>, McpError> = input
                    .protos
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| {
                        let proto_name_idx: usize = p.name.parse().map_err(|_| {
                            McpError::Validation("Invalid proto name index".to_string())
                        })?;
                        Ok(ObjProto {
                            name: RefString(proto_name_idx),
                            findex: RefFun(p.findex),
                            pindex: p.pindex,
                        })
                    })
                    .collect();
                let protos = protos?;
                let bindings = input.bindings.map_or(existing_bindings, |bindings| {
                    bindings
                        .into_iter()
                        .map(|binding| (RefField(binding.field), RefFun(binding.findex)))
                        .collect()
                });

                let type_obj = TypeObj {
                    name,
                    super_,
                    global,
                    own_fields,
                    fields: Vec::new(),
                    protos,
                    bindings,
                };
                if input.type_kind == "obj" {
                    Type::Obj(type_obj)
                } else {
                    Type::Struct(type_obj)
                }
            }
            "enum" => {
                let name_str = input
                    .name
                    .ok_or_else(|| McpError::Validation("enum type requires name".to_string()))?;
                let name_idx: usize = name_str
                    .parse()
                    .map_err(|_| McpError::Validation("Invalid name index".to_string()))?;
                let name = RefString(name_idx);
                let global = RefGlobal(input.global.unwrap_or(0));

                let constructs: Result<Vec<_>, McpError> = input
                    .constructs
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| {
                        let construct_name_idx: usize = c.name.parse().map_err(|_| {
                            McpError::Validation("Invalid construct name index".to_string())
                        })?;
                        Ok(EnumConstruct {
                            name: RefString(construct_name_idx),
                            params: c.params.into_iter().map(RefType).collect(),
                        })
                    })
                    .collect();
                let constructs = constructs?;

                Type::Enum {
                    name,
                    global,
                    constructs,
                }
            }
            "abstract" => {
                let name_str = input.name.ok_or_else(|| {
                    McpError::Validation("abstract type requires name".to_string())
                })?;
                let name_idx: usize = name_str
                    .parse()
                    .map_err(|_| McpError::Validation("Invalid name index".to_string()))?;
                let name = RefString(name_idx);
                Type::Abstract { name }
            }
            "virtual" => {
                let fields: Result<Vec<_>, McpError> = input
                    .fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(|f| {
                        let field_name_idx: usize = f.name.parse().map_err(|_| {
                            McpError::Validation("Invalid field name index".to_string())
                        })?;
                        Ok(ObjField {
                            name: RefString(field_name_idx),
                            t: RefType(f.field_type),
                        })
                    })
                    .collect();
                let fields = fields?;
                Type::Virtual { fields }
            }
            _ => {
                return Err(McpError::Validation(format!(
                    "Unknown type kind: {}",
                    input.type_kind
                )))
            }
        };

        bytecode_refs::validate_type_refs(bytecode, &updated_type, "updated type")
            .map_err(McpError::Validation)?;
        let mut candidate = bytecode.clone();
        candidate.types[input.index] = updated_type;
        super::support::rebuild_runtime_indexes(&mut candidate)?;
        *bytecode = candidate;
        Ok(CallToolResult::text("ok"))
    }
}

pub async fn register(server: &mut McpServer, app_handle: AppHandle) -> McpResult<()> {
    let ah = app_handle;
    server
        .add_tool(
            "update_type".to_string(),
            Some("Update an existing type".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "index": {"type": "integer"},
                    "type_kind": {"type": "string", "enum": ["void", "ui8", "ui16", "i32", "i64", "f32", "f64", "bool", "bytes", "dyn", "fun", "obj", "array", "type", "ref", "virtual", "dynobj", "abstract", "enum", "null", "method", "struct", "packed", "guid"]},
                    "name": {"type": "string"},
                    "super_type": {"type": "integer"},
                    "global": {"type": "integer"},
                    "inner_type": {"type": "integer"},
                    "args": {"type": "array", "items": {"type": "integer"}},
                    "ret": {"type": "integer"},
                    "fields": {"type": "array"},
                    "protos": {"type": "array"},
                    "constructs": {"type": "array"},
                    "bindings": {"type": "array", "items": {"type": "object", "properties": {"field": {"type": "integer", "minimum": 0}, "findex": {"type": "integer", "minimum": 0}}, "required": ["field", "findex"], "additionalProperties": false}}
                },
                "required": ["index", "type_kind"],
                "additionalProperties": false
            }),
            UpdateTypeHandler { app_handle: ah },
        )
        .await?;
    Ok(())
}
