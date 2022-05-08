use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use std::{fs::File, io::Write};

use crate::parser::{ComparisonOp, Literal, MathOp, Program, Stmt};
use crate::RET_STACK_SIZE;

const PREDEFINED: [&str; 6] = ["puts", "print", "drop", "dup", "swap", "read"];

pub fn generate_asm(ast: Program, filename: PathBuf) {
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
    let mut num_ifs = 0;

    // .text section
    writeln!(tmpfile, "section .text").unwrap();

    for (name, function) in &ast.functions {
        writeln!(tmpfile, "{}:", name).unwrap();
        if name != "main" {
            writeln!(tmpfile, "mov [ret_sp], rsp").unwrap();
            writeln!(tmpfile, "mov rsp, rax").unwrap();
        } else {
            writeln!(tmpfile, "mov qword [ret_sp], ret_stack_end").unwrap();
        }

        for stmt in &function.expr.0 {
            let (mut strings, new_ifs) =
                write_stmt(&mut tmpfile, stmt, &ast, string_literals.len(), num_ifs);
            string_literals.append(&mut strings);
            num_ifs = new_ifs;
        }
        if name != "main" {
            writeln!(tmpfile, "mov rax, rsp").unwrap();
            writeln!(tmpfile, "mov rsp, [ret_sp]").unwrap();
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
        .map(escape_str)
        .enumerate()
        .for_each(|(i, string)| {
            writeln!(tmpfile, "str_{}: db {}, 0", i, string).unwrap();
        });

    writeln!(tmpfile, "fint: db \"%d\", 0").unwrap();
    writeln!(tmpfile).unwrap();

    // .bss section
    writeln!(tmpfile, "section .bss").unwrap();

    writeln!(tmpfile, "ret_sp: resq 1").unwrap();
    writeln!(tmpfile, "ret_stack: resq {}", RET_STACK_SIZE).unwrap();
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

fn write_stmt(
    tmpfile: &mut File,
    stmt: &Stmt,
    ast: &Program,
    num_strs: usize,
    mut num_ifs: usize,
) -> (Vec<String>, usize) {
    let mut string_literals = vec![];
    match stmt {
        Stmt::Literal(literal) => match literal {
            Literal::Integer(num) => writeln!(tmpfile, "push {}", num).unwrap(),
            Literal::String(string) => {
                writeln!(tmpfile, "push str_{}", num_strs + string_literals.len()).unwrap();
                string_literals.push(string.to_owned());
            }
        }, // Push value to stack
        Stmt::MathOp(op) => match op {
            MathOp::Plus => {
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "pop rax").unwrap();
                writeln!(tmpfile, "add rax, rbx").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            MathOp::Minus => {
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "pop rax").unwrap();
                writeln!(tmpfile, "sub rax, rbx").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            MathOp::Multiply => {
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "pop rax").unwrap();
                writeln!(tmpfile, "imul rbx").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            MathOp::Divide => {
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "pop rax").unwrap();
                writeln!(tmpfile, "idiv rbx").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            MathOp::Mod => {
                writeln!(tmpfile, "xor rdx, rdx").unwrap();
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "pop rax").unwrap();
                writeln!(tmpfile, "idiv rbx").unwrap();
                writeln!(tmpfile, "push rdx").unwrap();
            }
        },
        Stmt::ComparisonOp(op) => match op {
            ComparisonOp::Eq => {
                writeln!(tmpfile, "xor rax, rax").unwrap();
                writeln!(tmpfile, "pop rcx").unwrap();
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "cmp rbx, rcx").unwrap();
                writeln!(tmpfile, "sete al").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            ComparisonOp::NotEq => {
                writeln!(tmpfile, "xor rax, rax").unwrap();
                writeln!(tmpfile, "pop rcx").unwrap();
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "cmp rbx, rcx").unwrap();
                writeln!(tmpfile, "setne al").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            ComparisonOp::Gt => {
                writeln!(tmpfile, "xor rax, rax").unwrap();
                writeln!(tmpfile, "pop rcx").unwrap();
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "cmp rbx, rcx").unwrap();
                writeln!(tmpfile, "setg al").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
            ComparisonOp::Lt => {
                writeln!(tmpfile, "xor rax, rax").unwrap();
                writeln!(tmpfile, "pop rcx").unwrap();
                writeln!(tmpfile, "pop rbx").unwrap();
                writeln!(tmpfile, "cmp rbx, rcx").unwrap();
                writeln!(tmpfile, "setl al").unwrap();
                writeln!(tmpfile, "push rax").unwrap();
            }
        },
        Stmt::Ident(name) => {
            if let Some(literal) = ast.constants.get(name) {
                match literal {
                    // TODO: Push number instead of using section in .data
                    Literal::Integer(_) => writeln!(tmpfile, "push qword [{}]", name).unwrap(),
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
                write_predefined(tmpfile, name);
            } else {
                panic!("Unknown identifier: {}", name);
            }
        }
        Stmt::IfStmt(if_stmt) => {
            writeln!(tmpfile, "pop rax").unwrap();
            writeln!(tmpfile, "cmp rax, 0").unwrap();
            writeln!(tmpfile, "je IF_{}", num_ifs).unwrap();
            let sto = num_ifs;
            num_ifs += 1;
            // if expression
            for stmt in &if_stmt.if_expr.0 {
                let mut results = write_stmt(tmpfile, stmt, ast, num_strs, num_ifs);
                string_literals.append(&mut results.0);
                num_ifs = results.1;
            }
            if if_stmt.else_expr.is_some() {
                writeln!(tmpfile, "jmp ELSE_{}", sto).unwrap();
            }
            writeln!(tmpfile, "IF_{}:", sto).unwrap();
            // else expression
            if if_stmt.else_expr.is_some() {
                for stmt in &if_stmt.else_expr.as_ref().unwrap().0 {
                    let mut results = write_stmt(
                        tmpfile,
                        stmt,
                        ast,
                        num_strs + string_literals.len(),
                        num_ifs,
                    );
                    string_literals.append(&mut results.0);
                    num_ifs = results.1;
                }
                writeln!(tmpfile, "ELSE_{}:", sto).unwrap();
            }
        }
    }
    (string_literals, num_ifs)
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
