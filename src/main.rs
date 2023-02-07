use anyhow::Context;
use clap::Parser;
use either::Either;
use std::fs::File;
use std::io::{read_to_string, stdin};
use std::path::PathBuf;
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
    #[clap(default_value_os_t = PathBuf::from("-"), hide_default_value = true)]
    infile: PathBuf,
}

fn main() -> Result<ExitCode, anyhow::Error> {
    let args = Arguments::parse();
    let infile = if args.infile == PathBuf::from("-") {
        Either::Left(stdin())
    } else {
        Either::Right(File::open(args.infile).context("Error opening input file")?)
    };
    let puzzle = read_to_string(infile)
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
