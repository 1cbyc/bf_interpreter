use crate::error::BrainfuckError;
use crate::optimizer::Instruction;
use anyhow::Result;
use std::io::{self, Read, Write};

/// Configuration for the Brainfuck interpreter
#[derive(Debug, Clone)]
pub struct InterpreterConfig {
    /// Size of the memory tape (default: 30000)
    pub memory_size: usize,
    /// Whether to enable debug output
    pub debug: bool,
    /// Whether to enable optimizations
    pub optimize: bool,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            memory_size: 30000,
            debug: false,
            optimize: true,
        }
    }
}

/// The Brainfuck interpreter that executes optimized instructions
pub struct Interpreter {
    /// The program instructions
    instructions: Vec<Instruction>,
    /// Memory tape (array of u8 cells)
    memory: Vec<u8>,
    /// Current memory pointer position
    pointer: usize,
    /// Current instruction pointer
    instruction_pointer: usize,
    /// Configuration
    config: InterpreterConfig,
}

impl Interpreter {
    /// Create a new interpreter with the given instructions and configuration
    pub fn new(instructions: Vec<Instruction>, config: InterpreterConfig) -> Self {
        Self {
            memory: vec![0; config.memory_size],
            pointer: 0,
            instruction_pointer: 0,
            instructions,
            config,
        }
    }

    /// Run the interpreter until completion
    pub fn run(&mut self) -> Result<()> {
        while self.instruction_pointer < self.instructions.len() {
            if self.config.debug {
                eprintln!(
                    "IP: {}, PTR: {}, CELL: {}, INST: {:?}",
                    self.instruction_pointer,
                    self.pointer,
                    self.memory[self.pointer],
                    self.instructions[self.instruction_pointer]
                );
            }

            self.execute_instruction()?;
        }
        Ok(())
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self) -> Result<()> {
        let instruction = &self.instructions[self.instruction_pointer];

        match instruction {
            Instruction::MoveRight(count) => {
                self.pointer = self.pointer.wrapping_add(*count);
                if self.pointer >= self.memory.len() {
                    return Err(BrainfuckError::MemoryOutOfBounds {
                        address: self.pointer,
                    }
                    .into());
                }
                self.instruction_pointer += 1;
            }

            Instruction::MoveLeft(count) => {
                if self.pointer < *count {
                    return Err(BrainfuckError::MemoryOutOfBounds {
                        address: self.pointer.wrapping_sub(*count),
                    }
                    .into());
                }
                self.pointer -= count;
                self.instruction_pointer += 1;
            }

            Instruction::Increment(count) => {
                self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(*count);
                self.instruction_pointer += 1;
            }

            Instruction::Decrement(count) => {
                self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(*count);
                self.instruction_pointer += 1;
            }

            Instruction::Output(count) => {
                let mut stdout = io::stdout();
                for _ in 0..*count {
                    stdout
                        .write_all(&[self.memory[self.pointer]])
                        .map_err(|e| {
                            BrainfuckError::IoError {
                                message: format!("Failed to write to stdout: {}", e),
                            }
                        })?;
                }
                stdout.flush().map_err(|e| {
                    BrainfuckError::IoError {
                        message: format!("Failed to flush stdout: {}", e),
                    }
                })?;
                self.instruction_pointer += 1;
            }

            Instruction::Input(count) => {
                let mut stdin = io::stdin();
                for _ in 0..*count {
                    let mut buf = [0u8; 1];
                    stdin
                        .read_exact(&mut buf)
                        .map_err(|e| {
                            BrainfuckError::IoError {
                                message: format!("Failed to read from stdin: {}", e),
                            }
                        })?;
                    self.memory[self.pointer] = buf[0];
                }
                self.instruction_pointer += 1;
            }

            Instruction::JumpForward(target) => {
                if self.memory[self.pointer] == 0 {
                    self.instruction_pointer = *target;
                } else {
                    self.instruction_pointer += 1;
                }
            }

            Instruction::JumpBackward(target) => {
                if self.memory[self.pointer] != 0 {
                    self.instruction_pointer = *target;
                } else {
                    self.instruction_pointer += 1;
                }
            }
        }

        Ok(())
    }

    /// Get the current memory state (for debugging)
    pub fn memory_state(&self) -> &[u8] {
        &self.memory
    }

    /// Get the current pointer position
    pub fn pointer(&self) -> usize {
        self.pointer
    }

    /// Get the current instruction pointer
    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    /// Get the number of instructions
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::optimizer::Optimizer;
    use std::io::Cursor;

    fn run_program(input: &str) -> Result<String> {
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();
        let instructions = optimizer.optimize(lexer)?;
        
        let config = InterpreterConfig::default();
        let mut interpreter = Interpreter::new(instructions, config);
        interpreter.run()?;
        
        // Capture output
        let mut output = Vec::new();
        io::stdout().write_all(&output)?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }

    #[test]
    fn test_simple_increment() {
        let input = "+++";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();
        let instructions = optimizer.optimize(lexer).unwrap();
        
        let config = InterpreterConfig::default();
        let mut interpreter = Interpreter::new(instructions, config);
        interpreter.run().unwrap();
        
        assert_eq!(interpreter.memory_state()[0], 3);
    }

    #[test]
    fn test_pointer_movement() {
        let input = ">+++<+";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();
        let instructions = optimizer.optimize(lexer).unwrap();
        
        let config = InterpreterConfig::default();
        let mut interpreter = Interpreter::new(instructions, config);
        interpreter.run().unwrap();
        
        assert_eq!(interpreter.memory_state()[0], 1);
        assert_eq!(interpreter.memory_state()[1], 3);
    }

    #[test]
    fn test_simple_loop() {
        let input = "+++[>+<-]";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();
        let instructions = optimizer.optimize(lexer).unwrap();
        
        let config = InterpreterConfig::default();
        let mut interpreter = Interpreter::new(instructions, config);
        interpreter.run().unwrap();
        
        assert_eq!(interpreter.memory_state()[0], 0);
        assert_eq!(interpreter.memory_state()[1], 3);
    }

    #[test]
    fn test_memory_bounds() {
        let input = "<";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();
        let instructions = optimizer.optimize(lexer).unwrap();
        
        let config = InterpreterConfig::default();
        let mut interpreter = Interpreter::new(instructions, config);
        let result = interpreter.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_mode() {
        let input = "+++";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();
        let instructions = optimizer.optimize(lexer).unwrap();
        
        let mut config = InterpreterConfig::default();
        config.debug = true;
        let mut interpreter = Interpreter::new(instructions, config);
        interpreter.run().unwrap();
        
        assert_eq!(interpreter.memory_state()[0], 3);
    }
} 