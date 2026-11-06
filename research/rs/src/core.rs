//! Core AST and byte-cursor parser.
use std::fmt;
#[derive(Debug, Clone, PartialEq)]
pub enum Expr { Number(f64), Str(Box<str>), Bool(bool), Var(Box<str>), Lambda(Box<str>, Box<Expr>), List(Vec<Expr>), Call(Op, Vec<Expr>), Table(Box<str>) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op { Add, Sub, Mul, Div, Lt, Le, Gt, Ge, Eq, And, Or, Not, If, Map, Select, Field, Project, Count, Sum, Avg, Argmax, Argmin, Join }
impl Op {
    fn from_name(s: &str) -> Option<Op> { Some(match s { "+" => Op::Add, "-" => Op::Sub, "*" => Op::Mul, "/" => Op::Div, "<" => Op::Lt, "<=" => Op::Le, ">" => Op::Gt, ">=" => Op::Ge, "=" => Op::Eq, "and" => Op::And, "or" => Op::Or, "not" => Op::Not, "if" => Op::If, "map" => Op::Map, "select" => Op::Select, "field" => Op::Field, "project" => Op::Project, "count" => Op::Count, "sum" => Op::Sum, "avg" => Op::Avg, "argmax" => Op::Argmax, "argmin" => Op::Argmin, "join" => Op::Join, _ => return None }) }
    fn name(self) -> &'static str { match self { Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/", Op::Lt => "<", Op::Le => "<=", Op::Gt => ">", Op::Ge => ">=", Op::Eq => "=", Op::And => "and", Op::Or => "or", Op::Not => "not", Op::If => "if", Op::Map => "map", Op::Select => "select", Op::Field => "field", Op::Project => "project", Op::Count => "count", Op::Sum => "sum", Op::Avg => "avg", Op::Argmax => "argmax", Op::Argmin => "argmin", Op::Join => "join" } }
}
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(v) => if v.fract() == 0.0 { write!(f, "(number {})", *v as i64) } else { write!(f, "(number {})", v) },
            Expr::Str(s) => if s.contains(' ') || s.is_empty() { write!(f, "(string \"{}\")", s) } else { write!(f, "(string {})", s) },
            Expr::Bool(b) => write!(f, "(boolean {})", b),
            Expr::Var(n) => write!(f, "(var {})", n),
            Expr::Lambda(p, b) => write!(f, "(lambda {} {})", p, b),
            Expr::List(items) => { write!(f, "(list")?; for it in items { write!(f, " {}", it)?; } write!(f, ")") }
            Expr::Call(op, args) => { write!(f, "(call {}", op.name())?; for a in args { write!(f, " {}", a)?; } write!(f, ")") }
            Expr::Table(n) => write!(f, "(table {})", n),
        }
    }
}
pub struct Parser<'a> { b: &'a [u8], i: usize }
impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self { Parser { b: s.as_bytes(), i: 0 } }
    fn ws(&mut self) { while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() { self.i += 1; } }
    fn peek(&mut self) -> Option<u8> { self.ws(); self.b.get(self.i).copied() }
    fn expect(&mut self, c: u8) -> Result<(), String> { if self.peek() == Some(c) { self.i += 1; Ok(()) } else { Err(format!("expected '{}' at {}", c as char, self.i)) } }
    fn ident(&mut self) -> &'a str { self.ws(); let st = self.i; while self.i < self.b.len() && !self.b[self.i].is_ascii_whitespace() && self.b[self.i] != b'(' && self.b[self.i] != b')' { self.i += 1; } std::str::from_utf8(&self.b[st..self.i]).unwrap_or("") }
    fn quoted(&mut self) -> Result<String, String> { self.expect(b'"')?; let mut out = String::new(); while self.i < self.b.len() && self.b[self.i] != b'"' { if self.b[self.i] == b'\\' { self.i += 1; if self.i >= self.b.len() { break; } } out.push(self.b[self.i] as char); self.i += 1; } self.expect(b'"')?; Ok(out) }
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.expect(b'(')?; let head = self.ident().to_string();
        match head.as_str() {
            "number" => { let v: f64 = self.ident().parse().map_err(|_| "bad number")?; self.expect(b')')?; Ok(Expr::Number(v)) }
            "string" => { let v = if self.peek() == Some(b'"') { self.quoted()? } else if self.peek() == Some(b')') { String::new() } else { self.ident().replace("\\ ", " ") }; self.expect(b')')?; Ok(Expr::Str(v.into_boxed_str())) }
            "boolean" => { let v = self.ident() == "true"; self.expect(b')')?; Ok(Expr::Bool(v)) }
            "var" => { let v = self.ident().to_string(); self.expect(b')')?; Ok(Expr::Var(v.into_boxed_str())) }
            "list" => { let mut items = Vec::new(); while self.peek() != Some(b')') { items.push(self.parse_expr()?); } self.i += 1; Ok(Expr::List(items)) }
            "lambda" => { let p = self.ident().to_string(); let body = self.parse_expr()?; self.expect(b')')?; Ok(Expr::Lambda(p.into_boxed_str(), Box::new(body))) }
            "table" => { let v = self.ident().to_string(); self.expect(b')')?; Ok(Expr::Table(v.into_boxed_str())) }
            "call" => { let fname = self.ident().to_string(); let op = Op::from_name(&fname).ok_or_else(|| format!("unknown function {fname:?} (not in registry)"))?; let mut args = Vec::new(); while self.peek() != Some(b')') { args.push(self.parse_expr()?); } self.i += 1; Ok(Expr::Call(op, args)) }
            _ => Err(format!("unknown formula head {head:?}")),
        }
    }
}
pub fn parse(s: &str) -> Result<Expr, String> { let mut p = Parser::new(s); let e = p.parse_expr()?; if p.peek().is_some() { return Err("trailing tokens".into()); } Ok(e) }
