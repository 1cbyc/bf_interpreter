# Brainfuck Interpreter

A fast, efficient, and well-documented Brainfuck interpreter written in Rust. This project provides a complete implementation of the Brainfuck esoteric programming language with robust error handling, optimization, and a clean command-line interface.

## What is Brainfuck?

Brainfuck is an esoteric programming language created by Urban Müller in 1993. It consists of only 8 commands and operates on a tape of memory cells, each initially set to zero. The language is Turing-complete, meaning it can compute anything that any other programming language can compute.

### Brainfuck Commands

| Command | Description |
|---------|-------------|
| `>` | Move pointer to the right |
| `<` | Move pointer to the left |
| `+` | Increment the memory cell under the pointer |
| `-` | Decrement the memory cell under the pointer |
| `.` | Output the character signified by the cell at the pointer |
| `,` | Input a character and store it in the cell at the pointer |
| `[` | Jump past the matching `]` if the cell under the pointer is 0 |
| `]` | Jump back to the matching `[` if the cell under the pointer is nonzero |

## Features

- **Fast Execution**: Optimized for performance with efficient memory management
- **Robust Error Handling**: Comprehensive error reporting with line and column information
- **Command Line Interface**: Clean CLI with multiple options using clap
- **Optimization**: Consecutive operations are optimized (e.g., `++++` becomes a single increment by 4)
- **Memory Safety**: Leverages Rust's memory safety guarantees
- **Cross Platform**: Works on Windows, macOS, and Linux

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/brainfuck-interpreter-rust.git
cd brainfuck-interpreter-rust

# Build in release mode for optimal performance
cargo build --release

# The executable will be in target/release/brainfuck-interpreter
```

## Usage

### Basic Usage

```bash
# Run a Brainfuck file
cargo run --release -- examples/hello_world.bf

# Or use the built executable
./target/release/brainfuck-interpreter examples/hello_world.bf
```

### Command Line Options

```bash
# Show help
cargo run -- --help

# Run with debug output
cargo run -- --debug examples/hello_world.bf

# Set memory size (default: 30000 cells)
cargo run -- --memory-size 50000 examples/hello_world.bf

# Enable optimization (default: enabled)
cargo run -- --no-optimize examples/hello_world.bf
```

## Examples

### Hello World

```brainfuck
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

This program outputs "Hello World!".

### Simple Counter

```brainfuck
++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.
```

### Interactive Input

```brainfuck
,.,.,.,.,.
```

This program reads 5 characters from input and outputs them.

## Project Structure

```
src/
├── main.rs              # Entry point and CLI handling
├── lexer.rs             # Tokenization of Brainfuck source
├── parser.rs            # Parsing tokens into instructions
├── interpreter.rs       # Execution engine
├── optimizer.rs         # Instruction optimization
└── error.rs            # Error types and handling
```

## Architecture

The interpreter follows a classic three-stage architecture:

1. **Lexer**: Converts source code into tokens
2. **Parser**: Converts tokens into executable instructions
3. **Interpreter**: Executes the instructions

### Key Design Decisions

- **Memory Management**: Uses a fixed-size array for memory cells (default 30,000 cells)
- **Optimization**: Consecutive operations are combined for better performance
- **Error Handling**: Comprehensive error reporting with context
- **Type Safety**: Leverages Rust's type system for memory safety

## Performance

The interpreter is optimized for performance:

- **Release Build**: Uses aggressive optimization settings
- **Instruction Optimization**: Consecutive operations are combined
- **Memory Layout**: Efficient memory access patterns
- **Zero-Copy**: Minimizes unnecessary data copying

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
# Clone and setup
git clone https://github.com/yourusername/brainfuck-interpreter-rust.git
cd brainfuck-interpreter-rust

# Run tests
cargo test

# Run with debug output
cargo run -- --debug examples/hello_world.bf

# Check code quality
cargo clippy
cargo fmt
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Urban Müller for creating the Brainfuck language
- The Rust community for excellent tooling and ecosystem
- Contributors and maintainers of the dependencies used in this project