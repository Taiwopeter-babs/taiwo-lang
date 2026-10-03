use std::mem::MaybeUninit;

use crate::{
    Node,
    compiler::vm::{bytecode::Bytecode, opcode::convert_two_u8s_to_usize},
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
