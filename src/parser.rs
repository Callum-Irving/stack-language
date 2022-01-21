use std::collections::HashMap;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ProgramParser;

#[derive(Debug)]
pub struct Program {
    pub functions: HashMap<String, Function>,
    pub constants: HashMap<String, Literal>,
    pub arrays: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct Function {
    pub inputs: Vec<BasicType>,
    pub outputs: Vec<BasicType>,
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Constant {
    Single(Literal),
    Array(usize),
}

#[derive(Debug)]
pub enum BasicType {
    Pointer,
    Integer,
}

#[derive(Debug)]
pub struct Expr(pub Vec<Stmt>);

#[derive(Debug)]
pub enum Stmt {
    Literal(Literal),
    IfStmt(IfStmt),
    MathOp(MathOp),
    ComparisonOp(ComparisonOp),
    Ident(String),
}

#[derive(Debug)]
pub enum Literal {
    Integer(i64),
    String(String),
}

#[derive(Debug)]
pub struct IfStmt {
    pub if_expr: Expr,
    pub else_expr: Option<Expr>,
}

#[derive(Debug)]
pub enum MathOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
}

#[derive(Debug)]
pub enum ComparisonOp {
    Eq,
    NotEq,
    Gt,
    Lt,
}

pub fn parse(input: String) -> Program {
    // TODO: Handle error unwrapping
    let pairs = ProgramParser::parse(Rule::program, &input).unwrap();
    let mut program = Program {
        functions: HashMap::new(),
        constants: HashMap::new(),
        arrays: HashMap::new(),
    };

    for pair in pairs {
        match pair.as_rule() {
            Rule::constant => {
                let mut inners = pair.into_inner();
                let name = inners.next().unwrap().as_str().to_owned();
                let value_raw = inners.next().unwrap();
                match value_raw.as_rule() {
                    Rule::integer => {
                        program
                            .arrays
                            .insert(name, value_raw.as_str().parse().unwrap());
                    }
                    Rule::literal => {
                        let inner = value_raw.into_inner().next().unwrap();
                        program.constants.insert(
                            name,
                            match inner.as_rule() {
                                Rule::integer => Literal::Integer(inner.as_str().parse().unwrap()),
                                Rule::string => Literal::String(inner.as_str().to_owned()),
                                _ => panic!(),
                            },
                        );
                    }
                    _ => panic!(),
                };
            }
            Rule::function => {
                let mut func = Function {
                    inputs: vec![],
                    outputs: vec![],
                    expr: Expr(vec![]),
                };
                let mut inners = pair.into_inner();
                let name = inners.next().unwrap().as_str().to_owned();
                for input in inners.next().unwrap().into_inner() {
                    match input.as_str() {
                        "int" => func.inputs.push(BasicType::Integer),
                        "ptr" => func.inputs.push(BasicType::Pointer),
                        _ => panic!("invalid input type"),
                    }
                }
                for output in inners.next().unwrap().into_inner() {
                    match output.as_str() {
                        "int" => func.outputs.push(BasicType::Integer),
                        "ptr" => func.outputs.push(BasicType::Pointer),
                        _ => panic!(),
                    }
                }
                func.expr = parse_expr(inners.next().unwrap().into_inner().collect());
                program.functions.insert(name, func);
            }
            Rule::EOI => (),
            _ => panic!("OTHER RULE: {:?}", pair),
        }
    }

    program
}

fn parse_expr(expr: Vec<Pair<Rule>>) -> Expr {
    let expr: Vec<Pair<Rule>> = expr
        .into_iter()
        .map(|stmt| stmt.into_inner().next().unwrap())
        .collect();
    let mut statements = vec![];
    for statement in expr {
        statements.push(match statement.as_rule() {
            Rule::literal => parse_literal(statement),
            Rule::ifstmt => {
                let mut inner = statement.into_inner();
                let if_expr = parse_expr(inner.next().unwrap().into_inner().collect());
                let else_expr = if let Some(expression) = inner.next() {
                    Some(parse_expr(expression.into_inner().collect()))
                } else {
                    None
                };
                Stmt::IfStmt(IfStmt { if_expr, else_expr })
            }
            Rule::mathop => match statement.as_str() {
                "+" => Stmt::MathOp(MathOp::Plus),
                "-" => Stmt::MathOp(MathOp::Minus),
                "*" => Stmt::MathOp(MathOp::Multiply),
                "/" => Stmt::MathOp(MathOp::Divide),
                "%" => Stmt::MathOp(MathOp::Mod),
                _ => panic!(),
            },
            Rule::comparisonop => match statement.as_str() {
                "=?" => Stmt::ComparisonOp(ComparisonOp::Eq),
                "!=" => Stmt::ComparisonOp(ComparisonOp::NotEq),
                ">" => Stmt::ComparisonOp(ComparisonOp::Gt),
                "<" => Stmt::ComparisonOp(ComparisonOp::Lt),
                _ => panic!(),
            },
            Rule::ident => Stmt::Ident(statement.as_str().to_owned()),
            _ => panic!(),
        });
    }
    Expr(statements)
}

fn parse_literal(literal: Pair<Rule>) -> Stmt {
    match literal.as_rule() {
        Rule::integer => Stmt::Literal(Literal::Integer(literal.as_str().parse().unwrap())),
        Rule::string => Stmt::Literal(Literal::String(literal.as_str().to_owned())),
        _ => panic!(),
    }
}
