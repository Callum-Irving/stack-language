use clap::{App, AppSettings, Arg, Parser};
use std::process::Command;
use std::{fs, path::PathBuf, process::exit};
use std::io::Read;

use crate::codegen::generate_asm;

mod codegen;
mod parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

const RET_STACK_SIZE: u32 = 256;

fn main() {
    let mut args = App::new("badforth")
        .author("Callum Irving")
        .version("0.0.1")
        .about("A bad stack-oriented programming language")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::new("filename").help("The program source file"));
    let matches = args.get_matches_mut();
    let file = PathBuf::from(matches.value_of("filename").unwrap());
    if !file.is_file() {
        eprintln!("ERROR: file provided does not exist\n");
        args.print_help().unwrap();
        exit(1);
    }
    let file_contents = match fs::read_to_string(file) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("ERROR: could not read file");
            args.print_help().unwrap();
            exit(1);
        }
    };

    let program = parser::parse(file_contents);
    let file = generate_asm(program);
    file.sync_all().unwrap();
    drop(file);

    Command::new("nasm").arg("-felf64").arg("-g").arg("output.asm").output().unwrap();
    Command::new("gcc").arg("-no-pie").arg("-g").arg("output.o").output().unwrap();
    Command::new("rm").arg("output.asm").arg("output.o").output().unwrap();
}
