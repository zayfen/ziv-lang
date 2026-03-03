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
    
    // Control flow
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
            },
            IRInstruction::Store { dest, ty, value } => {
                write!(f, "  store {} {}, %{}", ty, value, dest)
            },
            IRInstruction::Load { dest, ty, ptr } => {
                write!(f, "  %{} = load {} %{}", dest, ty, ptr)
            },
            IRInstruction::Add { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = add {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Sub { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = sub {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Mul { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = mul {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Div { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = div {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Ret { ty, value } => {
                if let Some(v) = value {
                    write!(f, "  ret {} {}", ty, v)
                } else {
                    write!(f, "  ret void")
                }
            },
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
        let params_str: Vec<String> = self.params.iter()
            .map(|(name, ty)| format!("{} %{}", ty, name))
            .collect();
        
        writeln!(f, "define {} @{}({}) {{", self.ret_ty, self.name, params_str.join(", "))?;
        
        for instr in &self.instructions {
            writeln!(f, "{}", instr)?;
        }
        
        writeln!(f, "}}")
    }
}
