use crate::error::{BrainfuckError, Position};
use anyhow::Result;
use std::io::Read;

/// Represents a Brainfuck token with position information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

/// The different types of Brainfuck tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Move pointer right: `>`
    MoveRight,
    /// Move pointer left: `<`
    MoveLeft,
    /// Increment current cell: `+`
    Increment,
    /// Decrement current cell: `-`
    Decrement,
    /// Output current cell: `.`
    Output,
    /// Input to current cell: `,`
    Input,
    /// Start loop: `[`
    LoopStart,
    /// End loop: `]`
    LoopEnd,
}

impl TokenKind {
    /// Check if a character is a valid Brainfuck token
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '>' => Some(Self::MoveRight),
            '<' => Some(Self::MoveLeft),
            '+' => Some(Self::Increment),
            '-' => Some(Self::Decrement),
            '.' => Some(Self::Output),
            ',' => Some(Self::Input),
            '[' => Some(Self::LoopStart),
            ']' => Some(Self::LoopEnd),
            _ => None,
        }
    }

    /// Get the character representation of this token
    pub fn to_char(self) -> char {
        match self {
            Self::MoveRight => '>',
            Self::MoveLeft => '<',
            Self::Increment => '+',
            Self::Decrement => '-',
            Self::Output => '.',
            Self::Input => ',',
            Self::LoopStart => '[',
            Self::LoopEnd => ']',
        }
    }
}

/// A lexer that tokenizes Brainfuck source code
pub struct Lexer<R> {
    reader: R,
    position: Position,
    buffer: Vec<char>,
    buffer_pos: usize,
}

impl<R> Lexer<R>
where
    R: Read,
{
    /// Create a new lexer from a reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            position: Position::default(),
            buffer: Vec::new(),
            buffer_pos: 0,
        }
    }

    /// Read the next token from the source
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        // Read more characters if buffer is empty
        if self.buffer_pos >= self.buffer.len() {
            self.read_more()?;
        }

        // Skip non-Brainfuck characters
        while self.buffer_pos < self.buffer.len() {
            let c = self.buffer[self.buffer_pos];
            self.buffer_pos += 1;
            self.update_position(c);

            if let Some(kind) = TokenKind::from_char(c) {
                return Ok(Some(Token {
                    kind,
                    position: self.position,
                }));
            }
        }

        Ok(None)
    }

    /// Read more characters into the buffer
    fn read_more(&mut self) -> Result<()> {
        let mut buf = [0u8; 1024];
        let bytes_read = self
            .reader
            .read(&mut buf)
            .map_err(|e| BrainfuckError::IoError {
                message: format!("Failed to read source: {}", e),
            })?;

        if bytes_read == 0 {
            return Ok(());
        }

        // Convert bytes to chars, handling UTF-8 properly
        let string = String::from_utf8_lossy(&buf[..bytes_read]);
        self.buffer.extend(string.chars());
        Ok(())
    }

    /// Update position based on character
    fn update_position(&mut self, c: char) {
        if c == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }
    }

    /// Get the current position
    pub fn position(&self) -> Position {
        self.position
    }
}

impl<R> Iterator for Lexer<R>
where
    R: Read,
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_token_kind_from_char() {
        assert_eq!(TokenKind::from_char('>'), Some(TokenKind::MoveRight));
        assert_eq!(TokenKind::from_char('<'), Some(TokenKind::MoveLeft));
        assert_eq!(TokenKind::from_char('+'), Some(TokenKind::Increment));
        assert_eq!(TokenKind::from_char('-'), Some(TokenKind::Decrement));
        assert_eq!(TokenKind::from_char('.'), Some(TokenKind::Output));
        assert_eq!(TokenKind::from_char(','), Some(TokenKind::Input));
        assert_eq!(TokenKind::from_char('['), Some(TokenKind::LoopStart));
        assert_eq!(TokenKind::from_char(']'), Some(TokenKind::LoopEnd));
        assert_eq!(TokenKind::from_char('a'), None);
        assert_eq!(TokenKind::from_char(' '), None);
    }

    #[test]
    fn test_token_kind_to_char() {
        assert_eq!(TokenKind::MoveRight.to_char(), '>');
        assert_eq!(TokenKind::MoveLeft.to_char(), '<');
        assert_eq!(TokenKind::Increment.to_char(), '+');
        assert_eq!(TokenKind::Decrement.to_char(), '-');
        assert_eq!(TokenKind::Output.to_char(), '.');
        assert_eq!(TokenKind::Input.to_char(), ',');
        assert_eq!(TokenKind::LoopStart.to_char(), '[');
        assert_eq!(TokenKind::LoopEnd.to_char(), ']');
    }

    #[test]
    fn test_lexer_basic() {
        let input = ">+<-[],.";
        let cursor = Cursor::new(input.as_bytes());
        let mut lexer = Lexer::new(cursor);

        let expected_tokens = vec![
            TokenKind::MoveRight,
            TokenKind::Increment,
            TokenKind::MoveLeft,
            TokenKind::Decrement,
            TokenKind::LoopStart,
            TokenKind::Input,
            TokenKind::LoopEnd,
            TokenKind::Output,
        ];

        for (i, expected) in expected_tokens.iter().enumerate() {
            let token = lexer.next().unwrap().unwrap();
            assert_eq!(token.kind, *expected, "Token {} should be {:?}", i, expected);
        }

        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_lexer_ignores_comments() {
        let input = "Hello World! >+<-[],.";
        let cursor = Cursor::new(input.as_bytes());
        let mut lexer = Lexer::new(cursor);

        let expected_tokens = vec![
            TokenKind::MoveRight,
            TokenKind::Increment,
            TokenKind::MoveLeft,
            TokenKind::Decrement,
            TokenKind::LoopStart,
            TokenKind::Input,
            TokenKind::LoopEnd,
            TokenKind::Output,
        ];

        for (i, expected) in expected_tokens.iter().enumerate() {
            let token = lexer.next().unwrap().unwrap();
            assert_eq!(token.kind, *expected, "Token {} should be {:?}", i, expected);
        }

        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_lexer_position_tracking() {
        let input = ">+\n<-[],.";
        let cursor = Cursor::new(input.as_bytes());
        let mut lexer = Lexer::new(cursor);

        // First token at position 1:1
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token.kind, TokenKind::MoveRight);
        assert_eq!(token.position, Position::new(1, 1));

        // Second token at position 1:2
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token.kind, TokenKind::Increment);
        assert_eq!(token.position, Position::new(1, 2));

        // Third token at position 2:1 (after newline)
        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token.kind, TokenKind::MoveLeft);
        assert_eq!(token.position, Position::new(2, 1));
    }
} 