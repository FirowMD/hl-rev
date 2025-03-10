use hlbc::Bytecode;
use hlbc::Resolve;
use hlbc::fmt::EnhancedFmt;
use hlbc::types::{Type, TypeObj, TypeFun, RefType, ObjField, RefString, EnumConstruct};
use std::collections::HashMap;

pub struct ImHexGenerator<'a> {
    bytecode: &'a Bytecode,
    generated_types: HashMap<String, bool>,
    pattern: String,
    last_array_type: String,
}

pub struct ImHexField {
    name: String,
    typ: ImHexType,
}

pub enum ImHexType {
    Basic(String),  // For u8, s32, etc
    Array(Box<ImHexType>), // For array types
    Struct(String), // For custom struct types
    Nullable(Box<ImHexType>), // For nullable types
}

impl ImHexField {
    fn to_string(&self) -> String {
        match &self.typ {
            ImHexType::Array(element_type) => {
                match &**element_type {
                    ImHexType::Basic(typ) => {
                        if typ == "u8" {
                            format!("{}* {}[]: u64", typ, self.name)
                        } else {
                            format!("{}* {}[]: u64", typ, self.name)
                        }
                    },
                    ImHexType::Struct(typ) => format!("{}* {}[]: u64", typ, self.name),
                    _ => format!("StDynamic* {}[]: u64", self.name)
                }
            }
            ImHexType::Basic(typ) => format!("{} {}", typ, self.name),
            ImHexType::Struct(typ) => format!("{} {}", typ, self.name),
            ImHexType::Nullable(inner) => {
                match &**inner {
                    ImHexType::Basic(typ) => format!("nullable<{}> {}", typ, self.name),
                    ImHexType::Struct(typ) => format!("nullable<{}> {}", typ, self.name),
                    _ => format!("nullable<StDynamic> {}", self.name)
                }
            }
        }
    }
}

