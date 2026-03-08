//! IR Builder - converts AST to IR

use crate::ir::{IRFunction, IRInstruction, IRModule, IRType, IRValue};
use crate::parser::ast::*;
use crate::stdlib::Stdlib;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
enum ParamLayout {
    Scalar {
        name: String,
        is_string: bool,
    },
    Struct {
        name: String,
        struct_name: String,
        fields: Vec<String>,
    },
}

#[derive(Debug, Clone)]
struct StructReturnTemplate {
    struct_name: String,
    params: Vec<String>,
    expr: Expr,
}

pub struct IRBuilder {
    module: IRModule,
    var_counter: usize,
    label_counter: usize,
    variables: HashMap<String, String>,
    defined_functions: HashSet<String>,
    builtin_functions: HashSet<String>,
    builtin_return_types: HashMap<String, String>,
    last_expr_value: Option<IRValue>,
    // Track if current block has a terminator (return/branch)
    current_block_terminated: bool,
    struct_defs: HashMap<String, Vec<String>>,
    struct_var_types: HashMap<String, String>,
    struct_field_ptrs: HashMap<(String, String), String>,
    string_variables: HashSet<String>,
    declared_string_variables: HashSet<String>,
    function_param_layouts: HashMap<String, Vec<ParamLayout>>,
    function_return_types: HashMap<String, String>,
    struct_return_templates: HashMap<String, StructReturnTemplate>,
}

impl IRBuilder {
    const PRINT_I64: &'static str = "ziv_print_i64";
    const PRINTLN_I64: &'static str = "ziv_println_i64";
    const PRINT_STR: &'static str = "ziv_print_str";
    const PRINTLN_STR: &'static str = "ziv_println_str";
    const EPRINT_I64: &'static str = "ziv_eprint_i64";
    const EPRINTLN_I64: &'static str = "ziv_eprintln_i64";
    const EPRINT_STR: &'static str = "ziv_eprint_str";
    const EPRINTLN_STR: &'static str = "ziv_eprintln_str";

    fn is_runtime_container_builtin(name: &str) -> bool {
        matches!(
            name,
            "vectorNew"
                | "vectorLen"
                | "vectorPush"
                | "vectorPop"
                | "vectorGet"
                | "vectorSet"
                | "vectorInsert"
                | "vectorRemove"
                | "vectorContains"
                | "vectorClear"
                | "hashMapNew"
                | "hashMapLen"
                | "hashMapSet"
                | "hashMapGet"
                | "hashMapHas"
                | "hashMapRemove"
                | "hashMapKeys"
                | "hashMapValues"
                | "hashMapClear"
                | "hashMapMerge"
        )
    }

    fn is_runtime_string_builtin(name: &str) -> bool {
        matches!(
            name,
            "strlen"
                | "concat"
                | "substr"
                | "char_at"
                | "to_upper"
                | "to_lower"
                | "trim"
                | "contains"
        )
    }

    fn is_runtime_math_builtin(name: &str) -> bool {
        matches!(
            name,
            "abs" | "min" | "max" | "sqrt" | "pow" | "floor" | "ceil" | "round"
        )
    }

    fn is_runtime_array_builtin(name: &str) -> bool {
        matches!(
            name,
            "push" | "pop" | "arrlen" | "get" | "set" | "first" | "last" | "reverse"
        )
    }

    fn is_runtime_utils_builtin(name: &str) -> bool {
        matches!(
            name,
            "parseInt"
                | "parseFloat"
                | "isNaN"
                | "isFinite"
                | "Number"
                | "String"
                | "Boolean"
                | "jsonParse"
                | "jsonStringify"
                | "includes"
                | "indexOf"
                | "startsWith"
                | "endsWith"
                | "split"
                | "replace"
                | "map"
                | "filter"
                | "reduce"
        )
    }

    fn is_runtime_io_builtin(name: &str) -> bool {
        matches!(
            name,
            "read" | "eprint" | "eprintln" | "input" | "readAll" | "printf" | "flush"
        )
    }

    fn is_runtime_filesystem_builtin(name: &str) -> bool {
        matches!(
            name,
            "readFile"
                | "writeFile"
                | "appendFile"
                | "exists"
                | "mkdir"
                | "readDir"
                | "removeFile"
                | "removeDir"
                | "rename"
                | "copyFile"
                | "fileSize"
                | "cwd"
        )
    }

    fn is_runtime_net_builtin(name: &str) -> bool {
        matches!(
            name,
            "fetch"
                | "httpGet"
                | "httpPost"
                | "httpPut"
                | "httpDelete"
                | "download"
                | "upload"
                | "websocketConnect"
                | "dnsLookup"
                | "ping"
        )
    }

    fn is_runtime_crypto_builtin(name: &str) -> bool {
        matches!(
            name,
            "md5"
                | "sha1"
                | "sha256"
                | "sha512"
                | "hmacSha256"
                | "pbkdf2"
                | "encryptAES"
                | "decryptAES"
                | "sign"
                | "verify"
                | "randomBytes"
                | "randomUUID"
        )
    }

    fn is_runtime_encoding_builtin(name: &str) -> bool {
        matches!(
            name,
            "base64Encode"
                | "base64Decode"
                | "hexEncode"
                | "hexDecode"
                | "urlEncode"
                | "urlDecode"
                | "utf8Encode"
                | "utf8Decode"
                | "csvEncode"
                | "csvDecode"
                | "queryStringify"
                | "queryParse"
        )
    }

