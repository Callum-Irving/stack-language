use clap::{App, AppSettings, Arg, Parser};
use std::process::Command;
use std::{fs, path::PathBuf, process::exit};

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
    let input_file = PathBuf::from(matches.value_of("filename").unwrap());
    if !input_file.is_file() {
        eprintln!("ERROR: file provided does not exist\n");
        args.print_help().unwrap();
        exit(1);
    }
    let file_contents = match fs::read_to_string(&input_file) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("ERROR: could not read file");
            args.print_help().unwrap();
            exit(1);
        }
    };

    // TODO: Add option to not remove intermediate
    // TODO: Add option to only output assembly
    // TODO: Add option to enable/disable debug symbols

    let program = parser::parse(file_contents);
    generate_asm(program, input_file.with_extension("asm"));

    let nasm_out = Command::new("nasm")
        .arg("-felf64")
        .arg("-g")
        .arg("-o")
        .arg(input_file.with_extension("o").as_os_str())
        .arg(input_file.with_extension("asm").as_os_str())
        .output()
        .unwrap();
    print!("{}", std::str::from_utf8(&nasm_out.stdout).unwrap());
    eprint!("{}", std::str::from_utf8(&nasm_out.stderr).unwrap());
    let gcc = Command::new("gcc")
        .arg("-no-pie")
        .arg("-g")
        .arg("-o")
        .arg(input_file.with_extension("").as_os_str())
        .arg(input_file.with_extension("o").as_os_str())
        .output()
        .unwrap();
    print!("{}", std::str::from_utf8(&gcc.stdout).unwrap());
    eprint!("{}", std::str::from_utf8(&gcc.stderr).unwrap());
    // Command::new("rm")
    //     .arg(input_file.with_extension("asm").as_os_str())
    //     .arg(input_file.with_extension("o").as_os_str())
    //     .output()
    //     .unwrap();
}
