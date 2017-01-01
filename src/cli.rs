extern crate clap;

use cli::clap::{Arg, App};
use std::path::PathBuf;
use std;

#[derive(Debug)]
pub enum CliParseError {
    NotAProfilingFile(std::string::String),
}

#[derive(Debug)]
pub struct Args {
    pub file_path: PathBuf,
}

impl Args {
    pub fn parse() -> Result<Args, CliParseError> {
        let matches = cli().get_matches();
        let prof_file = try!(matches.value_of("input_file")
            .ok_or(CliParseError::NotAProfilingFile(String::from("An input file is required.")))
            .map(PathBuf::from));
        let args = Args { file_path: prof_file };
        Ok(args)
    }
}

pub fn cli() -> App<'static, 'static> {
    let prof_file_arg = Arg::with_name("input_file")
        .long("file")
        .short("f")
        .value_name("PATH_TO_FILE")
        .help("The path to a valid .prof file.")
        .required(true);
    let app = App::new("Provis")
        .version("0.0.1")
        .author("Alfredo Di Napoli")
        .about("Interactive Haskell .prof visualiser.")
        .arg(prof_file_arg);
    app
}