    fn runtime_builtin_symbol(name: &str) -> String {
        match name {
            "mkdir" => "ziv_mkdir".to_string(),
            "rename" => "ziv_rename".to_string(),
            "abs" => "ziv_abs".to_string(),
            "min" => "ziv_min".to_string(),
            "max" => "ziv_max".to_string(),
            "sqrt" => "ziv_sqrt".to_string(),
            "pow" => "ziv_pow".to_string(),
            "floor" => "ziv_floor".to_string(),
            "ceil" => "ziv_ceil".to_string(),
            "round" => "ziv_round".to_string(),
            "push" => "ziv_array_push".to_string(),
            "pop" => "ziv_array_pop".to_string(),
            "arrlen" => "ziv_array_len".to_string(),
            "get" => "ziv_array_get".to_string(),
            "set" => "ziv_array_set".to_string(),
            "first" => "ziv_array_first".to_string(),
            "last" => "ziv_array_last".to_string(),
            "reverse" => "ziv_array_reverse".to_string(),
            "parseInt" => "ziv_parse_int".to_string(),
            "parseFloat" => "ziv_parse_float".to_string(),
            "isNaN" => "ziv_is_nan".to_string(),
            "isFinite" => "ziv_is_finite".to_string(),
            "Number" => "ziv_number".to_string(),
            "String" => "ziv_string".to_string(),
            "Boolean" => "ziv_boolean".to_string(),
            "jsonParse" => "ziv_json_parse".to_string(),
            "jsonStringify" => "ziv_json_stringify".to_string(),
            "includes" => "ziv_includes".to_string(),
            "indexOf" => "ziv_index_of".to_string(),
            "startsWith" => "ziv_starts_with".to_string(),
            "endsWith" => "ziv_ends_with".to_string(),
            "split" => "ziv_split".to_string(),
            "replace" => "ziv_replace".to_string(),
            "map" => "ziv_map".to_string(),
            "filter" => "ziv_filter".to_string(),
            "reduce" => "ziv_reduce".to_string(),
            "read" => "ziv_read".to_string(),
            "eprint" => "ziv_eprint".to_string(),
            "eprintln" => "ziv_eprintln".to_string(),
            "input" => "ziv_input".to_string(),
            "readAll" => "ziv_read_all".to_string(),
            "printf" => "ziv_printf".to_string(),
            "flush" => "ziv_flush".to_string(),
            _ => name.to_string(),
        }
    }

    fn type_name_is_string(type_name: &str) -> bool {
        type_name.eq_ignore_ascii_case("string")
    }

    pub fn new() -> Self {
        let stdlib = Stdlib::new();
        let mut builtin_functions = HashSet::new();
        let mut builtin_return_types = HashMap::new();
        for func in stdlib.all_functions() {
            builtin_functions.insert(func.name.clone());
            if let Some(ret) = &func.return_type {
                builtin_return_types.insert(func.name.clone(), ret.clone());
            }
        }

        IRBuilder {
            module: IRModule::new(),
            var_counter: 0,
            label_counter: 0,
            variables: HashMap::new(),
            defined_functions: HashSet::new(),
            builtin_functions,
            builtin_return_types,
            last_expr_value: None,
            current_block_terminated: false,
            struct_defs: HashMap::new(),
            struct_var_types: HashMap::new(),
            struct_field_ptrs: HashMap::new(),
            string_variables: HashSet::new(),
            declared_string_variables: HashSet::new(),
            function_param_layouts: HashMap::new(),
            function_return_types: HashMap::new(),
            struct_return_templates: HashMap::new(),
        }
    }

    fn fresh_var(&mut self) -> String {
        let name = format!("t{}", self.var_counter);
        self.var_counter += 1;
        name
    }

    fn fresh_label(&mut self) -> String {
        let name = format!("L{}", self.label_counter);
        self.label_counter += 1;
        name
    }

    fn add_instr(&mut self, func: &mut IRFunction, instr: IRInstruction) {
        // Label always starts a new block, even if previous was terminated
        if let IRInstruction::Label(label_name) = &instr {
            // If previous block wasn't terminated, add a jump to this label
            // This handles fall-through between basic blocks
            if !self.current_block_terminated {
                func.add_instruction(IRInstruction::Jump(label_name.clone()));
            }
            self.current_block_terminated = false;
            func.add_instruction(instr);
            return;
        }

        // Don't add other instructions if current block is already terminated
        if self.current_block_terminated {
            return;
        }

        // Check if this instruction terminates the block
        match &instr {
            IRInstruction::Ret { .. }
            | IRInstruction::Jump(_)
            | IRInstruction::CondBranch { .. } => {
                self.current_block_terminated = true;
            }
            _ => {}
        }

        func.add_instruction(instr);
    }

