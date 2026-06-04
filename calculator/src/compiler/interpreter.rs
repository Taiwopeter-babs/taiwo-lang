use crate::{Compile, Node, Result, ast::Operator};

pub struct Interpreter;

impl Compile for Interpreter {
    type Output = Result<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut return_val = 0i32;

        let evaluator = Eval::new();

        for node in ast {
            return_val += evaluator.eval(&node)
        }

        Ok(return_val)
    }
}

struct Eval;

impl Eval {
    pub fn new() -> Self {
        Self
    }

    pub fn eval(&self, node: &Node) -> i32 {
        match node {
            Node::Int(n) => *n,

            Node::UnaryExpr { op, child } => {
                let child = self.eval(child);

                match op {
                    Operator::Minus => -child,
                    Operator::Plus => child,
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left_child = self.eval(lhs);
                let right_child = self.eval(rhs);

                match op {
                    Operator::Minus => left_child - right_child,
                    Operator::Plus => left_child + right_child,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(Interpreter::from_source("1").unwrap() as i32, 1);

        assert_eq!(Interpreter::from_source("1 + 2").unwrap() as i32, 3);

        assert_eq!(Interpreter::from_source("2 + (2 - 1)").unwrap() as i32, 3);

        assert_eq!(Interpreter::from_source("(2 + 3) - 1").unwrap() as i32, 4);

        assert_eq!(
            Interpreter::from_source("1 + ((2 + 3) - (2 + 3))").unwrap() as i32,
            1
        );

        assert_eq!(
            Interpreter::from_source("-57 + (-7 + 10) - 80 + 100").unwrap() as i32,
            -34
        );
    }

    #[test]
    #[should_panic]
    fn test_panic_for_unknown_operator() {
        assert_eq!(
            Interpreter::from_source("-57 + (-7 + 10) - 80 * 100").unwrap(),
            -39
        );
    }
}
