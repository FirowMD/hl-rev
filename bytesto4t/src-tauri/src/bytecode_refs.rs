use hlbc::opcodes::Opcode;
use hlbc::types::{
    ConstantDef, Function, Native, ObjField, ObjProto, RefBytes, RefFloat, RefFun, RefInt, RefType,
    Type, TypeFun, TypeObj,
};
use hlbc::Bytecode;

fn bounds_error(kind: &str, index: usize, len: usize, context: &str) -> String {
    format!(
        "{} reference {} is out of bounds for {} (len: {})",
        kind, index, context, len
    )
}

fn ensure_index(kind: &str, index: usize, len: usize, context: &str) -> Result<(), String> {
    if index < len {
        Ok(())
    } else {
        Err(bounds_error(kind, index, len, context))
    }
}

pub fn ensure_type_index(bytecode: &Bytecode, index: usize, context: &str) -> Result<(), String> {
    ensure_index("Type", index, bytecode.types.len(), context)
}

pub fn ensure_function_type_index(
    bytecode: &Bytecode,
    index: usize,
    context: &str,
) -> Result<(), String> {
    ensure_type_index(bytecode, index, context)?;
    match &bytecode.types[index] {
        Type::Fun(_) | Type::Method(_) => Ok(()),
        other => Err(format!(
            "Type reference {} for {} must point to Type::Fun or Type::Method, got {:?}",
            index, context, other
        )),
    }
}

pub fn ensure_string_index(
    bytecode: &Bytecode,
    index: usize,
    context: &str,
    allow_null: bool,
) -> Result<(), String> {
    if allow_null && index == 0 {
        return Ok(());
    }
    ensure_index("String", index, bytecode.strings.len(), context)
}

pub fn ensure_global_index(bytecode: &Bytecode, index: usize, context: &str) -> Result<(), String> {
    ensure_index("Global", index, bytecode.globals.len(), context)
}

pub fn ensure_type_global_index(
    bytecode: &Bytecode,
    index: usize,
    context: &str,
) -> Result<(), String> {
    if index <= bytecode.globals.len() {
        Ok(())
    } else {
        Err(format!(
            "Global reference {} is out of bounds for {} (valid sentinel/index range: 0..={})",
            index,
            context,
            bytecode.globals.len()
        ))
    }
}

pub fn ensure_fun_ref(bytecode: &Bytecode, findex: RefFun, context: &str) -> Result<(), String> {
    if bytecode.safe_get_ref_fun(findex).is_some() {
        Ok(())
    } else {
        Err(format!(
            "Function reference {} is out of bounds for {}",
            findex.0, context
        ))
    }
}

pub fn ensure_int_ref(bytecode: &Bytecode, ptr: RefInt, context: &str) -> Result<(), String> {
    ensure_index("Int", ptr.0, bytecode.ints.len(), context)
}

pub fn ensure_float_ref(bytecode: &Bytecode, ptr: RefFloat, context: &str) -> Result<(), String> {
    ensure_index("Float", ptr.0, bytecode.floats.len(), context)
}

pub fn ensure_bytes_ref(bytecode: &Bytecode, ptr: RefBytes, context: &str) -> Result<(), String> {
    let len = bytecode
        .bytes
        .as_ref()
        .map(|(_, indices)| indices.len())
        .unwrap_or(0);
    ensure_index("Bytes", ptr.0, len, context)
}