impl<'a> ImHexGenerator<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        ImHexGenerator {
            bytecode,
            generated_types: HashMap::new(),
            pattern: String::new(),
            last_array_type: String::new(),
        }
    }

    fn write_standard_types(&mut self) {
        // Add standard dynamic type struct
        self.pattern.push_str("struct StDynamic {\n    s32* hl_type: u64;\n};\n\n");

        // Add Array structure definition with tabs and renamed padding field
        self.pattern.push_str(
            "struct Array {\n\
            \tu64 stack;\n\
            \ts32* hl_type: u64;\n\
            \ts32 size;\n\
            \ts32 pad;\n\
            \tStDynamic* elements[]: u64;\n\
            };\n\n"
        );
    }

    pub fn generate_pattern(&mut self, type_index: usize) -> String {
        self.pattern.clear();
        self.generated_types.clear();
        
        // Write standard types first
        self.write_standard_types();
        
        // Get the type from bytecode
        if let Some(typ) = self.bytecode.types.get(type_index) {
            self.generate_type(typ);
        }
        
        self.pattern.clone()
    }

    fn generate_type(&mut self, typ: &Type) {
        self.generate_type_inner(typ, 0)
    }

    fn generate_type_inner(&mut self, typ: &Type, depth: usize) {
        if depth > 100 {
            return;
        }

        match typ {
            Type::Obj(obj) => self.generate_obj_type_inner(obj, depth + 1),
            Type::Struct(obj) => self.generate_obj_type_inner(obj, depth + 1),
            // Type::Virtual { fields } => self.generate_virtual_type_inner(fields, depth + 1),
            Type::Enum { name, constructs, .. } => self.generate_enum_type(name, constructs),
            Type::Fun(fun) => self.generate_fun_type(fun),
            Type::Ref(ref_type) => {
                let target_type = self.bytecode.get(*ref_type);
                self.generate_type_inner(target_type, depth + 1)
            }
            Type::Null(ref_type) => {
                let target_type = self.bytecode.get(*ref_type);
                self.generate_type_inner(target_type, depth + 1)
            }
            Type::Array => self.generate_array_type(depth + 1),
            _ => {}
        }
    }

    fn generate_obj_type_inner(&mut self, obj: &TypeObj, depth: usize) {
        let type_name = sanitize_type_name(&obj.name.display::<EnhancedFmt>(self.bytecode).to_string());
        
        if self.generated_types.contains_key(&type_name) {
            return;
        }
        self.generated_types.insert(type_name.clone(), true);

        if let Some(parent_type) = &obj.super_ {
            let parent = self.bytecode.get(*parent_type);
            self.generate_type_inner(parent, depth + 1);
        }

        for field in &obj.fields {
            let field_type = self.bytecode.get(field.t);
            self.generate_type_inner(field_type, depth + 1);
        }

        self.pattern.push_str(&format!("struct {} {{\n", type_name));
        self.pattern.push_str("    s32* hl_type: u64;\n");

        if let Some(parent_type) = &obj.super_ {
            let parent = self.bytecode.get(*parent_type);
            if let Type::Obj(parent_obj) = parent {
                let parent_name = sanitize_type_name(&parent_obj.name.display::<EnhancedFmt>(self.bytecode).to_string());
                self.pattern.push_str(&format!("    {} super;\n", parent_name));
            }
        }

        for field in &obj.fields {
            let field_type = self.type_to_imhex_type(self.bytecode.get(field.t));
            let field_name = field.name.display::<EnhancedFmt>(self.bytecode).to_string();
            let imhex_field = ImHexField {
                name: field_name,
                typ: field_type,
            };
            self.pattern.push_str(&format!("    {};\n", imhex_field.to_string()));
        }

        self.pattern.push_str("};\n\n");
    }

    fn generate_virtual_type_inner(&mut self, fields: &[ObjField], depth: usize) {
        let type_name = format!("Virtual_{}", self.generated_types.len());
        
        if self.generated_types.contains_key(&type_name) {
            return;
        }
        self.generated_types.insert(type_name.clone(), true);

        for field in fields {
            let field_type = self.bytecode.get(field.t);
            self.generate_type_inner(field_type, depth + 1);
        }

        self.pattern.push_str(&format!("struct {} {{\n", type_name));
        self.pattern.push_str("    s32* hl_type: u64;\n");
        
        for field in fields {
            let field_type = self.type_to_imhex_type(self.bytecode.get(field.t));
            let field_name = field.name.display::<EnhancedFmt>(self.bytecode).to_string();
            let imhex_field = ImHexField {
                name: field_name,
                typ: field_type,
            };
            self.pattern.push_str(&format!("    {};\n", imhex_field.to_string()));
        }
        
        self.pattern.push_str("};\n\n");
    }

    fn generate_enum_type(&mut self, name: &RefString, constructs: &[EnumConstruct]) {
        let type_name = sanitize_type_name(&name.display::<EnhancedFmt>(self.bytecode).to_string());
        
        if self.generated_types.contains_key(&type_name) {
            return;
        }
        self.generated_types.insert(type_name.clone(), true);

        self.pattern.push_str(&format!("enum {}: s32 {{\n", type_name));
        
        for (i, construct) in constructs.iter().enumerate() {
            let construct_name = construct.name.display::<EnhancedFmt>(self.bytecode);
            self.pattern.push_str(&format!("    {} = {},\n", construct_name, i));
        }
        
        self.pattern.push_str("};\n\n");
    }

    fn generate_fun_type(&mut self, _fun: &TypeFun) {
        //! Function type representation - currently empty
    }

    fn generate_array_type(&mut self, depth: usize) {
        let type_name = format!("Array_{}", self.generated_types.len());
        self.last_array_type = type_name.clone();
        
        if self.generated_types.contains_key(&type_name) {
            return;
        }
        self.generated_types.insert(type_name.clone(), true);

        self.pattern.push_str(&format!("struct {} {{\n", type_name));
        self.pattern.push_str("\tu64 stack;\n");
        self.pattern.push_str("\ts32* hl_type: u64;\n");
        self.pattern.push_str("\ts32 size;\n");
        self.pattern.push_str("\ts32 pad;\n");
        self.pattern.push_str("\tStDynamic* elements[]: u64;\n");
        self.pattern.push_str("};\n\n");
    }

    fn type_to_imhex_type(&self, typ: &Type) -> ImHexType {
        self.type_to_imhex_type_inner(typ, 0)
    }

    fn type_to_imhex_type_inner(&self, typ: &Type, depth: usize) -> ImHexType {
        if depth > 100 {
            return ImHexType::Struct("StDynamic".to_string());
        }

        match typ {
            Type::Void => ImHexType::Basic("u64".to_string()),
            Type::UI8 => ImHexType::Basic("u8".to_string()),
            Type::UI16 => ImHexType::Basic("u16".to_string()),
            Type::I32 => ImHexType::Basic("s32".to_string()),
            Type::I64 => ImHexType::Basic("s64".to_string()),
            Type::F32 => ImHexType::Basic("float".to_string()),
            Type::F64 => ImHexType::Basic("double".to_string()),
            Type::Bool => ImHexType::Basic("bool".to_string()),
            Type::Bytes => ImHexType::Array(Box::new(ImHexType::Basic("u8".to_string()))),
            Type::Dyn => ImHexType::Struct("StDynamic".to_string()),
            Type::Obj(obj) => {
                let type_name = obj.name.display::<EnhancedFmt>(self.bytecode).to_string();
                if type_name == "array" {
                    ImHexType::Struct(self.last_array_type.clone())
                } else {
                    ImHexType::Struct(sanitize_type_name(&type_name))
                }
            }
            Type::Struct(obj) => ImHexType::Struct(sanitize_type_name(&obj.name.display::<EnhancedFmt>(self.bytecode).to_string())),
            Type::Ref(ref_type) => {
                let target_type = self.bytecode.get(*ref_type);
                self.type_to_imhex_type_inner(target_type, depth + 1)
            }
            Type::Null(ref_type) => {
                let target_type = self.bytecode.get(*ref_type);
                ImHexType::Nullable(Box::new(self.type_to_imhex_type_inner(target_type, depth + 1)))
            }
            Type::Array => ImHexType::Struct(self.last_array_type.clone()),
            _ => ImHexType::Struct("StDynamic".to_string())
        }
    }
}

fn sanitize_type_name(name: &str) -> String {
    name.replace('.', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

pub fn generate_imhex_pattern(bytecode: &Bytecode, type_index: usize) -> String {
    let mut generator = ImHexGenerator::new(bytecode);
    generator.generate_pattern(type_index)
}