    pub fn build(mut self, program: &Program) -> IRModule {
        for stmt in &program.statements {
            if let Stmt::StructDecl { name, fields } = stmt {
                self.register_struct_decl(name, fields);
            }
        }
        self.collect_function_metadata(program);

        self.defined_functions = program
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::FunctionDecl { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();

        // First pass: collect all function definitions
        for stmt in &program.statements {
            if let Stmt::FunctionDecl {
                name, params, body, ..
            } = stmt
            {
                // Reset state for each function
                self.current_block_terminated = false;
                self.var_counter = 0;
                self.label_counter = 0;
                self.variables.clear();
                self.struct_var_types.clear();
                self.struct_field_ptrs.clear();
                self.string_variables.clear();
                self.declared_string_variables.clear();

                let mut func = IRFunction::new(name.clone(), IRType::I64);
                let layouts = self
                    .function_param_layouts
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| {
                        params
                            .iter()
                            .map(|param| ParamLayout::Scalar {
                                name: param.name.clone(),
                                is_string: param
                                    .type_annotation
                                    .as_deref()
                                    .map(Self::type_name_is_string)
                                    .unwrap_or(false),
                            })
                            .collect()
                    });
                let mut arg_index = 0usize;

                for layout in layouts {
                    match layout {
                        ParamLayout::Scalar { name, is_string } => {
                            let ptr = format!("arg{}", arg_index);
                            arg_index += 1;
                            func.params.push((ptr.clone(), IRType::I64));
                            self.add_instr(
                                &mut func,
                                IRInstruction::Alloc {
                                    dest: ptr.clone(),
                                    ty: IRType::I64,
                                },
                            );
                            self.variables.insert(name.clone(), ptr);
                            if is_string {
                                self.string_variables.insert(name.clone());
                                self.declared_string_variables.insert(name);
                            }
                        }
                        ParamLayout::Struct {
                            name,
                            struct_name,
                            fields,
                        } => {
                            self.struct_var_types.insert(name.clone(), struct_name);
                            for field in fields {
                                let ptr = format!("arg{}", arg_index);
                                arg_index += 1;
                                func.params.push((ptr.clone(), IRType::I64));
                                self.add_instr(
                                    &mut func,
                                    IRInstruction::Alloc {
                                        dest: ptr.clone(),
                                        ty: IRType::I64,
                                    },
                                );
                                self.struct_field_ptrs
                                    .insert((name.clone(), field), ptr.clone());
                            }
                        }
                    }
                }

                // Build function body
                for body_stmt in body {
                    self.build_stmt(body_stmt, &mut func);
                }

                // Add implicit return if not present
                self.add_instr(
                    &mut func,
                    IRInstruction::Ret {
                        ty: IRType::I64,
                        value: Some(IRValue::Const(0)),
                    },
                );

                self.module.add_function(func);
            }
        }

        // Second pass: build main function with non-function statements
        self.current_block_terminated = false;
        self.var_counter = 0;
        self.variables.clear();
        self.struct_var_types.clear();
        self.struct_field_ptrs.clear();
        self.string_variables.clear();
        self.declared_string_variables.clear();

        // Use _user_main to avoid conflict with C runtime's main
        let mut main_func = IRFunction::new("_user_main".to_string(), IRType::I64);

        for stmt in &program.statements {
            match stmt {
                Stmt::FunctionDecl { .. } | Stmt::Import { .. } | Stmt::StructDecl { .. } => {} // Skip, already processed
                _ => self.build_stmt(stmt, &mut main_func),
            }
        }

        self.add_instr(
            &mut main_func,
            IRInstruction::Ret {
                ty: IRType::I64,
                // Keep process exit deterministic; examples that represent importable modules
                // should still run successfully with code 0.
                value: Some(IRValue::Const(0)),
            },
        );

        self.module.add_function(main_func);
        self.module
    }

    fn register_struct_decl(&mut self, name: &str, fields: &[StructFieldDecl]) {
        let field_names = fields.iter().map(|field| field.name.clone()).collect();
        self.struct_defs.insert(name.to_string(), field_names);
    }

    fn collect_function_metadata(&mut self, program: &Program) {
        self.function_param_layouts.clear();
        self.function_return_types.clear();
        self.struct_return_templates.clear();

        for stmt in &program.statements {
            let Stmt::FunctionDecl {
                name,
                params,
                return_type,
                body,
            } = stmt
            else {
                continue;
            };

            let mut layouts = Vec::new();
            for param in params {
                if let Some(type_name) = &param.type_annotation {
                    if let Some(fields) = self.struct_defs.get(type_name).cloned() {
                        layouts.push(ParamLayout::Struct {
                            name: param.name.clone(),
                            struct_name: type_name.clone(),
                            fields,
                        });
                        continue;
                    }
                }
                layouts.push(ParamLayout::Scalar {
                    name: param.name.clone(),
                    is_string: param
                        .type_annotation
                        .as_deref()
                        .map(Self::type_name_is_string)
                        .unwrap_or(false),
                });
            }
            self.function_param_layouts.insert(name.clone(), layouts);

            if let Some(ret_type) = return_type {
                self.function_return_types
                    .insert(name.clone(), ret_type.clone());
            }

            if let Some(ret_type) = return_type {
                if self.struct_defs.contains_key(ret_type) {
                    if let Some(expr) = Self::extract_simple_struct_return_expr(body, ret_type) {
                        self.struct_return_templates.insert(
                            name.clone(),
                            StructReturnTemplate {
                                struct_name: ret_type.clone(),
                                params: params.iter().map(|param| param.name.clone()).collect(),
                                expr,
                            },
                        );
                    }
                }
            }
        }
    }

