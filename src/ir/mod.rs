//! Intermediate Representation (IR) for LightLang

pub mod instructions;
pub mod builder;

pub use instructions::*;
pub use builder::*;

use std::fmt;

/// IR Module (contains all functions)
#[derive(Debug, Clone)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
}

impl IRModule {
    pub fn new() -> Self {
        IRModule {
            functions: Vec::new(),
        }
    }
    
    pub fn add_function(&mut self, func: IRFunction) {
        self.functions.push(func);
    }
}

impl fmt::Display for IRModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRInstruction, IRType, IRValue};

    #[test]
    fn test_ir_module() {
        let module = IRModule::new();
        assert_eq!(module.functions.len(), 0);
    }

    #[test]
    fn test_add_function_and_display() {
        let mut module = IRModule::new();
        let mut func = IRFunction::new("main".to_string(), IRType::I64);
        func.add_instruction(IRInstruction::Ret {
            ty: IRType::I64,
            value: Some(IRValue::Const(0)),
        });
        module.add_function(func);

        assert_eq!(module.functions.len(), 1);
        let text = format!("{}", module);
        assert!(text.contains("@main"));
        assert!(text.contains("ret i64 0"));
    }
}
