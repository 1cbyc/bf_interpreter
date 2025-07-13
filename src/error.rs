use anyhow::{Context, Result};
use std::fmt;

/// Represents a position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Custom error types for the Brainfuck interpreter
#[derive(Debug, thiserror::Error)]
pub enum BrainfuckError {
    #[error("Unmatched bracket at position {position}")]
    UnmatchedBracket { position: Position },

    #[error("Invalid character '{character}' at position {position}")]
    InvalidCharacter { character: char, position: Position },

    #[error("Memory access out of bounds at address {address}")]
    MemoryOutOfBounds { address: usize },

    #[error("Input/output error: {message}")]
    IoError { message: String },

    #[error("Parse error at position {position}: {message}")]
    ParseError { position: Position, message: String },

    #[error("Runtime error: {message}")]
    RuntimeError { message: String },
}

/// Extension trait for Result to add context with positions
pub trait WithPosition<T> {
    fn with_position(self, position: Position) -> Result<T>;
    fn with_context_str(self, context: &str) -> Result<T>;
}

impl<T> WithPosition<T> for Result<T, BrainfuckError> {
    fn with_position(self, position: Position) -> Result<T> {
        self.with_context(|| format!("at position {}", position))
    }

    fn with_context_str(self, context: &str) -> Result<T> {
        let context_string = context.to_string();
        self.with_context(move || context_string.clone())
    }
}

/// Helper function to create a parse error
pub fn parse_error(position: Position, message: &str) -> BrainfuckError {
    BrainfuckError::ParseError {
        position,
        message: message.to_string(),
    }
}

/// Helper function to create a runtime error
pub fn runtime_error(message: &str) -> BrainfuckError {
    BrainfuckError::RuntimeError {
        message: message.to_string(),
    }
}

/// Helper function to create an IO error
pub fn io_error(message: &str) -> BrainfuckError {
    BrainfuckError::IoError {
        message: message.to_string(),
    }
} 