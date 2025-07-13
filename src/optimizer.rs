use crate::error::{BrainfuckError, Position};
use crate::lexer::{Token, TokenKind};
use anyhow::Result;

/// Optimized instruction that can be executed by the interpreter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Move pointer right by N positions
    MoveRight(usize),
    /// Move pointer left by N positions
    MoveLeft(usize),
    /// Increment current cell by N
    Increment(u8),
    /// Decrement current cell by N
    Decrement(u8),
    /// Output current cell N times
    Output(usize),
    /// Input to current cell N times
    Input(usize),
    /// Jump forward to instruction at index if current cell is 0
    JumpForward(usize),
    /// Jump backward to instruction at index if current cell is not 0
    JumpBackward(usize),
}

impl Instruction {
    /// Get the number of operations this instruction represents
    pub fn operation_count(&self) -> usize {
        match self {
            Self::MoveRight(n) | Self::MoveLeft(n) | Self::Output(n) | Self::Input(n) => *n,
            Self::Increment(n) | Self::Decrement(n) => *n as usize,
            Self::JumpForward(_) | Self::JumpBackward(_) => 1,
        }
    }
}

/// An optimizer that combines consecutive operations for better performance
pub struct Optimizer {
    instructions: Vec<Instruction>,
    jump_stack: Vec<usize>,
}

impl Optimizer {
    /// Create a new optimizer
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            jump_stack: Vec::new(),
        }
    }

    /// Optimize a stream of tokens into instructions
    pub fn optimize(&mut self, tokens: impl Iterator<Item = Result<Token>>) -> Result<Vec<Instruction>> {
        self.instructions.clear();
        self.jump_stack.clear();

        for token_result in tokens {
            let token = token_result?;
            self.process_token(token)?;
        }

        // Check for unmatched brackets
        if !self.jump_stack.is_empty() {
            let position = self.instructions
                .iter()
                .filter_map(|inst| {
                    if let Instruction::JumpForward(_) = inst {
                        Some(Position::new(1, 1)) // We don't track positions in optimized instructions
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(Position::default());

            return Err(BrainfuckError::UnmatchedBracket { position }.into());
        }

        Ok(self.instructions.clone())
    }

    /// Process a single token and add optimized instructions
    fn process_token(&mut self, token: Token) -> Result<()> {
        match token.kind {
            TokenKind::MoveRight => self.optimize_move(1, true),
            TokenKind::MoveLeft => self.optimize_move(1, false),
            TokenKind::Increment => self.optimize_arithmetic(1, true),
            TokenKind::Decrement => self.optimize_arithmetic(1, false),
            TokenKind::Output => self.optimize_io(1, true),
            TokenKind::Input => self.optimize_io(1, false),
            TokenKind::LoopStart => self.handle_loop_start(),
            TokenKind::LoopEnd => self.handle_loop_end(token.position)?,
        }
        Ok(())
    }

    /// Optimize consecutive move operations
    fn optimize_move(&mut self, count: usize, right: bool) {
        if let Some(last_inst) = self.instructions.last_mut() {
            match (last_inst, right) {
                (Instruction::MoveRight(n), true) => *n += count,
                (Instruction::MoveLeft(n), false) => *n += count,
                _ => {
                    let inst = if right {
                        Instruction::MoveRight(count)
                    } else {
                        Instruction::MoveLeft(count)
                    };
                    self.instructions.push(inst);
                }
            }
        } else {
            let inst = if right {
                Instruction::MoveRight(count)
            } else {
                Instruction::MoveLeft(count)
            };
            self.instructions.push(inst);
        }
    }

    /// Optimize consecutive arithmetic operations
    fn optimize_arithmetic(&mut self, count: u8, increment: bool) {
        if let Some(last_inst) = self.instructions.last_mut() {
            match (last_inst, increment) {
                (Instruction::Increment(n), true) => *n = n.wrapping_add(count),
                (Instruction::Decrement(n), false) => *n = n.wrapping_add(count),
                _ => {
                    let inst = if increment {
                        Instruction::Increment(count)
                    } else {
                        Instruction::Decrement(count)
                    };
                    self.instructions.push(inst);
                }
            }
        } else {
            let inst = if increment {
                Instruction::Increment(count)
            } else {
                Instruction::Decrement(count)
            };
            self.instructions.push(inst);
        }
    }

    /// Optimize consecutive I/O operations
    fn optimize_io(&mut self, count: usize, output: bool) {
        if let Some(last_inst) = self.instructions.last_mut() {
            match (last_inst, output) {
                (Instruction::Output(n), true) => *n += count,
                (Instruction::Input(n), false) => *n += count,
                _ => {
                    let inst = if output {
                        Instruction::Output(count)
                    } else {
                        Instruction::Input(count)
                    };
                    self.instructions.push(inst);
                }
            }
        } else {
            let inst = if output {
                Instruction::Output(count)
            } else {
                Instruction::Input(count)
            };
            self.instructions.push(inst);
        }
    }

    /// Handle the start of a loop
    fn handle_loop_start(&mut self) {
        self.jump_stack.push(self.instructions.len());
        self.instructions.push(Instruction::JumpForward(0)); // Placeholder
    }

    /// Handle the end of a loop
    fn handle_loop_end(&mut self, position: Position) -> Result<()> {
        if let Some(start_index) = self.jump_stack.pop() {
            // Update the forward jump to point to the current position
            if let Some(Instruction::JumpForward(_)) = self.instructions.get_mut(start_index) {
                self.instructions[start_index] = Instruction::JumpForward(self.instructions.len());
            }
            
            // Add the backward jump
            self.instructions.push(Instruction::JumpBackward(start_index));
        } else {
            return Err(BrainfuckError::UnmatchedBracket { position }.into());
        }
        Ok(())
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use std::io::Cursor;

    #[test]
    fn test_optimize_consecutive_moves() {
        let input = ">>>>";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();

        let instructions = optimizer.optimize(lexer).unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::MoveRight(4));
    }

    #[test]
    fn test_optimize_consecutive_increments() {
        let input = "++++";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();

        let instructions = optimizer.optimize(lexer).unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::Increment(4));
    }

    #[test]
    fn test_optimize_mixed_operations() {
        let input = ">+<->+";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();

        let instructions = optimizer.optimize(lexer).unwrap();
        assert_eq!(instructions.len(), 6);
        assert_eq!(instructions[0], Instruction::MoveRight(1));
        assert_eq!(instructions[1], Instruction::Increment(1));
        assert_eq!(instructions[2], Instruction::MoveLeft(1));
        assert_eq!(instructions[3], Instruction::Decrement(1));
        assert_eq!(instructions[4], Instruction::MoveRight(1));
        assert_eq!(instructions[5], Instruction::Increment(1));
    }

    #[test]
    fn test_optimize_simple_loop() {
        let input = "[+]";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();

        let instructions = optimizer.optimize(lexer).unwrap();
        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0], Instruction::JumpForward(1));
        assert_eq!(instructions[1], Instruction::JumpBackward(0));
    }

    #[test]
    fn test_optimize_unmatched_bracket() {
        let input = "[";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();

        let result = optimizer.optimize(lexer);
        assert!(result.is_err());
    }

    #[test]
    fn test_optimize_unmatched_bracket_end() {
        let input = "]";
        let cursor = Cursor::new(input.as_bytes());
        let lexer = Lexer::new(cursor);
        let mut optimizer = Optimizer::new();

        let result = optimizer.optimize(lexer);
        assert!(result.is_err());
    }
} 