use std::mem::MaybeUninit;

use anyhow::anyhow;

use crate::{
    Compile, Node, Result,
    compiler::vm::{
        bytecode::{Bytecode, Interpreter as BytecodeInterpreter},
        opcode::convert_two_u8s_to_usize,
    },
};

const STACK_SIZE: usize = 512;

pub struct VM {
    bytecode: Bytecode,
    // stack: [Node; STACK_SIZE],
    // stack: Vec<Node>,
    stack: [MaybeUninit<Node>; STACK_SIZE],

    /// Points to the next free space
    stack_ptr: usize,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            bytecode,
            // stack: unsafe { std::mem::zeroed() }, // This will cause undefined behaviour. Node is an enum, so it's a non-zero discriminant
            // stack: Vec::with_capacity(STACK_SIZE), // Alternate init
            stack: unsafe { MaybeUninit::uninit().assume_init() },
            stack_ptr: 0,
        }
    }

    pub fn run(&mut self) {
        // Instruction pointer
        let mut ip = 0;

        while ip < self.bytecode.instructions.len() {
            let instr_addr = ip;
            ip += 1;

            match self.bytecode.instructions[instr_addr] {
                // OpConst
                0x01 => {
                    let first_8_bits = self.bytecode.instructions[ip];
                    let second_8_bits = self.bytecode.instructions[ip + 1];

                    let const_addr = convert_two_u8s_to_usize(first_8_bits, second_8_bits);
                    ip += 2;

                    // Read the actual constant value from the constants and push on the stack
                    self.push(self.bytecode.constants[const_addr].clone())
                }
                // OpPop
                0x02 => {
                    self.pop();
                }
                // OpAdd
                0x03 => match (self.pop(), self.pop()) {
                    (Node::Int(rhs), Node::Int(lhs)) => self.push(Node::Int(lhs + rhs)),
                    _ => panic!("Unknown types for Instruction OpAdd"),
                },
                // OpSub
                0x04 => match (self.pop(), self.pop()) {
                    (Node::Int(rhs), Node::Int(lhs)) => self.push(Node::Int(lhs - rhs)),
                    _ => panic!("Unknown types for Instruction OpSub"),
                },
                // OpPlus
                0x0A => match self.pop() {
                    Node::Int(num) => self.push(Node::Int(num)),
                    _ => panic!("Unknwon argument type to OpPlus"),
                },
                // OpMinus
                0x0B => match self.pop() {
                    Node::Int(num) => self.push(Node::Int(-num)),
                    _ => panic!("Unknwon argument type to OpMinus"),
                },
                _ => panic!("Unknown Instruction"),
            }
        }
    }

    pub fn push(&mut self, node: Node) {
        self.stack[self.stack_ptr].write(node);

        // Here, we don't ignore the potential stack overflow
        if self.stack_ptr + 1 > STACK_SIZE {
            panic!("Stack Overflow")
        }

        // Point to the next free space
        self.stack_ptr += 1;
    }

    pub fn pop(&mut self) -> Node {
        let current_stack_ptr = self.stack_ptr;

        if current_stack_ptr == 0 {
            println!("Yes it is {}", current_stack_ptr);
        }
        let node = unsafe { self.stack[self.stack_ptr - 1].assume_init_read() };

        self.stack_ptr -= 1;

        node
    }

    pub fn pop_last(&self) -> &Node {
        // The stack pointer points to the next "free" space
        // which also holds the most recently popped element
        let node = unsafe { self.stack[self.stack_ptr].assume_init_ref() };

        node
    }
}

impl Compile for VM {
    type Output = Result<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let bytecode = BytecodeInterpreter::from_ast(ast);
        let mut vm = VM::new(bytecode);

        vm.run();

        let result_int = vm.pop_last();

        match result_int {
            Node::Int(n) => Ok(*n),
            _ => Err(anyhow!("Expected Integer Result")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Compile, Node, compiler::vm::bytecode::Interpreter as BytecodeInterpreter};

    fn assert_pop_last(source: &str, node: Node) {
        let byte_code = BytecodeInterpreter::from_source(source);
        println!("byte code: {:?}", byte_code);

        let mut vm = VM::new(byte_code);
        vm.run();

        assert_eq!(&node, vm.pop_last());
    }

    #[test]
    fn unary() {
        assert_pop_last("+3", Node::Int(3));
        assert_pop_last("-6", Node::Int(-6));
    }

    #[test]
    fn binary() {
        assert_pop_last("10 + 33;", Node::Int(43));
        assert_pop_last("1 - 98;", Node::Int(-97));
    }
}
