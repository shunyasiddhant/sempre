//! sempre-ir: typed IR for SEMPRE formulas. Byte-cursor parser (no token alloc), Op as Copy enum (no string hashing hot path), small-Vec envs.
use std::fmt;
#[derive(Debug, Clone, PartialEq)]
pub enum Expr { Number(f64), Str(Box<str>), Bool(bool), Var(Box<str>), Lambda(Box<str>, Box<Expr>), List(Vec<Expr>), Call(Op, Vec<Expr>) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op { Add, Sub, Mul, Div, Lt, Le, Gt, Ge, Eq, And, Or, Not, If, Map, Select }
impl Op {
    fn from_name(s: &str) -> Option<Op> { Some(match s { "+" => Op::Add, "-" => Op::Sub, "*" => Op::Mul, "/" => Op::Div, "<" => Op::Lt, "<=" => Op::Le, ">" => Op::Gt, ">=" => Op::Ge, "=" => Op::Eq, "and" => Op::And, "or" => Op::Or, "not" => Op::Not, "if" => Op::If, "map" => Op::Map, "select" => Op::Select, _ => return None }) }
    fn name(self) -> &'static str { match self { Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/", Op::Lt => "<", Op::Le => "<=", Op::Gt => ">", Op::Ge => ">=", Op::Eq => "=", Op::And => "and", Op::Or => "or", Op::Not => "not", Op::If => "if", Op::Map => "map", Op::Select => "select" } }
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
            "call" => { let fname = self.ident().to_string(); let op = Op::from_name(&fname).ok_or_else(|| format!("unknown function {fname:?} (not in registry)"))?; let mut args = Vec::new(); while self.peek() != Some(b')') { args.push(self.parse_expr()?); } self.i += 1; Ok(Expr::Call(op, args)) }
            _ => Err(format!("unknown formula head {head:?}")),
        }
    }
}
pub fn parse(s: &str) -> Result<Expr, String> { let mut p = Parser::new(s); let e = p.parse_expr()?; if p.peek().is_some() { return Err("trailing tokens".into()); } Ok(e) }
#[derive(Debug, Clone, PartialEq)]
pub enum Type { Number, Str, Bool, Any, List(Box<Type>), Func(Box<Type>) }
impl Type { pub fn name(&self) -> String { match self { Type::Number => "number".into(), Type::Str => "string".into(), Type::Bool => "boolean".into(), Type::Any => "any".into(), Type::List(t) => format!("list[{}]", t.name()), Type::Func(r) => format!("function[any->{}]", r.name()) } } }
pub fn type_of(e: &Expr, env: &[(String, Type)]) -> Result<Type, String> {
    match e {
        Expr::Number(_) => Ok(Type::Number), Expr::Str(_) => Ok(Type::Str), Expr::Bool(_) => Ok(Type::Bool),
        Expr::Var(n) => env.iter().rev().find(|(k, _)| k == &**n).map(|(_, v)| v.clone()).ok_or_else(|| format!("unbound variable {n}")),
        Expr::List(items) => { let mut ts: Vec<Type> = Vec::new(); for it in items { let t = type_of(it, env)?; if !ts.contains(&t) { ts.push(t); } } Ok(Type::List(Box::new(if ts.len() == 1 { ts.into_iter().next().unwrap() } else { Type::Any }))) }
        Expr::Lambda(p, body) => { let mut env2 = env.to_vec(); env2.push((p.to_string(), Type::Any)); Ok(Type::Func(Box::new(type_of(body, &env2)?))) }
        Expr::Call(op, args) => {
            let mut ts = Vec::with_capacity(args.len()); for a in args { ts.push(type_of(a, env)?); }
            let num = ts.iter().all(|t| *t == Type::Number);
            match op {
                Op::Add if num => Ok(Type::Number),
                Op::Add if ts.iter().all(|t| *t == Type::Str) => Ok(Type::Str),
                Op::Add => Err("+ requires all-number or all-string".into()),
                Op::Sub | Op::Mul | Op::Div if num => Ok(Type::Number),
                Op::Sub | Op::Mul | Op::Div => Err("arithmetic requires numbers".into()),
                Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Eq if ts.len() >= 2 && ts.iter().all(|t| *t == ts[0]) && (ts[0] == Type::Number || ts[0] == Type::Str) => Ok(Type::Bool),
                Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Eq => Err("comparison requires comparable same-type args".into()),
                Op::And | Op::Or if ts.iter().all(|t| *t == Type::Bool) => Ok(Type::Bool),
                Op::And | Op::Or => Err("and/or require booleans".into()),
                Op::Not if ts == [Type::Bool] => Ok(Type::Bool), Op::Not => Err("not requires boolean".into()),
                Op::If if ts.len() == 3 && ts[0] == Type::Bool && ts[1] == ts[2] => Ok(ts[1].clone()), Op::If => Err("if requires (boolean, T, T)".into()),
                Op::Map => match (&ts[0], &ts[1]) { (Type::List(_), Type::Func(r)) => Ok(Type::List(r.clone())), _ => Err("map requires (list, function)".into()) },
                Op::Select => match (&ts[0], &ts[1]) { (Type::List(_), Type::Func(_)) => Ok(ts[0].clone()), _ => Err("select requires (list, function)".into()) },
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Value { Number(f64), Str(Box<str>), Bool(bool), List(Vec<Value>) }
pub fn execute(e: &Expr, env: &mut Vec<(Box<str>, Value)>) -> Result<Value, String> {
    match e {
        Expr::Number(v) => Ok(Value::Number(*v)), Expr::Str(s) => Ok(Value::Str(s.clone())), Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Var(n) => env.iter().rev().find(|(k, _)| k == n).map(|(_, v)| v.clone()).ok_or_else(|| format!("unbound variable {n}")),
        Expr::List(items) => { let mut out = Vec::with_capacity(items.len()); for it in items { out.push(execute(it, env)?); } Ok(Value::List(out)) }
        Expr::Lambda(_, _) => Err("lambda only supported as map/select argument".into()),
        Expr::Call(op, args) => {
            match op {
                Op::If => return if execute(&args[0], env)? == Value::Bool(true) { execute(&args[1], env) } else { execute(&args[2], env) },
                Op::Map => { let xs = match execute(&args[0], env)? { Value::List(v) => v, _ => return Err("map: not a list".into()) }; let (p, body) = match &args[1] { Expr::Lambda(p, b) => (p, b), _ => return Err("map: second arg must be lambda".into()) }; let mut out = Vec::with_capacity(xs.len()); for x in xs { env.push((p.clone(), x)); let r = execute(body, env); env.pop(); out.push(r?); } return Ok(Value::List(out)); }
                Op::Select => { let xs = match execute(&args[0], env)? { Value::List(v) => v, _ => return Err("select: not a list".into()) }; let (p, body) = match &args[1] { Expr::Lambda(p, b) => (p, b), _ => return Err("select: second arg must be lambda".into()) }; let mut out = Vec::new(); for x in xs { env.push((p.clone(), x.clone())); let r = execute(body, env); env.pop(); if r? == Value::Bool(true) { out.push(x); } } return Ok(Value::List(out)); }
                _ => {}
            }
            let mut v = Vec::with_capacity(args.len()); for a in args { v.push(execute(a, env)?); }
            let nums = |v: &Vec<Value>| -> Option<Vec<f64>> { v.iter().map(|x| if let Value::Number(n) = x { Some(*n) } else { None }).collect() };
            match op {
                Op::Add => if let Some(ns) = nums(&v) { Ok(Value::Number(ns.iter().sum())) } else { let mut s = String::new(); for x in &v { if let Value::Str(t) = x { s.push_str(t); } else { return Err("+ type error at exec".into()); } } Ok(Value::Str(s.into_boxed_str())) },
                Op::Sub => { let ns = nums(&v).ok_or("sub: numbers only")?; Ok(Value::Number(ns[0] - ns[1..].iter().sum::<f64>())) }
                Op::Mul => { let ns = nums(&v).ok_or("mul: numbers only")?; Ok(Value::Number(ns.iter().product())) }
                Op::Div => { let ns = nums(&v).ok_or("div: numbers only")?; Ok(Value::Number(ns[0] / ns[1])) }
                Op::Lt => Ok(Value::Bool(cmp(&v, |a, b| a < b)?)), Op::Le => Ok(Value::Bool(cmp(&v, |a, b| a <= b)?)), Op::Gt => Ok(Value::Bool(cmp(&v, |a, b| a > b)?)), Op::Ge => Ok(Value::Bool(cmp(&v, |a, b| a >= b)?)),
                Op::Eq => Ok(Value::Bool(v[0] == v[1])), Op::And => Ok(Value::Bool(v.iter().all(|x| *x == Value::Bool(true)))), Op::Or => Ok(Value::Bool(v.iter().any(|x| *x == Value::Bool(true)))), Op::Not => Ok(Value::Bool(v[0] != Value::Bool(true))),
                _ => unreachable!(),
            }
        }
    }
}
fn cmp(v: &[Value], f: impl Fn(f64, f64) -> bool) -> Result<bool, String> { match (&v[0], &v[1]) { (Value::Number(a), Value::Number(b)) => Ok(f(*a, *b)), _ => Err("cmp: numbers only in this subset".into()) } }
#[derive(Debug)]
pub struct Verdict { pub parsed: bool, pub type_ok: bool, pub exec_ok: bool, pub result: Option<Value>, pub result_type: Option<String>, pub error: String }
pub fn verify(candidate: &str, expected_type: Option<&str>, expected: Option<&Value>, check_expected: bool) -> Verdict {
    let mut v = Verdict { parsed: false, type_ok: false, exec_ok: false, result: None, result_type: None, error: String::new() };
    let e = match parse(candidate) { Ok(e) => { v.parsed = true; e } Err(ex) => { v.error = format!("parse: {ex}"); return v; } };
    match type_of(&e, &[]) { Ok(t) => { v.result_type = Some(t.name()); v.type_ok = true; } Err(ex) => { v.error = format!("type: {ex}"); return v; } }
    if let Some(et) = expected_type { if v.result_type.as_deref() != Some(et) { v.error = format!("type: expected {et}"); v.type_ok = false; return v; } }
    match execute(&e, &mut Vec::new()) { Ok(r) => { v.exec_ok = true; if check_expected && Some(&r) != expected { v.error = format!("denotation: expected {:?}, got {:?}", expected, r); } v.result = Some(r); } Err(ex) => { v.error = format!("exec: {ex}"); } }
    v
}
pub fn first_verified(candidates: &[&str], expected_type: Option<&str>) -> Option<Verdict> { for c in candidates { let v = verify(c, expected_type, None, false); if v.parsed && v.type_ok && v.exec_ok && v.error.is_empty() { return Some(v); } } None }
#[cfg(test)]
mod tests {
    use super::*;
    fn ev(s: &str) -> Value { execute(&parse(s).unwrap(), &mut Vec::new()).unwrap() }
    #[test] fn roundtrip() { for s in ["(call + (number 3) (number 4))", "(list (number 1) (number 2))"] { assert_eq!(parse(&parse(s).unwrap().to_string()).unwrap().to_string(), parse(s).unwrap().to_string()); } }
    #[test] fn arithmetic() { assert_eq!(ev("(call + (number 3) (number 4))"), Value::Number(7.0)); assert_eq!(ev("(call + (string 3) (string 4) (string 5))"), Value::Str("345".into())); }
    #[test] fn compare_if() { assert_eq!(ev("(call if (call < (number 3) (number 4)) (string yes) (string no))"), Value::Str("yes".into())); }
    #[test] fn list_lambda() { assert_eq!(ev("(call map (list (number 3) (number 4)) (lambda x (call * (var x) (var x))))"), Value::List(vec![Value::Number(9.0), Value::Number(16.0)])); assert_eq!(ev("(call select (list (number 3) (number 4)) (lambda x (call < (var x) (number 3.5))))"), Value::List(vec![Value::Number(3.0)])); }
    #[test] fn type_rejection() { for bad in ["(call + (number 3) (string hi))", "(call - (string a) (string b))", "(call + (number 1) (var x))", "(call nosuchfn (number 1))"] { let v = verify(bad, None, None, false); assert!(!(v.type_ok && v.exec_ok), "{bad}"); } }
    #[test] fn first_verified_skips_bad() { let v = first_verified(&["(call + (number 3) (string hi))", "(call + (number 3) (number 4))"], Some("number")).unwrap(); assert_eq!(v.result, Some(Value::Number(7.0))); }
    #[test] fn semantics_needs_denotation() { let v = verify("(call * (number 3) (number 4))", None, Some(&Value::Number(7.0)), true); assert!(v.exec_ok && v.error.contains("denotation")); }
}