pub fn validate_type_refs(bytecode: &Bytecode, ty: &Type, context: &str) -> Result<(), String> {
    match ty {
        Type::Fun(fun) | Type::Method(fun) => validate_type_fun(bytecode, fun, context),
        Type::Obj(obj) | Type::Struct(obj) => validate_type_obj(bytecode, obj, context),
        Type::Ref(inner) | Type::Null(inner) | Type::Packed(inner) => {
            ensure_type_index(bytecode, inner.0, context)
        }
        Type::Virtual { fields } => validate_fields(bytecode, fields, context),
        Type::Abstract { name } => ensure_string_index(bytecode, name.0, context, false),
        Type::Enum {
            name,
            global,
            constructs,
        } => {
            ensure_string_index(bytecode, name.0, context, false)?;
            ensure_type_global_index(bytecode, global.0, context)?;
            for (construct_idx, construct) in constructs.iter().enumerate() {
                let construct_context = format!("{} construct {}", context, construct_idx);
                ensure_string_index(bytecode, construct.name.0, &construct_context, false)?;
                for (param_idx, param) in construct.params.iter().enumerate() {
                    ensure_type_index(
                        bytecode,
                        param.0,
                        &format!("{} param {}", construct_context, param_idx),
                    )?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_type_fun(bytecode: &Bytecode, fun: &TypeFun, context: &str) -> Result<(), String> {
    for (arg_idx, arg) in fun.args.iter().enumerate() {
        ensure_type_index(bytecode, arg.0, &format!("{} arg {}", context, arg_idx))?;
    }
    ensure_type_index(bytecode, fun.ret.0, &format!("{} return", context))
}

fn validate_type_obj(bytecode: &Bytecode, obj: &TypeObj, context: &str) -> Result<(), String> {
    ensure_string_index(bytecode, obj.name.0, context, false)?;
    if let Some(super_type) = obj.super_ {
        ensure_type_index(bytecode, super_type.0, &format!("{} super", context))?;
    }
    ensure_type_global_index(bytecode, obj.global.0, context)?;
    validate_fields(
        bytecode,
        &obj.own_fields,
        &format!("{} own_fields", context),
    )?;
    validate_fields(bytecode, &obj.fields, &format!("{} fields", context))?;
    for (proto_idx, proto) in obj.protos.iter().enumerate() {
        validate_proto(bytecode, proto, &format!("{} proto {}", context, proto_idx))?;
    }
    for (field, fun) in &obj.bindings {
        if field.0 >= obj.fields.len() {
            return Err(format!(
                "Field reference {} is out of bounds for {} bindings (fields len: {})",
                field.0,
                context,
                obj.fields.len()
            ));
        }
        ensure_fun_ref(bytecode, *fun, &format!("{} binding {}", context, field.0))?;
    }
    Ok(())
}

fn validate_fields(bytecode: &Bytecode, fields: &[ObjField], context: &str) -> Result<(), String> {
    for (field_idx, field) in fields.iter().enumerate() {
        let field_context = format!("{} field {}", context, field_idx);
        ensure_string_index(bytecode, field.name.0, &field_context, false)?;
        ensure_type_index(bytecode, field.t.0, &field_context)?;
    }
    Ok(())
}

fn validate_proto(bytecode: &Bytecode, proto: &ObjProto, context: &str) -> Result<(), String> {
    ensure_string_index(bytecode, proto.name.0, context, false)?;
    ensure_fun_ref(bytecode, proto.findex, context)
}

pub fn validate_global_refs(
    bytecode: &Bytecode,
    global_type: RefType,
    context: &str,
) -> Result<(), String> {
    ensure_type_index(bytecode, global_type.0, context)
}

pub fn validate_native_refs(
    bytecode: &Bytecode,
    native: &Native,
    context: &str,
) -> Result<(), String> {
    ensure_string_index(bytecode, native.lib.0, &format!("{} lib", context), false)?;
    ensure_string_index(bytecode, native.name.0, &format!("{} name", context), false)?;
    ensure_function_type_index(bytecode, native.t.0, &format!("{} signature", context))
}

pub fn validate_constant_refs(
    bytecode: &Bytecode,
    constant: &ConstantDef,
    context: &str,
) -> Result<(), String> {
    ensure_global_index(bytecode, constant.global.0, context)?;
    let global_type = bytecode.globals[constant.global.0];
    if let Some(obj) = bytecode
        .types
        .get(global_type.0)
        .and_then(Type::get_type_obj)
    {
        for field in &constant.fields {
            if *field >= obj.fields.len() {
                return Err(format!(
                    "Field reference {} is out of bounds for {} (fields len: {})",
                    field,
                    context,
                    obj.fields.len()
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_function_refs(
    bytecode: &Bytecode,
    function: &Function,
    context: &str,
    allow_null_name: bool,
) -> Result<(), String> {
    validate_function_refs_with_pending_fun(bytecode, function, context, allow_null_name, None)
}

pub fn validate_function_refs_with_pending_fun(
    bytecode: &Bytecode,
    function: &Function,
    context: &str,
    allow_null_name: bool,
    pending_fun: Option<RefFun>,
) -> Result<(), String> {
    ensure_function_type_index(bytecode, function.t.0, &format!("{} type", context))?;
    ensure_string_index(
        bytecode,
        function.name.0,
        &format!("{} name", context),
        allow_null_name,
    )?;
    if let Some(parent) = function.parent {
        ensure_type_index(bytecode, parent.0, &format!("{} parent", context))?;
    }
    for (reg_idx, reg_type) in function.regs.iter().enumerate() {
        ensure_type_index(
            bytecode,
            reg_type.0,
            &format!("{} register {}", context, reg_idx),
        )?;
    }
    if let Some(assigns) = &function.assigns {
        for (assign_idx, (name, _slot)) in assigns.iter().enumerate() {
            ensure_string_index(
                bytecode,
                name.0,
                &format!("{} assign {}", context, assign_idx),
                false,
            )?;
        }
    }
    if let Some(debug_info) = &function.debug_info {
        if let Some(files) = &bytecode.debug_files {
            for (debug_idx, (file_idx, _line)) in debug_info.iter().enumerate() {
                ensure_index(
                    "Debug file",
                    *file_idx,
                    files.len(),
                    &format!("{} debug_info {}", context, debug_idx),
                )?;
            }
        } else if !debug_info.is_empty() {
            return Err(format!(
                "{} has debug_info but bytecode has no debug_files",
                context
            ));
        }
    }
    for (op_idx, op) in function.ops.iter().enumerate() {
        validate_opcode_refs(
            bytecode,
            function,
            op,
            &format!("{} opcode {}", context, op_idx),
            pending_fun,
        )?;
    }
    Ok(())
}

fn validate_opcode_refs(
    bytecode: &Bytecode,
    function: &Function,
    op: &Opcode,
    context: &str,
    pending_fun: Option<RefFun>,
) -> Result<(), String> {
    match op {
        Opcode::Int { ptr, .. } => ensure_int_ref(bytecode, *ptr, context),
        Opcode::Float { ptr, .. } => ensure_float_ref(bytecode, *ptr, context),
        Opcode::Bytes { ptr, .. } => ensure_bytes_ref(bytecode, *ptr, context),
        Opcode::String { ptr, .. } => ensure_string_index(bytecode, ptr.0, context, false),
        Opcode::Call0 { fun, .. }
        | Opcode::Call1 { fun, .. }
        | Opcode::Call2 { fun, .. }
        | Opcode::Call3 { fun, .. }
        | Opcode::Call4 { fun, .. }
        | Opcode::CallN { fun, .. }
        | Opcode::StaticClosure { fun, .. }
        | Opcode::InstanceClosure { fun, .. } => {
            if pending_fun
                .map(|pending| pending.0 == fun.0)
                .unwrap_or(false)
            {
                Ok(())
            } else {
                ensure_fun_ref(bytecode, *fun, context)
            }
        }
        Opcode::GetGlobal { global, .. } | Opcode::SetGlobal { global, .. } => {
            ensure_global_index(bytecode, global.0, context)
        }
        Opcode::DynGet { field, .. } | Opcode::DynSet { field, .. } => {
            ensure_string_index(bytecode, field.0, context, false)
        }
        Opcode::Type { ty, .. } => ensure_type_index(bytecode, ty.0, context),
        Opcode::Field { obj, field, .. } | Opcode::SetField { obj, field, .. } => {
            validate_field_ref(bytecode, function, obj.0 as usize, field.0, context)
        }
        Opcode::GetThis { field, .. }
        | Opcode::SetThis { field, .. }
        | Opcode::CallThis { field, .. } => {
            if !function.regs.is_empty() {
                validate_field_ref(bytecode, function, 0, field.0, context)?;
            }
            Ok(())
        }
        Opcode::CallMethod { field, args, .. } => {
            if let Some(obj) = args.first() {
                validate_field_ref(bytecode, function, obj.0 as usize, field.0, context)?;
            }
            Ok(())
        }
        Opcode::Prefetch { value, field, .. } => {
            validate_field_ref(bytecode, function, value.0 as usize, field.0, context)
        }
        _ => Ok(()),
    }
}

fn validate_field_ref(
    bytecode: &Bytecode,
    function: &Function,
    reg_idx: usize,
    field_idx: usize,
    context: &str,
) -> Result<(), String> {
    let Some(reg_type) = function.regs.get(reg_idx) else {
        return Ok(());
    };
    let Some(obj) = bytecode.types.get(reg_type.0).and_then(Type::get_type_obj) else {
        return Ok(());
    };
    if field_idx < obj.fields.len() {
        Ok(())
    } else {
        Err(format!(
            "Field reference {} is out of bounds for {} (fields len: {})",
            field_idx,
            context,
            obj.fields.len()
        ))
    }
}

pub fn ensure_tail_delete(
    kind: &str,
    index: usize,
    len: usize,
    references: Vec<String>,
) -> Result<(), String> {
    ensure_index(kind, index, len, "delete")?;
    if index + 1 != len {
        return Err(format!(
            "Cannot delete {} index {} because it would shift following indexes; reference repair is required first.",
            kind, index
        ));
    }
    if references.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Cannot delete {} index {} because it is still referenced: {}",
            kind,
            index,
            summarize_references(&references)
        ))
    }
}

fn summarize_references(references: &[String]) -> String {
    let mut summary = references
        .iter()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if references.len() > 5 {
        summary.push_str(&format!(", and {} more", references.len() - 5));
    }
    summary
}

pub fn type_references(bytecode: &Bytecode, target: usize) -> Vec<String> {
    let mut refs = Vec::new();
    for (idx, global_type) in bytecode.globals.iter().enumerate() {
        if global_type.0 == target {
            refs.push(format!("global[{}]", idx));
        }
    }
    for (idx, ty) in bytecode.types.iter().enumerate() {
        collect_type_refs(ty, target, &format!("type[{}]", idx), &mut refs);
    }
    for (idx, native) in bytecode.natives.iter().enumerate() {
        if native.t.0 == target {
            refs.push(format!("native[{}].t", idx));
        }
    }
    for (idx, function) in bytecode.functions.iter().enumerate() {
        if function.t.0 == target {
            refs.push(format!("function[{}].t", idx));
        }
        if function
            .parent
            .map(|parent| parent.0 == target)
            .unwrap_or(false)
        {
            refs.push(format!("function[{}].parent", idx));
        }
        for (reg_idx, reg) in function.regs.iter().enumerate() {
            if reg.0 == target {
                refs.push(format!("function[{}].regs[{}]", idx, reg_idx));
            }
        }
        for (op_idx, op) in function.ops.iter().enumerate() {
            if matches!(op, Opcode::Type { ty, .. } if ty.0 == target) {
                refs.push(format!("function[{}].ops[{}]", idx, op_idx));
            }
        }
    }
    refs
}

fn collect_type_refs(ty: &Type, target: usize, context: &str, refs: &mut Vec<String>) {
    match ty {
        Type::Fun(fun) | Type::Method(fun) => collect_type_fun_refs(fun, target, context, refs),
        Type::Obj(obj) | Type::Struct(obj) => collect_type_obj_refs(obj, target, context, refs),
        Type::Ref(inner) | Type::Null(inner) | Type::Packed(inner) => {
            if inner.0 == target {
                refs.push(format!("{}.inner", context));
            }
        }
        Type::Virtual { fields } => collect_field_type_refs(fields, target, context, refs),
        Type::Enum { constructs, .. } => {
            for (construct_idx, construct) in constructs.iter().enumerate() {
                for (param_idx, param) in construct.params.iter().enumerate() {
                    if param.0 == target {
                        refs.push(format!(
                            "{}.constructs[{}].params[{}]",
                            context, construct_idx, param_idx
                        ));
                    }
                }
            }
        }
        _ => {}
    }
}

fn collect_type_fun_refs(fun: &TypeFun, target: usize, context: &str, refs: &mut Vec<String>) {
    for (arg_idx, arg) in fun.args.iter().enumerate() {
        if arg.0 == target {
            refs.push(format!("{}.args[{}]", context, arg_idx));
        }
    }
    if fun.ret.0 == target {
        refs.push(format!("{}.ret", context));
    }
}

fn collect_type_obj_refs(obj: &TypeObj, target: usize, context: &str, refs: &mut Vec<String>) {
    if obj
        .super_
        .map(|super_type| super_type.0 == target)
        .unwrap_or(false)
    {
        refs.push(format!("{}.super", context));
    }
    collect_field_type_refs(
        &obj.own_fields,
        target,
        &format!("{}.own_fields", context),
        refs,
    );
    collect_field_type_refs(&obj.fields, target, &format!("{}.fields", context), refs);
}

fn collect_field_type_refs(
    fields: &[ObjField],
    target: usize,
    context: &str,
    refs: &mut Vec<String>,
) {
    for (field_idx, field) in fields.iter().enumerate() {
        if field.t.0 == target {
            refs.push(format!("{}[{}].t", context, field_idx));
        }
    }
}

pub fn global_references(bytecode: &Bytecode, target: usize) -> Vec<String> {
    let mut refs = Vec::new();
    for (idx, ty) in bytecode.types.iter().enumerate() {
        match ty {
            Type::Obj(obj) | Type::Struct(obj) if obj.global.0 != 0 && obj.global.0 == target => {
                refs.push(format!("type[{}].global", idx));
            }
            Type::Enum { global, .. } if global.0 != 0 && global.0 == target => {
                refs.push(format!("type[{}].global", idx));
            }
            _ => {}
        }
    }
    if let Some(constants) = &bytecode.constants {
        for (idx, constant) in constants.iter().enumerate() {
            if constant.global.0 == target {
                refs.push(format!("constant[{}].global", idx));
            }
        }
    }
    for (function_idx, function) in bytecode.functions.iter().enumerate() {
        for (op_idx, op) in function.ops.iter().enumerate() {
            match op {
                Opcode::GetGlobal { global, .. } | Opcode::SetGlobal { global, .. }
                    if global.0 == target =>
                {
                    refs.push(format!("function[{}].ops[{}]", function_idx, op_idx));
                }
                _ => {}
            }
        }
    }
    refs
}

pub fn function_references(bytecode: &Bytecode, target: RefFun) -> Vec<String> {
    let mut refs = Vec::new();
    if bytecode.entrypoint.0 == target.0 {
        refs.push("entrypoint".to_string());
    }
    for (idx, ty) in bytecode.types.iter().enumerate() {
        collect_type_fun_refs_for_delete(ty, target, &format!("type[{}]", idx), &mut refs);
    }
    for (function_idx, function) in bytecode.functions.iter().enumerate() {
        for (op_idx, _op, fun) in function.find_fun_refs() {
            if fun.0 == target.0 {
                refs.push(format!("function[{}].ops[{}]", function_idx, op_idx));
            }
        }
    }
    refs
}

fn collect_type_fun_refs_for_delete(
    ty: &Type,
    target: RefFun,
    context: &str,
    refs: &mut Vec<String>,
) {
    let Some(obj) = ty.get_type_obj() else {
        return;
    };
    for (proto_idx, proto) in obj.protos.iter().enumerate() {
        if proto.findex.0 == target.0 {
            refs.push(format!("{}.protos[{}].findex", context, proto_idx));
        }
    }
    for (field, fun) in &obj.bindings {
        if fun.0 == target.0 {
            refs.push(format!("{}.bindings[{}]", context, field.0));
        }
    }
}

pub fn string_references(bytecode: &Bytecode, target: usize) -> Vec<String> {
    let mut refs = Vec::new();
    for (idx, ty) in bytecode.types.iter().enumerate() {
        collect_type_string_refs(ty, target, &format!("type[{}]", idx), &mut refs);
    }
    for (idx, native) in bytecode.natives.iter().enumerate() {
        if native.lib.0 == target {
            refs.push(format!("native[{}].lib", idx));
        }
        if native.name.0 == target {
            refs.push(format!("native[{}].name", idx));
        }
    }
    for (idx, function) in bytecode.functions.iter().enumerate() {
        if function.name.0 == target {
            refs.push(format!("function[{}].name", idx));
        }
        if let Some(assigns) = &function.assigns {
            for (assign_idx, (name, _slot)) in assigns.iter().enumerate() {
                if name.0 == target {
                    refs.push(format!("function[{}].assigns[{}]", idx, assign_idx));
                }
            }
        }
        for (op_idx, op) in function.ops.iter().enumerate() {
            match op {
                Opcode::String { ptr, .. } if ptr.0 == target => {
                    refs.push(format!("function[{}].ops[{}]", idx, op_idx));
                }
                Opcode::DynGet { field, .. } | Opcode::DynSet { field, .. }
                    if field.0 == target =>
                {
                    refs.push(format!("function[{}].ops[{}]", idx, op_idx));
                }
                _ => {}
            }
        }
    }
    refs
}

fn collect_type_string_refs(ty: &Type, target: usize, context: &str, refs: &mut Vec<String>) {
    match ty {
        Type::Obj(obj) | Type::Struct(obj) => {
            if obj.name.0 == target {
                refs.push(format!("{}.name", context));
            }
            collect_field_string_refs(
                &obj.own_fields,
                target,
                &format!("{}.own_fields", context),
                refs,
            );
            collect_field_string_refs(&obj.fields, target, &format!("{}.fields", context), refs);
            for (proto_idx, proto) in obj.protos.iter().enumerate() {
                if proto.name.0 == target {
                    refs.push(format!("{}.protos[{}].name", context, proto_idx));
                }
            }
        }
        Type::Virtual { fields } => {
            collect_field_string_refs(fields, target, &format!("{}.fields", context), refs);
        }
        Type::Abstract { name } => {
            if name.0 == target {
                refs.push(format!("{}.name", context));
            }
        }
        Type::Enum {
            name, constructs, ..
        } => {
            if name.0 == target {
                refs.push(format!("{}.name", context));
            }
            for (construct_idx, construct) in constructs.iter().enumerate() {
                if construct.name.0 == target {
                    refs.push(format!("{}.constructs[{}].name", context, construct_idx));
                }
            }
        }
        _ => {}
    }
}

fn collect_field_string_refs(
    fields: &[ObjField],
    target: usize,
    context: &str,
    refs: &mut Vec<String>,
) {
    for (field_idx, field) in fields.iter().enumerate() {
        if field.name.0 == target {
            refs.push(format!("{}[{}].name", context, field_idx));
        }
    }
}

pub fn int_references(bytecode: &Bytecode, target: usize) -> Vec<String> {
    opcode_pool_references(bytecode, |op| match op {
        Opcode::Int { ptr, .. } if ptr.0 == target => true,
        _ => false,
    })
}

pub fn float_references(bytecode: &Bytecode, target: usize) -> Vec<String> {
    opcode_pool_references(bytecode, |op| match op {
        Opcode::Float { ptr, .. } if ptr.0 == target => true,
        _ => false,
    })
}

#[allow(dead_code)]
pub fn bytes_references(bytecode: &Bytecode, target: usize) -> Vec<String> {
    opcode_pool_references(bytecode, |op| match op {
        Opcode::Bytes { ptr, .. } if ptr.0 == target => true,
        _ => false,
    })
}

fn opcode_pool_references<F>(bytecode: &Bytecode, matches_target: F) -> Vec<String>
where
    F: Fn(&Opcode) -> bool,
{
    let mut refs = Vec::new();
    for (function_idx, function) in bytecode.functions.iter().enumerate() {
        for (op_idx, op) in function.ops.iter().enumerate() {
            if matches_target(op) {
                refs.push(format!("function[{}].ops[{}]", function_idx, op_idx));
            }
        }
    }
    refs
}
