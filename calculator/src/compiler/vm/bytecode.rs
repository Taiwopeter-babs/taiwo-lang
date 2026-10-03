use crate::{
    Compile, Node,
    ast::Operator,
    compiler::vm::opcode::{OpCode, make_op},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<Node>,
}

impl Bytecode {
    fn new() -> Self {
        Bytecode {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
}

/// Bytecode interpreter
pub struct Interpreter {
    bytecode: Bytecode,
}

impl Compile for Interpreter {
    type Output = Bytecode;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut interpreter = Interpreter {
            bytecode: Bytecode::new(),
        };

        for node in ast {
            println!("compiling node {:?}", node);

            interpreter.interprete_node(node);
            // Add pop to pop one element from the stack after each expression statement to clean up
            interpreter.add_instruction(OpCode::OpPop);
        }

        interpreter.bytecode
    }
}

impl Interpreter {
    /// Adds a new constant to the [Bytecode::constants] pool and returns the index.
    /// The length of the array is casted to u16 because that's the size of the constants pool index
    fn add_constant(&mut self, node: Node) -> u16 {
        self.bytecode.constants.push(node);

        (self.bytecode.constants.len() - 1) as u16
    }

    fn add_instruction(&mut self, op_code: OpCode) -> u16 {
        let pos_of_new_instr = self.bytecode.instructions.len() as u16;

        self.bytecode.instructions.extend(make_op(op_code));
        println!(
            "Added instructions {:?} from opcode {:?}",
            self.bytecode.instructions,
            op_code.clone()
        );

        pos_of_new_instr
    }

    fn interprete_node(&mut self, node: Node) {
        match node {
            Node::Int(num) => {
                let const_idx = self.add_constant(Node::Int(num));
                self.add_instruction(OpCode::OpConstant(const_idx));
            }
            Node::UnaryExpr { op, child } => {
                self.interprete_node(*child);

                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpPlus),
                    Operator::Minus => self.add_instruction(OpCode::OpMinus),
                };
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                self.interprete_node(*lhs);
                self.interprete_node(*rhs);

                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpAdd),
                    Operator::Minus => self.add_instruction(OpCode::OpSub),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        infix_template("+", OpCode::OpAdd);
        infix_template("-", OpCode::OpSub);
    }

    fn infix_template(infix_str: &str, op_code: OpCode) {
        let input = format!("1 {} 2", infix_str);
        let bytecode = Interpreter::from_source(&input);

        let expected_instructions = vec![
            OpCode::OpConstant(0),
            OpCode::OpConstant(1),
            op_code,
            OpCode::OpPop,
        ]
        .into_iter()
        .flat_map(make_op)
        .collect();

        let expected_bytecode = Bytecode {
            instructions: expected_instructions,
            constants: vec![Node::Int(1), Node::Int(2)],
        };

        assert_eq!(expected_bytecode, bytecode);
    }
}
