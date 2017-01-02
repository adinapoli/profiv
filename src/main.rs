
#[macro_use] extern crate nom;

mod cli;
mod ui;
mod parser;

use ui::UI;
use parser::parse_prof;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use nom::{IResult};

#[derive(Debug)]
enum AppError {
    CliError(cli::CliParseError),
    IOError(std::io::Error),
    UIError(ui::UIError),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::IOError(err)
    }
}

impl From<ui::UIError> for AppError {
    fn from(err: ui::UIError) -> AppError {
        AppError::UIError(err)
    }
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
    let file_path = args.file_path;
    let mut prof_file = try!(File::open(&file_path));
    let mut profile   = String::new();
    try!(prof_file.read_to_string(&mut profile));
    match parse_prof(profile.as_bytes()) {
        IResult::Done(_, prof) => {
            let ui = try!(UI::new());
            ui.render_loop(prof);
            Ok(())
        },
        _ => {
            println!("Failed to parse {:?}. Report this as a bug.", file_path);
            process::exit(1)
        }
    }
}
