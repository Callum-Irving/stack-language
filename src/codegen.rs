use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use std::{fs::File, io::Write};

use crate::parser::{ComparisonOp, Literal, MathOp, Program, Stmt};

const PREDEFINED: [&'static str; 6] = ["puts", "print", "drop", "dup", "swap", "read"];

pub fn generate_asm(mut ast: Program, filename: PathBuf) {
    // let mut tmpfile = NamedTempFile::new().unwrap();
    let mut tmpfile = File::create(filename).unwrap();

    // Declare external C functions and main function
    writeln!(tmpfile, "global main").unwrap();
    writeln!(
        tmpfile,
        "extern fputs, printf, fflush, stdout, malloc, free"
    )
    .unwrap();
    writeln!(tmpfile).unwrap();

    let mut string_literals = vec![];

    // .text section
    writeln!(tmpfile, "section .text").unwrap();

    for (name, function) in &ast.functions {
        writeln!(tmpfile, "main:").unwrap();
        for stmt in &function.expr.0 {
            match stmt {
                Stmt::Literal(literal) => match literal {
                    Literal::Integer(num) => writeln!(tmpfile, "push {}", num).unwrap(),
                    Literal::String(string) => {
                        writeln!(tmpfile, "push str_{}", string_literals.len()).unwrap();
                        string_literals.push(string.to_owned());
                    }
                }, // Push value to stack
                Stmt::MathOp(op) => match op {
                    MathOp::Plus => {}
                    MathOp::Minus => {}
                    MathOp::Multiply => {}
                    MathOp::Divide => {}
                    MathOp::Mod => {}
                },
                Stmt::ComparisonOp(op) => match op {
                    ComparisonOp::Eq => {}
                    ComparisonOp::NotEq => {}
                    ComparisonOp::Gt => {}
                    ComparisonOp::Lt => {}
                },
                Stmt::Ident(name) => {
                    // Check if ident is constant
                    //  if array, push <name>
                    //  if literal, push [<name>]
                    // Check if ident is function
                    //  Example call:
                    //      mov      rax, rsp
                    //      mov      rsp, [ret_sp]
                    //      call     add1
                    //      mov      [ret_sp], rsp
                    //      mov      rsp, rax
                    //      error
                    if let Some(literal) = ast.constants.get(name) {
                        match literal {
                            // TODO: Push number instead of using section in .data
                            Literal::Integer(_) => {
                                writeln!(tmpfile, "push qword [{}]", name).unwrap()
                            }
                            Literal::String(_) => writeln!(tmpfile, "push {}", name).unwrap(),
                        }
                    } else if ast.arrays.get(name).is_some() {
                        writeln!(tmpfile, "push {}", name).unwrap();
                    } else if ast.functions.get(name).is_some() {
                        writeln!(tmpfile, "mov rax, rsp").unwrap();
                        writeln!(tmpfile, "mov rsp, [ret_sp]").unwrap();
                        writeln!(tmpfile, "call {}", name).unwrap();
                        writeln!(tmpfile, "mov [ret_sp], rsp").unwrap();
                        writeln!(tmpfile, "mov rsp, rax").unwrap();
                    } else if PREDEFINED.contains(&name.as_str()) {
                        write_predefined(&mut tmpfile, name);
                    } else {
                        panic!("Unknown identifier: {}", name);
                    }
                }
                Stmt::IfStmt(if_stmt) => {
                    todo!();
                }
            }
        }
        writeln!(tmpfile, "ret").unwrap();
    }

    writeln!(tmpfile).unwrap();

    // .data section
    writeln!(tmpfile, "section .data").unwrap();

    for (name, constant) in &ast.constants {
        match constant {
            Literal::Integer(num) => writeln!(tmpfile, "{}: {}", name, num).unwrap(),
            Literal::String(string) => {
                writeln!(tmpfile, "{}: db {}, 0", name, escape_str(string.to_owned())).unwrap();
            }
        }
    }

    string_literals
        .into_iter()
        .map(|s| escape_str(s))
        .enumerate()
        .for_each(|(i, string)| {
            writeln!(tmpfile, "str_{}: db {}, 0", i, string).unwrap();
        });

    writeln!(tmpfile, "fint: db \"%d\", 0").unwrap();

    writeln!(tmpfile).unwrap();

    // .bss section
    writeln!(tmpfile, "section .bss").unwrap();

    writeln!(tmpfile, "ret_sp: resq 1").unwrap();
    writeln!(tmpfile, "ret_stack: resb 2048").unwrap();
    writeln!(tmpfile, "ret_stack_end: equ $").unwrap();

    for (name, num_bytes) in &ast.arrays {
        writeln!(tmpfile, "{}: resb {}", name, num_bytes).unwrap();
    }

    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    tmpfile.sync_all().unwrap();
}

fn escape_str(mut string: String) -> String {
    if string.ends_with("\\n\"") {
        string.truncate(string.len() - 3);
        string.push_str("\", 10");
    }
    string.replace("\\n", "\", 10, \"")
}

fn write_predefined(file: &mut File, ident: &str) {
    match ident {
        "dup" => {
            writeln!(file, "pop rax").unwrap();
            writeln!(file, "push rax").unwrap();
            writeln!(file, "push rax").unwrap();
        }
        "drop" => {
            writeln!(file, "pop rax").unwrap();
        }
        "swap" => {
            writeln!(file, "pop rax").unwrap();
            writeln!(file, "pop rbx").unwrap();
            writeln!(file, "push rax").unwrap();
            writeln!(file, "push rbx").unwrap();
        }
        "puts" => {
            writeln!(file, "pop rdi").unwrap();
            writeln!(file, "mov rsi, [stdout]").unwrap();
            writeln!(file, "call fputs").unwrap();
            writeln!(file, "mov rdi, [stdout]").unwrap();
            writeln!(file, "call fflush").unwrap();
        }
        "print" => {
            writeln!(file, "mov rdi, fint").unwrap();
            writeln!(file, "pop rsi").unwrap();
            writeln!(file, "mov al, 0").unwrap();
            writeln!(file, "call printf").unwrap();
        }
        "read" => {
            writeln!(file, "mov rax, 0").unwrap();
            writeln!(file, "mov rdi, 0").unwrap();
            writeln!(file, "pop rdx").unwrap(); // bytes to read
            writeln!(file, "pop rsi").unwrap(); // buffer
            writeln!(file, "syscall").unwrap();
            writeln!(file, "push rax").unwrap();
        }
        _ => panic!("invalid predefined word"),
    }
}
