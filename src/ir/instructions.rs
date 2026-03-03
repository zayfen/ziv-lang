//! IR Instructions for LightLang

use std::fmt;

/// IR Value
#[derive(Debug, Clone, PartialEq)]
pub enum IRValue {
    Const(i64),
    Var(String),
}

impl fmt::Display for IRValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRValue::Const(n) => write!(f, "{}", n),
            IRValue::Var(name) => write!(f, "%{}", name),
        }
    }
}

/// IR Type
#[derive(Debug, Clone, PartialEq)]
pub enum IRType {
    I64,
    Void,
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRType::I64 => write!(f, "i64"),
            IRType::Void => write!(f, "void"),
        }
    }
}

/// IR Instructions
#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CmpOp::Eq => write!(f, "eq"),
            CmpOp::Ne => write!(f, "ne"),
            CmpOp::Lt => write!(f, "lt"),
            CmpOp::Le => write!(f, "le"),
            CmpOp::Gt => write!(f, "gt"),
            CmpOp::Ge => write!(f, "ge"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    // Memory
    Alloc {
        dest: String,
        ty: IRType,
    },
    Store {
        dest: String,
        ty: IRType,
        value: IRValue,
    },
    Load {
        dest: String,
        ty: IRType,
        ptr: String,
    },

    // Arithmetic
    Add {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    Sub {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    Mul {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    Div {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },

    Cmp {
        dest: String,
        op: CmpOp,
        lhs: IRValue,
        rhs: IRValue,
    },

    Call {
        result: Option<String>,
        function: String,
        args: Vec<IRValue>,
    },

    // Control flow
    Label(String),
    Jump(String),
    CondBranch {
        condition: IRValue,
        true_label: String,
        false_label: String,
    },

    Ret {
        ty: IRType,
        value: Option<IRValue>,
    },
}

impl fmt::Display for IRInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRInstruction::Alloc { dest, ty } => {
                write!(f, "  %{} = alloc {}", dest, ty)
            }
            IRInstruction::Store { dest, ty, value } => {
                write!(f, "  store {} {}, %{}", ty, value, dest)
            }
            IRInstruction::Load { dest, ty, ptr } => {
                write!(f, "  %{} = load {} %{}", dest, ty, ptr)
            }
            IRInstruction::Add { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = add {} {}, {}", dest, ty, lhs, rhs)
            }
            IRInstruction::Sub { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = sub {} {}, {}", dest, ty, lhs, rhs)
            }
            IRInstruction::Mul { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = mul {} {}, {}", dest, ty, lhs, rhs)
            }
            IRInstruction::Div { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = div {} {}, {}", dest, ty, lhs, rhs)
            }
            IRInstruction::Cmp { dest, op, lhs, rhs } => {
                let op_str = match op {
                    CmpOp::Eq => "eq",
                    CmpOp::Ne => "ne",
                    CmpOp::Lt => "lt",
                    CmpOp::Le => "le",
                    CmpOp::Gt => "gt",
                    CmpOp::Ge => "ge",
                };
                write!(f, "  %{} = cmp {} {}, {}", dest, op, lhs, rhs)
            }
            IRInstruction::Call {
                result,
                function,
                args,
            } => {
                let args_str: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                if let Some(res) = result {
                    write!(
                        f,
                        "  %{} = call @{}({})",
                        res,
                        function,
                        args_str.join(", ")
                    )
                } else {
                    write!(f, "  call @{}({})", function, args_str.join(", "))
                }
            }
            IRInstruction::Label(name) => write!(f, "{}:", name),
            IRInstruction::Jump(label) => write!(f, "  br label %{}", label),
            IRInstruction::CondBranch {
                condition,
                true_label,
                false_label,
            } => write!(
                f,
                "  br i1 {}, label %{}, label %{}",
                condition, true_label, false_label
            ),
            IRInstruction::Ret { ty, value } => {
                if let Some(v) = value {
                    write!(f, "  ret {} {}", ty, v)
                } else {
                    write!(f, "  ret void")
                }
            }
        }
    }
}

/// IR Function
#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<(String, IRType)>,
    pub ret_ty: IRType,
    pub instructions: Vec<IRInstruction>,
}

impl IRFunction {
    pub fn new(name: String, ret_ty: IRType) -> Self {
        IRFunction {
            name,
            params: Vec::new(),
            ret_ty,
            instructions: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, instr: IRInstruction) {
        self.instructions.push(instr);
    }
}

impl fmt::Display for IRFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params_str: Vec<String> = self
            .params
            .iter()
            .map(|(name, ty)| format!("{} %{}", ty, name))
            .collect();

        writeln!(
            f,
            "define {} @{}({}) {{",
            self.ret_ty,
            self.name,
            params_str.join(", ")
        )?;

        for instr in &self.instructions {
            writeln!(f, "{}", instr)?;
        }

        writeln!(f, "}}")
    }
}