    fn extract_simple_struct_return_expr(body: &[Stmt], expected_struct: &str) -> Option<Expr> {
        if body.len() != 1 {
            return None;
        }

        match &body[0] {
            Stmt::Return(Some(expr)) => match expr {
                Expr::StructInit { struct_name, .. } if struct_name == expected_struct => {
                    Some(expr.clone())
                }
                Expr::Identifier(_) => Some(expr.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    fn substitute_expr(expr: &Expr, bindings: &HashMap<String, Expr>) -> Expr {
        match expr {
            Expr::Identifier(name) => bindings
                .get(name)
                .cloned()
                .unwrap_or_else(|| Expr::Identifier(name.clone())),
            Expr::Literal(lit) => Expr::Literal(lit.clone()),
            Expr::Binary { left, op, right } => Expr::Binary {
                left: Box::new(Self::substitute_expr(left, bindings)),
                op: op.clone(),
                right: Box::new(Self::substitute_expr(right, bindings)),
            },
            Expr::Call { callee, args } => Expr::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::substitute_expr(arg, bindings))
                    .collect(),
            },
            Expr::StructInit {
                struct_name,
                fields,
            } => Expr::StructInit {
                struct_name: struct_name.clone(),
                fields: fields
                    .iter()
                    .map(|field| StructFieldInit {
                        name: field.name.clone(),
                        value: Self::substitute_expr(&field.value, bindings),
                    })
                    .collect(),
            },
            Expr::FieldAccess { object, field } => Expr::FieldAccess {
                object: Box::new(Self::substitute_expr(object, bindings)),
                field: field.clone(),
            },
        }
    }

    fn resolve_struct_call_expr(
        &self,
        callee: &str,
        args: &[Expr],
        expected_struct: &str,
    ) -> Option<Expr> {
        let template = self.struct_return_templates.get(callee)?;
        if template.struct_name != expected_struct {
            return None;
        }

        let mut bindings = HashMap::new();
        for (idx, param_name) in template.params.iter().enumerate() {
            if let Some(arg) = args.get(idx) {
                bindings.insert(param_name.clone(), arg.clone());
            }
        }

        Some(Self::substitute_expr(&template.expr, &bindings))
    }

    fn load_struct_field_value(
        &mut self,
        var_name: &str,
        field: &str,
        func: &mut IRFunction,
    ) -> IRValue {
        if let Some(ptr) = self
            .struct_field_ptrs
            .get(&(var_name.to_string(), field.to_string()))
            .cloned()
        {
            let dest = self.fresh_var();
            self.add_instr(
                func,
                IRInstruction::Load {
                    dest: dest.clone(),
                    ty: IRType::I64,
                    ptr,
                },
            );
            IRValue::Var(dest)
        } else {
            IRValue::Const(0)
        }
    }

    fn resolve_struct_var_type(
        &self,
        type_annotation: &Option<String>,
        init: &Option<Expr>,
    ) -> Option<String> {
        if let Some(type_name) = type_annotation {
            if self.struct_defs.contains_key(type_name) {
                return Some(type_name.clone());
            }
        }

        if let Some(Expr::StructInit { struct_name, .. }) = init {
            if self.struct_defs.contains_key(struct_name) {
                return Some(struct_name.clone());
            }
        }

        if let Some(Expr::Call { callee, .. }) = init {
            if let Some(template) = self.struct_return_templates.get(callee) {
                return Some(template.struct_name.clone());
            }
        }

        None
    }

    fn build_struct_var_decl(
        &mut self,
        func: &mut IRFunction,
        var_name: &str,
        struct_name: &str,
        init: Option<&Expr>,
    ) {
        let Some(field_order) = self.struct_defs.get(struct_name).cloned() else {
            return;
        };

        self.struct_var_types
            .insert(var_name.to_string(), struct_name.to_string());

        let resolved_call_expr = match init {
            Some(Expr::Call { callee, args }) => {
                self.resolve_struct_call_expr(callee, args, struct_name)
            }
            _ => None,
        };

        for field in field_order {
            let ptr = self.fresh_var();
            self.add_instr(
                func,
                IRInstruction::Alloc {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                },
            );

            let value = match init {
                Some(Expr::StructInit { fields, .. }) => self
                    .find_struct_field_init(fields, &field)
                    .map(|expr| self.build_expr(expr, func))
                    .unwrap_or(IRValue::Const(0)),
                Some(Expr::Call { .. }) => match resolved_call_expr.as_ref() {
                    Some(Expr::StructInit { fields, .. }) => self
                        .find_struct_field_init(fields, &field)
                        .map(|expr| self.build_expr(expr, func))
                        .unwrap_or(IRValue::Const(0)),
                    Some(Expr::Identifier(var_name)) => {
                        self.load_struct_field_value(var_name, &field, func)
                    }
                    _ => IRValue::Const(0),
                },
                _ => IRValue::Const(0),
            };

            self.add_instr(
                func,
                IRInstruction::Store {
                    dest: ptr.clone(),
                    ty: IRType::I64,
                    value,
                },
            );
            self.struct_field_ptrs
                .insert((var_name.to_string(), field), ptr);
        }
    }

    fn find_struct_field_init<'a>(
        &self,
        fields: &'a [StructFieldInit],
        name: &str,
    ) -> Option<&'a Expr> {
        fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| &field.value)
    }

    fn builtin_returns_string(&self, name: &str) -> bool {
        self.builtin_return_types
            .get(name)
            .map(|ty| Self::type_name_is_string(ty))
            .unwrap_or(false)
    }

    fn function_returns_string(&self, name: &str) -> bool {
        self.function_return_types
            .get(name)
            .map(|ty| Self::type_name_is_string(ty))
            .unwrap_or(false)
    }

    fn expr_is_string(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(Literal::String(_)) => true,
            Expr::Identifier(name) => {
                self.string_variables.contains(name)
                    || self.declared_string_variables.contains(name)
            }
            Expr::Call { callee, .. } => {
                self.builtin_returns_string(callee) || self.function_returns_string(callee)
            }
            Expr::Binary { .. }
            | Expr::Literal(_)
            | Expr::StructInit { .. }
            | Expr::FieldAccess { .. } => false,
        }
    }

    fn apply_struct_update(
        &mut self,
        func: &mut IRFunction,
        var_name: &str,
        struct_name: &str,
        fields: &[StructFieldInit],
        partial: bool,
    ) {
        let Some(var_struct_name) = self.struct_var_types.get(var_name).cloned() else {
            return;
        };
        if var_struct_name != struct_name {
            return;
        }

        if partial {
            for field in fields {
                if let Some(ptr) = self
                    .struct_field_ptrs
                    .get(&(var_name.to_string(), field.name.clone()))
                    .cloned()
                {
                    let value = self.build_expr(&field.value, func);
                    self.add_instr(
                        func,
                        IRInstruction::Store {
                            dest: ptr,
                            ty: IRType::I64,
                            value,
                        },
                    );
                }
            }
            return;
        }

        let Some(field_order) = self.struct_defs.get(&var_struct_name).cloned() else {
            return;
        };
        for field_name in field_order {
            if let Some(ptr) = self
                .struct_field_ptrs
                .get(&(var_name.to_string(), field_name.clone()))
                .cloned()
            {
                let value = self
                    .find_struct_field_init(fields, &field_name)
                    .map(|expr| self.build_expr(expr, func))
                    .unwrap_or(IRValue::Const(0));
                self.add_instr(
                    func,
                    IRInstruction::Store {
                        dest: ptr,
                        ty: IRType::I64,
                        value,
                    },
                );
            }
        }
    }

    fn lower_struct_arg_values(
        &mut self,
        arg_expr: Option<&Expr>,
        field_order: &[String],
        func: &mut IRFunction,
    ) -> Vec<IRValue> {
        let mut values = Vec::new();

        for field in field_order {
            let value = match arg_expr {
                Some(Expr::Identifier(var_name)) => {
                    self.load_struct_field_value(var_name, field, func)
                }
                Some(Expr::StructInit { fields, .. }) => self
                    .find_struct_field_init(fields, field)
                    .map(|expr| self.build_expr(expr, func))
                    .unwrap_or(IRValue::Const(0)),
                _ => IRValue::Const(0),
            };
            values.push(value);
        }

        values
    }

    fn lower_user_call_args(
        &mut self,
        callee: &str,
        args: &[Expr],
        func: &mut IRFunction,
    ) -> Vec<IRValue> {
        let Some(layouts) = self.function_param_layouts.get(callee).cloned() else {
            return args
                .iter()
                .map(|arg| self.build_expr(arg, func))
                .collect::<Vec<_>>();
        };

        let mut values = Vec::new();
        for (idx, layout) in layouts.iter().enumerate() {
            let arg_expr = args.get(idx);
            match layout {
                ParamLayout::Scalar { .. } => {
                    values.push(
                        arg_expr
                            .map(|expr| self.build_expr(expr, func))
                            .unwrap_or(IRValue::Const(0)),
                    );
                }
                ParamLayout::Struct { fields, .. } => {
                    values.extend(self.lower_struct_arg_values(arg_expr, fields, func));
                }
            }
        }

        if args.len() > layouts.len() {
            for extra in &args[layouts.len()..] {
                values.push(self.build_expr(extra, func));
            }
        }

        values
    }

    fn build_stmt(&mut self, stmt: &Stmt, func: &mut IRFunction) {
        match stmt {
            Stmt::Import { .. } => {}

            Stmt::StructDecl { name, fields } => {
                self.register_struct_decl(name, fields);
            }

            Stmt::Expression(expr) => {
                let value = self.build_expr(expr, func);
                self.last_expr_value = Some(value);
            }

            Stmt::VariableDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                if let Some(struct_name) = self.resolve_struct_var_type(type_annotation, init) {
                    self.build_struct_var_decl(func, name, &struct_name, init.as_ref());
                    return;
                }

                let declared_as_string = type_annotation
                    .as_deref()
                    .map(Self::type_name_is_string)
                    .unwrap_or(false);
                if declared_as_string {
                    self.declared_string_variables.insert(name.clone());
                } else {
                    self.declared_string_variables.remove(name);
                }

                let ptr = self.fresh_var();
                self.add_instr(
                    func,
                    IRInstruction::Alloc {
                        dest: ptr.clone(),
                        ty: IRType::I64,
                    },
                );

                if let Some(init_expr) = init {
                    let init_is_string = self.expr_is_string(init_expr) || declared_as_string;
                    let value = self.build_expr(init_expr, func);
                    self.add_instr(
                        func,
                        IRInstruction::Store {
                            dest: ptr.clone(),
                            ty: IRType::I64,
                            value,
                        },
                    );
                    self.last_expr_value = Some(IRValue::Var(ptr.clone()));
                    if init_is_string {
                        self.string_variables.insert(name.clone());
                    } else {
                        self.string_variables.remove(name);
                    }
                } else if declared_as_string {
                    self.string_variables.insert(name.clone());
                } else {
                    self.string_variables.remove(name);
                }

                self.variables.insert(name.clone(), ptr);
            }

            Stmt::Assignment { name, value } => {
                if self.struct_var_types.contains_key(name) {
                    if let Expr::StructInit {
                        struct_name,
                        fields,
                    } = value
                    {
                        self.apply_struct_update(func, name, struct_name, fields, false);
                    }
                    return;
                }

                if let Some(ptr) = self.variables.get(name).cloned() {
                    let value_is_string =
                        self.expr_is_string(value) || self.declared_string_variables.contains(name);
                    let val = self.build_expr(value, func);
                    self.add_instr(
                        func,
                        IRInstruction::Store {
                            dest: ptr,
                            ty: IRType::I64,
                            value: val,
                        },
                    );
                    if value_is_string {
                        self.string_variables.insert(name.clone());
                    } else {
                        self.string_variables.remove(name);
                    }
                }
            }

            Stmt::StructMergeAssign { name, value } => {
                if let Expr::StructInit {
                    struct_name,
                    fields,
                } = value
                {
                    self.apply_struct_update(func, name, struct_name, fields, true);
                }
            }

            Stmt::FunctionDecl { .. } => {}

            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    Some(self.build_expr(e, func))
                } else {
                    None
                };
                self.add_instr(
                    func,
                    IRInstruction::Ret {
                        ty: IRType::I64,
                        value,
                    },
                );
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.build_expr(condition, func);
                let then_label = self.fresh_label();
                let else_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.add_instr(
                    func,
                    IRInstruction::CondBranch {
                        condition: cond_val,
                        true_label: then_label.clone(),
                        false_label: else_label.clone(),
                    },
                );

                // Then branch
                self.add_instr(func, IRInstruction::Label(then_label));
                for stmt in then_branch {
                    self.build_stmt(stmt, func);
                }
                // Only add jump if block wasn't terminated by return
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(end_label.clone()));
                }

                // Else branch
                self.add_instr(func, IRInstruction::Label(else_label));
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.build_stmt(stmt, func);
                    }
                }
                // Only add jump if block wasn't terminated by return
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(end_label.clone()));
                }

                // End label
                self.add_instr(func, IRInstruction::Label(end_label));
            }

            Stmt::While { condition, body } => {
                let start_label = self.fresh_label();
                let body_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.add_instr(func, IRInstruction::Label(start_label.clone()));
                let cond_val = self.build_expr(condition, func);
                self.add_instr(
                    func,
                    IRInstruction::CondBranch {
                        condition: cond_val,
                        true_label: body_label.clone(),
                        false_label: end_label.clone(),
                    },
                );

                self.add_instr(func, IRInstruction::Label(body_label));
                for stmt in body {
                    self.build_stmt(stmt, func);
                }
                // Jump back to start if not terminated
                if !self.current_block_terminated {
                    self.add_instr(func, IRInstruction::Jump(start_label));
                }

                self.add_instr(func, IRInstruction::Label(end_label));
            }

            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.build_stmt(stmt, func);
                }
            }
        }
    }

    fn build_expr(&mut self, expr: &Expr, func: &mut IRFunction) -> IRValue {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => IRValue::Const(*n),
                Literal::String(s) => IRValue::Str(s.clone()),
                _ => IRValue::Const(0),
            },

            Expr::Identifier(name) => {
                if let Some(ptr) = self.variables.get(name).cloned() {
                    let dest = self.fresh_var();
                    self.add_instr(
                        func,
                        IRInstruction::Load {
                            dest: dest.clone(),
                            ty: IRType::I64,
                            ptr: ptr,
                        },
                    );
                    IRValue::Var(dest)
                } else if self.defined_functions.contains(name) {
                    IRValue::Func(name.clone())
                } else {
                    IRValue::Const(0)
                }
            }

            Expr::StructInit { .. } => IRValue::Const(0),

            Expr::FieldAccess { object, field } => {
                if let Expr::Identifier(var_name) = object.as_ref() {
                    return self.load_struct_field_value(var_name, field, func);
                }
                IRValue::Const(0)
            }

            Expr::Binary { left, op, right } => {
                let lhs = self.build_expr(left, func);
                let rhs = self.build_expr(right, func);
                let dest = self.fresh_var();

                let instr = match op {
                    BinaryOp::Add => IRInstruction::Add {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Sub => IRInstruction::Sub {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Mul => IRInstruction::Mul {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Div => IRInstruction::Div {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Eq => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Eq,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Ne => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Ne,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Lt => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Lt,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Le => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Le,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Gt => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Gt,
                        lhs,
                        rhs,
                    },
                    BinaryOp::Ge => IRInstruction::Cmp {
                        dest: dest.clone(),
                        op: crate::ir::CmpOp::Ge,
                        lhs,
                        rhs,
                    },
                    _ => IRInstruction::Add {
                        dest: dest.clone(),
                        ty: IRType::I64,
                        lhs: IRValue::Const(0),
                        rhs: IRValue::Const(0),
                    },
                };

                self.add_instr(func, instr);
                IRValue::Var(dest)
            }

            Expr::Call { callee, args } => {
                // `callee` may refer to a function pointer variable (higher-order call).
                let is_indirect = self.variables.contains_key(callee);
                let arg_values = if is_indirect {
                    args.iter().map(|arg| self.build_expr(arg, func)).collect()
                } else {
                    self.lower_user_call_args(callee, args, func)
                };

                // Keep most built-ins semantic-only for now.
                // Runtime-backed built-ins are lowered to runtime calls.
                if !is_indirect
                    && self.builtin_functions.contains(callee)
                    && !self.defined_functions.contains(callee)
                {
                    if matches!(callee.as_str(), "print" | "println" | "eprint" | "eprintln") {
                        if let Some(value) = arg_values.first() {
                            let arg_is_string = args
                                .first()
                                .map(|arg| self.expr_is_string(arg))
                                .unwrap_or(false)
                                || matches!(value, IRValue::Str(_));
                            let function = match (callee.as_str(), value) {
                                ("print", _) if arg_is_string => Self::PRINT_STR,
                                ("println", _) if arg_is_string => Self::PRINTLN_STR,
                                ("eprint", _) if arg_is_string => Self::EPRINT_STR,
                                ("eprintln", _) if arg_is_string => Self::EPRINTLN_STR,
                                ("print", _) => Self::PRINT_I64,
                                ("println", _) => Self::PRINTLN_I64,
                                ("eprint", _) => Self::EPRINT_I64,
                                ("eprintln", _) => Self::EPRINTLN_I64,
                                _ => unreachable!(),
                            };

                            self.add_instr(
                                func,
                                IRInstruction::Call {
                                    result: None,
                                    function: function.to_string(),
                                    args: arg_values,
                                },
                            );
                        }
                        return IRValue::Const(0);
                    }

                    if Self::is_runtime_container_builtin(callee)
                        || Self::is_runtime_math_builtin(callee)
                        || Self::is_runtime_array_builtin(callee)
                        || Self::is_runtime_string_builtin(callee)
                        || Self::is_runtime_utils_builtin(callee)
                        || Self::is_runtime_io_builtin(callee)
                        || Self::is_runtime_filesystem_builtin(callee)
                        || Self::is_runtime_net_builtin(callee)
                        || Self::is_runtime_crypto_builtin(callee)
                        || Self::is_runtime_encoding_builtin(callee)
                    {
                        let dest = self.fresh_var();
                        let runtime_symbol = Self::runtime_builtin_symbol(callee);
                        self.add_instr(
                            func,
                            IRInstruction::Call {
                                result: Some(dest.clone()),
                                function: runtime_symbol,
                                args: arg_values,
                            },
                        );
                        return IRValue::Var(dest);
                    }

                    return IRValue::Const(0);
                }

                let dest = self.fresh_var();
                if is_indirect {
                    let function_value = self.build_expr(&Expr::Identifier(callee.clone()), func);
                    self.add_instr(
                        func,
                        IRInstruction::CallIndirect {
                            result: Some(dest.clone()),
                            function: function_value,
                            args: arg_values,
                        },
                    );
                } else {
                    self.add_instr(
                        func,
                        IRInstruction::Call {
                            result: Some(dest.clone()),
                            function: callee.clone(),
                            args: arg_values,
                        },
                    );
                }
                IRValue::Var(dest)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_builtin_call_lowers_println_to_runtime_helper() {
        let mut parser = Parser::new(
            r#"
            function helper() { return 1; }
            helper();
            println(42);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_direct_println_call = main.instructions.iter().any(|instr| match instr {
            IRInstruction::Call { function, .. } => function == "println",
            _ => false,
        });
        let has_runtime_println = main.instructions.iter().any(|instr| match instr {
            IRInstruction::Call { function, .. } => function == IRBuilder::PRINTLN_I64,
            _ => false,
        });
        assert!(!has_direct_println_call);
        assert!(has_runtime_println);
    }

    #[test]
    fn test_build_no_else_block_div_eq_and_nested_function_decl_paths() {
        let nested = Stmt::FunctionDecl {
            name: "inner".to_string(),
            params: vec![],
            return_type: None,
            body: vec![Stmt::Return(Some(Expr::Literal(Literal::Number(1))))],
        };
        let outer = Stmt::FunctionDecl {
            name: "outer".to_string(),
            params: vec![],
            return_type: None,
            body: vec![nested, Stmt::Return(None)],
        };

        let program = Program::new(vec![
            outer,
            Stmt::If {
                condition: Expr::Literal(Literal::Boolean(true)),
                then_branch: vec![Stmt::Expression(Expr::Literal(Literal::Number(1)))],
                else_branch: None,
            },
            Stmt::Block(vec![Stmt::Expression(Expr::Literal(Literal::Number(2)))]),
            Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(8))),
                op: BinaryOp::Div,
                right: Box::new(Expr::Literal(Literal::Number(2))),
            }),
            Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            }),
        ]);

        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::Div { .. })));
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Cmp {
                op: crate::ir::CmpOp::Eq,
                ..
            }
        )));

        let outer_fn = module
            .functions
            .iter()
            .find(|func| func.name == "outer")
            .unwrap();
        assert!(outer_fn
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::Ret { value: None, .. })));
    }

    #[test]
    fn test_user_defined_function_call_is_preserved() {
        let mut parser = Parser::new(
            r#"
            function print(x) { return x; }
            print(1);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_user_print_call = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "print"
            )
        });
        assert!(has_user_print_call);
    }

    #[test]
    fn test_string_print_lowering_uses_string_runtime_helper() {
        let mut parser = Parser::new(
            r#"
            print("a");
            println("b");
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_print_str = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, args, .. }
                    if function == IRBuilder::PRINT_STR
                        && matches!(args.first(), Some(IRValue::Str(value)) if value == "a")
            )
        });
        let has_println_str = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, args, .. }
                    if function == IRBuilder::PRINTLN_STR
                        && matches!(args.first(), Some(IRValue::Str(value)) if value == "b")
            )
        });

        assert!(has_print_str);
        assert!(has_println_str);
    }

    #[test]
    fn test_println_fetch_lowers_to_string_runtime_helper() {
        let mut parser = Parser::new(
            r#"
            println(fetch("https://baidu.com"));
            let resp = httpGet("https://baidu.com");
            println(resp);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let fetch_call_exists = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "fetch"
            )
        });
        let http_get_call_exists = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "httpGet"
            )
        });
        let string_println_calls = main
            .instructions
            .iter()
            .filter(|instr| {
                matches!(
                    instr,
                    IRInstruction::Call { function, .. } if function == IRBuilder::PRINTLN_STR
                )
            })
            .count();
        let i64_println_call_exists = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == IRBuilder::PRINTLN_I64
            )
        });

        assert!(fetch_call_exists);
        assert!(http_get_call_exists);
        assert_eq!(string_println_calls, 2);
        assert!(!i64_println_call_exists);
    }

    #[test]
    fn test_build_control_flow_and_assignment_paths() {
        let mut parser = Parser::new(
            r#"
            let x;
            x = 1;
            if (x != 0) { x = x + 1; } else { x = x - 1; }
            while (x > 0) { x = x - 1; }
            x;
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::CondBranch { .. })));
        assert!(main
            .instructions
            .iter()
            .any(|i| matches!(i, IRInstruction::Jump(_))));
    }

    #[test]
    fn test_build_literal_fallback_and_unknown_identifier() {
        let program = Program::new(vec![
            Stmt::Expression(Expr::Literal(Literal::String("s".to_string()))),
            Stmt::Expression(Expr::Identifier("missing".to_string())),
        ]);
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Ret {
                value: Some(IRValue::Const(0)),
                ..
            }
        )));
    }

    #[test]
    fn test_build_all_comparison_ops_and_logical_fallback() {
        let exprs = vec![
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                op: BinaryOp::Lt,
                right: Box::new(Expr::Literal(Literal::Number(2))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(1))),
                op: BinaryOp::Le,
                right: Box::new(Expr::Literal(Literal::Number(2))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(2))),
                op: BinaryOp::Gt,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Number(2))),
                op: BinaryOp::Ge,
                right: Box::new(Expr::Literal(Literal::Number(1))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Boolean(true))),
                op: BinaryOp::And,
                right: Box::new(Expr::Literal(Literal::Boolean(false))),
            },
            Expr::Binary {
                left: Box::new(Expr::Literal(Literal::Boolean(true))),
                op: BinaryOp::Or,
                right: Box::new(Expr::Literal(Literal::Boolean(false))),
            },
        ];

        let program = Program::new(exprs.into_iter().map(Stmt::Expression).collect::<Vec<_>>());
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Cmp {
                op: crate::ir::CmpOp::Lt,
                ..
            }
        )));
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Cmp {
                op: crate::ir::CmpOp::Ge,
                ..
            }
        )));
        assert!(main.instructions.iter().any(|i| matches!(
            i,
            IRInstruction::Add {
                lhs: IRValue::Const(0),
                rhs: IRValue::Const(0),
                ..
            }
        )));
    }

    #[test]
    fn test_struct_field_access_and_merge_lowering() {
        let mut parser = Parser::new(
            r#"
            struct Person { age: int; score: int; }
            let p: Person = Person.(age = 18, score = 90);
            println(p.age);
            p += Person.(age = 20);
            println(p.age);
            println(p.score);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let stores = main
            .instructions
            .iter()
            .filter(|instr| matches!(instr, IRInstruction::Store { .. }))
            .count();
        let loads = main
            .instructions
            .iter()
            .filter(|instr| matches!(instr, IRInstruction::Load { .. }))
            .count();
        let print_calls = main
            .instructions
            .iter()
            .filter(|instr| {
                matches!(
                    instr,
                    IRInstruction::Call { function, .. } if function == IRBuilder::PRINTLN_I64
                )
            })
            .count();

        assert_eq!(stores, 3);
        assert_eq!(loads, 3);
        assert_eq!(print_calls, 3);
    }

    #[test]
    fn test_struct_assignment_replaces_all_fields() {
        let mut parser = Parser::new(
            r#"
            struct Person { age: int; score: int; }
            let p: Person = Person.(age = 1, score = 2);
            p = Person.(age = 3, score = 4);
            println(p.score);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let stores = main
            .instructions
            .iter()
            .filter(|instr| matches!(instr, IRInstruction::Store { .. }))
            .count();
        assert_eq!(stores, 4);
    }

    #[test]
    fn test_container_builtins_lower_to_runtime_calls() {
        let mut parser = Parser::new(
            r#"
            let v = vectorNew();
            vectorPush(v, 1);
            println(vectorLen(v));
            hashMapSet(hashMapNew(), 1, 2);
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);
        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();

        let has_vector_new = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "vectorNew"
            )
        });
        let has_vector_push = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "vectorPush"
            )
        });
        let has_hash_map_set = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, .. } if function == "hashMapSet"
            )
        });
        assert!(has_vector_new);
        assert!(has_vector_push);
        assert!(has_hash_map_set);
    }

    #[test]
    fn test_function_argument_lowering_uses_func_value_and_indirect_call() {
        let mut parser = Parser::new(
            r#"
            function inc(x: int): int { return x + 1; }
            function apply(f: function, v: int): int { return f(v); }
            println(apply(inc, 41));
            "#,
        );
        let program = parser.parse().unwrap();
        let module = IRBuilder::new().build(&program);

        let main = module
            .functions
            .iter()
            .find(|func| func.name == "_user_main")
            .unwrap();
        let has_func_value_arg = main.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::Call { function, args, .. }
                    if function == "apply"
                        && matches!(args.first(), Some(IRValue::Func(name)) if name == "inc")
            )
        });
        assert!(has_func_value_arg);

        let apply = module
            .functions
            .iter()
            .find(|func| func.name == "apply")
            .unwrap();
        let has_indirect_call = apply.instructions.iter().any(|instr| {
            matches!(
                instr,
                IRInstruction::CallIndirect {
                    function: IRValue::Var(_),
                    ..
                }
            )
        });
        assert!(has_indirect_call);
    }
}
