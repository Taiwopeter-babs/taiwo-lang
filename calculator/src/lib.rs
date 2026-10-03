pub use crate::ast::{Node, Operator};

pub mod ast;
pub mod compiler;
pub mod parser;

pub use crate::compiler::interpreter::Interpreter;
#[cfg(feature = "jit")]
pub use crate::compiler::jit::Jit;
pub use crate::compiler::vm::{self, vm::VM};

pub type Result<T> = anyhow::Result<T>;

pub trait Compile {
    type Output;

    fn from_ast(ast: Vec<Node>) -> Self::Output;

    fn from_source(source: &str) -> Self::Output {
        println!("Compiling the source: {}", source);

        let ast: Vec<Node> = parser::parse(source).unwrap();

        for node in ast.iter() {
            println!("Node => {:?}", node);
        }

        Self::from_ast(ast)
    }
}
