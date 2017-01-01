
#[macro_use] extern crate nom;

mod cli;
mod parser;

use std::process;

#[derive(Debug)]
enum AppError {
    CliError(cli::CliParseError),
}

fn main() {
   match cli::Args::parse().map_err(AppError::CliError).and_then(run) {
        Ok(()) => process::exit(0),
        Err(e) => {
            println!("{:?}", e);
            process::exit(1);
        }
   }
}

fn run(args: cli::Args) -> Result<(), AppError> {
    Ok(())
}
