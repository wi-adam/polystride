use std::{env, process::ExitCode};

const HELP: &str = "\
Polystride — one executable, many platforms.

Usage: polystride [--help | --version]

Options:
  -h, --help       Show this help
  -V, --version    Show the development version

Status: project scaffold. Portable linking and packaging are not implemented.
See GOALS.md for the research milestones.";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => println!("{HELP}"),
        [arg] if arg == "--help" || arg == "-h" => println!("{HELP}"),
        [arg] if arg == "--version" || arg == "-V" => {
            println!("polystride {} (scaffold)", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Unsupported arguments. Run polystride --help.");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
