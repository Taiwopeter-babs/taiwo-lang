use crate::{Compile, Node, Result, ast::Operator};
use inkwell::{
    OptimizationLevel, builder::Builder, context::Context, execution_engine::JitFunction,
    types::IntType, values::IntValue,
};

const JIT_FUNC_NAME: &str = "jit_add";

type JitFunc = unsafe extern "C" fn() -> i32;

pub struct Jit;

impl Compile for Jit {
    type Output = Result<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let context = Context::create();
        let module = context.create_module("calculator");

        let builder = context.create_builder();

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let fn_val = module.add_function(JIT_FUNC_NAME, fn_type, None);

        let entry_basic_block = context.append_basic_block(fn_val, "entry");

        builder.position_at_end(entry_basic_block);

        for node in ast {
            let recursive_builder = RecursiveBuilder::new(i32_type, &builder);
            let value = recursive_builder.build(&node);
            let _ = builder.build_return(Some(&value));
        }

        unsafe {
            let jit_function: JitFunction<JitFunc> =
                execution_engine.get_function(JIT_FUNC_NAME).unwrap();

            Ok(jit_function.call())
        }
    }
}

struct RecursiveBuilder<'a> {
    i32_type: IntType<'a>,
    builder: &'a Builder<'a>,
}

impl<'a> RecursiveBuilder<'a> {
    pub fn new(i32_type: IntType<'a>, builder: &'a Builder<'a>) -> Self {
        Self { i32_type, builder }
    }

    fn build(&self, node: &Node) -> IntValue<'a> {
        match node {
            Node::Int(n) => self.i32_type.const_int(*n as u64, true),
            Node::UnaryExpr { op, child } => {
                let child = self.build(child);

                match op {
                    Operator::Minus => child.const_neg(),
                    Operator::Plus => child,
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left_child = self.build(lhs);
                let right_child = self.build(rhs);

                match op {
                    Operator::Minus => self
                        .builder
                        .build_int_sub(left_child, right_child, "minus")
                        .unwrap(),
                    Operator::Plus => self
                        .builder
                        .build_int_add(left_child, right_child, "plus")
                        .unwrap(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_basics() {
        assert_eq!(Jit::from_source("1").unwrap() as i32, 1);

        assert_eq!(Jit::from_source("1 + 2").unwrap() as i32, 3);

        assert_eq!(Jit::from_source("2 + (2 - 1)").unwrap() as i32, 3);

        assert_eq!(Jit::from_source("(2 + 3) - 1").unwrap() as i32, 4);

        assert_eq!(
            Jit::from_source("1 + ((2 + 3) - (2 + 3))").unwrap() as i32,
            1
        );

        assert_eq!(
            Jit::from_source("-57 + (-7 + 10) - 80 + 100").unwrap() as i32,
            -34
        );

        assert_eq!(Jit::from_source("1 + 2 - 3").unwrap() as i32, 0);

        assert_eq!(Jit::from_source("-((1 + 2) + (3 - 4))").unwrap() as i32, -2);
    }

    #[test]
    #[should_panic]
    fn test_panic_for_unknown_operator() {
        assert_eq!(Jit::from_source("-57 + (-7 + 10) - 80 * 100").unwrap(), -39);
    }
}
