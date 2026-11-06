use crate::core::{Expr, Op};
#[derive(Debug, Clone, PartialEq)]
pub enum Value { Number(f64), Str(Box<str>), Bool(bool), List(Vec<Value>), Table(Vec<Box<str>>, Vec<Vec<Value>>) }
#[derive(Debug, Default)]
pub struct World { tables: std::collections::HashMap<String, (Vec<Box<str>>, Vec<Vec<Value>>)> }
impl World {
    pub fn add_table(&mut self, name: &str, header: Vec<&str>, rows: Vec<Vec<Value>>) { self.tables.insert(name.to_string(), (header.iter().map(|h| Box::from(*h)).collect(), rows)); }
    fn get(&self, name: &str) -> Result<Value, String> { self.tables.get(name).map(|(h, r)| Value::Table(h.clone(), r.clone())).ok_or_else(|| format!("no table {name:?}")) }
}
pub fn execute(e: &Expr, env: &mut Vec<(Box<str>, Value)>) -> Result<Value, String> { execute_w(e, env, &World::default()) }
pub fn execute_w(e: &Expr, env: &mut Vec<(Box<str>, Value)>, world: &World) -> Result<Value, String> {
    match e {
        Expr::Number(v) => Ok(Value::Number(*v)), Expr::Str(s) => Ok(Value::Str(s.clone())), Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Var(n) => env.iter().rev().find(|(k, _)| k == n).map(|(_, v)| v.clone()).ok_or_else(|| format!("unbound variable {n}")),
        Expr::List(items) => { let mut out = Vec::with_capacity(items.len()); for it in items { out.push(execute_w(it, env, world)?); } Ok(Value::List(out)) }
        Expr::Lambda(_, _) => Err("lambda only supported as map/select argument".into()), Expr::Table(n) => world.get(n),
        Expr::Call(op, args) => {
            match op {
                Op::If => return if execute_w(&args[0], env, world)? == Value::Bool(true) { execute_w(&args[1], env, world) } else { execute_w(&args[2], env, world) },
                Op::Map => { let xs = match execute_w(&args[0], env, world)? { Value::List(v) => v, _ => return Err("map: not a list".into()) }; let (p, body) = match &args[1] { Expr::Lambda(p, b) => (p, b), _ => return Err("map: second arg must be lambda".into()) }; let mut out = Vec::with_capacity(xs.len()); for x in xs { env.push((p.clone(), x)); let r = execute_w(body, env, world); env.pop(); out.push(r?); } return Ok(Value::List(out)); }
                Op::Select => {
                    let input = execute_w(&args[0], env, world)?;
                    let (p, body) = match &args[1] { Expr::Lambda(p, b) => (p, b), _ => return Err("select: second arg must be lambda".into()) };
                    match input {
                        Value::List(xs) => { let mut out = Vec::new(); for x in xs { env.push((p.clone(), x.clone())); let r = execute_w(body, env, world); env.pop(); if r? == Value::Bool(true) { out.push(x); } } return Ok(Value::List(out)); }
                        Value::Table(header, rows) => { let mut out = Vec::new(); for row in rows { env.push((p.clone(), Value::Table(header.clone(), vec![row.clone()]))); let r = execute_w(body, env, world); env.pop(); if r? == Value::Bool(true) { out.push(row); } } return Ok(Value::Table(header, out)); }
                        _ => return Err("select: not a list/table".into()),
                    }
                }
                _ => {}
            }
            let mut v = Vec::with_capacity(args.len()); for a in args { v.push(execute_w(a, env, world)?); }
            let nums = |v: &Vec<Value>| -> Option<Vec<f64>> { v.iter().map(|x| if let Value::Number(n) = x { Some(*n) } else { None }).collect() };
            match op {
                Op::Add => if let Some(ns) = nums(&v) { Ok(Value::Number(ns.iter().sum())) } else { let mut s = String::new(); for x in &v { if let Value::Str(t) = x { s.push_str(t); } else { return Err("+ type error at exec".into()); } } Ok(Value::Str(s.into_boxed_str())) },
                Op::Sub => { let ns = nums(&v).ok_or("sub: numbers only")?; Ok(Value::Number(ns[0] - ns[1..].iter().sum::<f64>())) }
                Op::Mul => { let ns = nums(&v).ok_or("mul: numbers only")?; Ok(Value::Number(ns.iter().product())) }
                Op::Div => { let ns = nums(&v).ok_or("div: numbers only")?; Ok(Value::Number(ns[0] / ns[1])) }
                Op::Lt => Ok(Value::Bool(cmp(&v, |a, b| a < b)?)), Op::Le => Ok(Value::Bool(cmp(&v, |a, b| a <= b)?)), Op::Gt => Ok(Value::Bool(cmp(&v, |a, b| a > b)?)), Op::Ge => Ok(Value::Bool(cmp(&v, |a, b| a >= b)?)),
                Op::Eq => Ok(Value::Bool(v[0] == v[1])), Op::And => Ok(Value::Bool(v.iter().all(|x| *x == Value::Bool(true)))), Op::Or => Ok(Value::Bool(v.iter().any(|x| *x == Value::Bool(true)))), Op::Not => Ok(Value::Bool(v[0] != Value::Bool(true))),
                Op::Field => { let (header, rows) = table_parts(&v[0])?; let idx = col_idx(header, &v[1])?; if rows.is_empty() { return Err("field: empty table/row".into()); } Ok(rows[0][idx].clone()) }
                Op::Project => { let (header, rows) = table_parts(&v[0])?; let idx = col_idx(header, &v[1])?; Ok(Value::List(rows.iter().map(|r| r[idx].clone()).collect())) }
                Op::Count => match &v[0] { Value::Table(_, rows) => Ok(Value::Number(rows.len() as f64)), Value::List(xs) => Ok(Value::Number(xs.len() as f64)), _ => Err("count: table|list only".into()) },
                Op::Sum | Op::Avg => {
                    let vals: Vec<f64> = match &v[0] { Value::List(xs) => xs.iter().filter_map(|x| if let Value::Number(n) = x { Some(*n) } else { None }).collect(), Value::Table(header, rows) => { let idx = col_idx(header, &v[1])?; rows.iter().filter_map(|r| if let Value::Number(n) = &r[idx] { Some(*n) } else { None }).collect() } _ => return Err("sum/avg: list or table".into()), };
                    if vals.is_empty() { return Err("sum/avg: no numeric values".into()); }
                    let total: f64 = vals.iter().sum(); Ok(Value::Number(if *op == Op::Sum { total } else { total / vals.len() as f64 }))
                }
                Op::Argmax | Op::Argmin => {
                    let (header, rows) = table_parts(&v[0])?; let idx = col_idx(header, &v[1])?;
                    let mut best: Option<f64> = None;
                    for r in rows { if let Value::Number(n) = r[idx] { best = Some(best.map_or(n, |b: f64| if *op == Op::Argmax { b.max(n) } else { b.min(n) })); } }
                    let best = best.ok_or("argmax/argmin: no numeric values")?;
                    let out: Vec<Vec<Value>> = rows.iter().filter(|r| r[idx] == Value::Number(best)).cloned().collect();
                    Ok(Value::Table(header.clone(), out))
                }
                Op::Join => {
                    let (h1, r1) = table_parts(&v[0])?; let (h2, r2) = table_parts(&v[1])?;
                    let i1 = col_idx(h1, &v[2])?; let i2 = col_idx(h2, &v[3])?;
                    let dup = if let (Value::Str(a), Value::Str(b)) = (&v[2], &v[3]) { a == b } else { false };
                    let mut header: Vec<Box<str>> = h1.clone();
                    for (k, h) in h2.iter().enumerate() { if !(dup && k == i2) { header.push(h.clone()); } }
                    let mut out = Vec::new();
                    for a in r1 { for b in r2 { if a[i1] == b[i2] { let mut row = a.clone(); for (k, cell) in b.iter().enumerate() { if !(dup && k == i2) { row.push(cell.clone()); } } out.push(row); } } }
                    Ok(Value::Table(header, out))
                }
                _ => unreachable!(),
            }
        }
    }
}
fn cmp(v: &[Value], f: impl Fn(f64, f64) -> bool) -> Result<bool, String> { match (&v[0], &v[1]) { (Value::Number(a), Value::Number(b)) => Ok(f(*a, *b)), (Value::Str(a), Value::Str(b)) => Ok(f(if a < b { -1.0 } else if a > b { 1.0 } else { 0.0 }, 0.0)), _ => Err("cmp: type mismatch".into()) } }
fn table_parts(v: &Value) -> Result<(&Vec<Box<str>>, &Vec<Vec<Value>>), String> { if let Value::Table(h, r) = v { Ok((h, r)) } else { Err("expected table".into()) } }
fn col_idx(header: &[Box<str>], v: &Value) -> Result<usize, String> { if let Value::Str(name) = v { header.iter().position(|h| h == name).ok_or_else(|| format!("no column {name:?}")) } else { Err("column must be string".into()) } }
