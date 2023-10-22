use anyhow::Context;
use lexopt::{Arg, Parser};
use patharg::InputArg;
use std::process::ExitCode;
use sudoku::Puzzle;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Run { pretty: bool, infile: InputArg },
    Help,
    Version,
}

impl Command {
    fn from_parser(mut parser: Parser) -> Result<Command, lexopt::Error> {
        let mut pretty = false;
        let mut infile: Option<InputArg> = None;
        while let Some(arg) = parser.next()? {
            match arg {
                Arg::Short('h') | Arg::Long("help") => return Ok(Command::Help),
                Arg::Short('V') | Arg::Long("version") => return Ok(Command::Version),
                Arg::Short('P') | Arg::Long("pretty") => pretty = true,
                Arg::Value(val) if infile.is_none() => {
                    infile = Some(InputArg::from_arg(val));
                }
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(Command::Run {
            pretty,
            infile: infile.unwrap_or_default(),
        })
    }

    fn run(self) -> anyhow::Result<ExitCode> {
        match self {
            Command::Run { pretty, infile } => {
                let puzzle = infile
                    .read_to_string()
                    .context("Error reading input")?
                    .parse::<Puzzle>()
                    .context("Invalid input")?;
                match puzzle.solve() {
                    Some(s) => {
                        if pretty {
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
            Command::Help => {
                println!("Usage: sudoku [-P|--pretty] [<INFILE>]");
                println!();
                println!("Solve a Sudoku puzzle");
                println!();
                println!("Options:");
                println!("  -P, --pretty      Output the solution with borders and spacing");
                println!("  -h, --help        Display this help message and exit");
                println!("  -V, --version     Show the program version and exit");
                Ok(ExitCode::SUCCESS)
            }
            Command::Version => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

fn main() -> anyhow::Result<ExitCode> {
    Command::from_parser(Parser::from_env())?.run()
}
