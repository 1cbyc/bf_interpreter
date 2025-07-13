use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod error;
mod interpreter;
mod lexer;
mod optimizer;


use interpreter::{Interpreter, InterpreterConfig};
use lexer::Lexer;
use optimizer::Optimizer;

/// A fast and efficient Brainfuck interpreter written in Rust
#[derive(Parser)]
#[command(
    name = "brainfuck-interpreter",
    about = "A fast and efficient Brainfuck interpreter",
    version,
    long_about = "A Brainfuck interpreter that supports all standard Brainfuck operations with optimization and robust error handling."
)]
struct Cli {
    /// The Brainfuck source file to execute
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Enable debug output showing instruction execution
    #[arg(short, long)]
    debug: bool,

    /// Set the memory size (default: 30000)
    #[arg(short, long, default_value = "30000")]
    memory_size: usize,

    /// Disable optimizations
    #[arg(long)]
    no_optimize: bool,

    /// Show program statistics after execution
    #[arg(short, long)]
    stats: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate memory size
    if cli.memory_size == 0 {
        return Err(anyhow::anyhow!("Memory size must be greater than 0"));
    }

    // Read and execute the Brainfuck program
    run_brainfuck_program(&cli)?;

    Ok(())
}

fn run_brainfuck_program(cli: &Cli) -> Result<()> {
    // Open the source file
    let file = File::open(&cli.file).with_context(|| {
        format!("Failed to open file '{}'", cli.file.display())
    })?;

    let reader = BufReader::new(file);

    // Create lexer
    let lexer = Lexer::new(reader);

    // Create optimizer and parse instructions
    let mut optimizer = Optimizer::new();
    let instructions = optimizer.optimize(lexer)
        .with_context(|| format!("Failed to parse Brainfuck program from '{}'", cli.file.display()))?;

    // Create interpreter configuration
    let mut config = InterpreterConfig::default();
    config.memory_size = cli.memory_size;
    config.debug = cli.debug;
    config.optimize = !cli.no_optimize;

    // Create and run interpreter
    let mut interpreter = Interpreter::new(instructions.clone(), config);

    if cli.debug {
        eprintln!("Starting execution of '{}'", cli.file.display());
        eprintln!("Memory size: {}", cli.memory_size);
        eprintln!("Instructions: {}", instructions.len());
        eprintln!("Optimizations: {}", !cli.no_optimize);
        eprintln!("---");
    }

    // Execute the program
    let result = interpreter.run();

    // Handle execution result
    match result {
        Ok(()) => {
            if cli.debug {
                eprintln!("---");
                eprintln!("Execution completed successfully");
            }

            if cli.stats {
                print_statistics(&interpreter, &instructions);
            }
        }
        Err(e) => {
            eprintln!("Error during execution: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_statistics(interpreter: &Interpreter, instructions: &[optimizer::Instruction]) {
    eprintln!("\n=== Program Statistics ===");
    eprintln!("Total instructions: {}", instructions.len());
    eprintln!("Memory cells used: {}", interpreter.memory_state().len());
    eprintln!("Final pointer position: {}", interpreter.pointer());
    eprintln!("Final instruction pointer: {}", interpreter.instruction_pointer());

    // Count instruction types
    let mut counts = std::collections::HashMap::new();
    for instruction in instructions {
        let count = counts.entry(std::mem::discriminant(instruction)).or_insert(0);
        *count += 1;
    }

    eprintln!("\nInstruction breakdown:");
    for (discriminant, count) in counts {
        let name = match discriminant {
            _ if std::mem::discriminant(&optimizer::Instruction::MoveRight(0)) == discriminant => "MoveRight",
            _ if std::mem::discriminant(&optimizer::Instruction::MoveLeft(0)) == discriminant => "MoveLeft",
            _ if std::mem::discriminant(&optimizer::Instruction::Increment(0)) == discriminant => "Increment",
            _ if std::mem::discriminant(&optimizer::Instruction::Decrement(0)) == discriminant => "Decrement",
            _ if std::mem::discriminant(&optimizer::Instruction::Output(0)) == discriminant => "Output",
            _ if std::mem::discriminant(&optimizer::Instruction::Input(0)) == discriminant => "Input",
            _ if std::mem::discriminant(&optimizer::Instruction::JumpForward(0)) == discriminant => "JumpForward",
            _ if std::mem::discriminant(&optimizer::Instruction::JumpBackward(0)) == discriminant => "JumpBackward",
            _ => "Unknown",
        };
        eprintln!("  {}: {}", name, count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_cli_parsing() {
        let args = vec!["brainfuck-interpreter", "test.bf"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.file, PathBuf::from("test.bf"));
        assert!(!cli.debug);
        assert_eq!(cli.memory_size, 30000);
        assert!(!cli.no_optimize);
        assert!(!cli.stats);
    }

    #[test]
    fn test_cli_with_options() {
        let args = vec![
            "brainfuck-interpreter",
            "--debug",
            "--memory-size", "50000",
            "--no-optimize",
            "--stats",
            "test.bf"
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.file, PathBuf::from("test.bf"));
        assert!(cli.debug);
        assert_eq!(cli.memory_size, 50000);
        assert!(cli.no_optimize);
        assert!(cli.stats);
    }
}
