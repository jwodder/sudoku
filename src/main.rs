use anyhow::Context;
use clap::Parser;
use patharg::InputArg;
use std::process::ExitCode;
use sudoku::Puzzle;

/// Solve Sudoku puzzles
#[derive(Parser)]
#[clap(version)]
struct Arguments {
    /// Output the solution with borders and spacing
    #[clap(short = 'P', long)]
    pretty: bool,

    /// File containing Sudoku puzzle to solve [default: read from stdin]
    #[clap(default_value_t, hide_default_value = true)]
    infile: InputArg,
}

fn main() -> Result<ExitCode, anyhow::Error> {
    let args = Arguments::parse();
    let puzzle = args
        .infile
        .read_to_string()
        .context("Error reading input")?
        .parse::<Puzzle>()
        .context("Invalid input")?;
    match puzzle.solve() {
        Some(s) => {
            if args.pretty {
                println!("{s:#}");
            } else {
                println!("{s}");
            }
            Ok(ExitCode::SUCCESS)
        }
        None => {
            eprintln!("No solution");
            Ok(ExitCode::FAILURE)
        }
    }
}
