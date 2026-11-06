use crate::core::{Expr, Op};
#[derive(Debug, Clone, PartialEq)]
pub enum Type { Number, Str, Bool, Any, List(Box<Type>), Func(Box<Type>), Table }
impl Type { pub fn name(&self) -> String { match self { Type::Number => "number".into(), Type::Str => "string".into(), Type::Bool => "boolean".into(), Type::Any => "any".into(), Type::List(t) => format!("list[{}]", t.name()), Type::Func(r) => format!("function[any->{}]", r.name()), Type::Table => "table".into() } } }
pub fn type_of(e: &Expr, env: &[(String, Type)]) -> Result<Type, String> {
    match e {
        Expr::Number(_) => Ok(Type::Number), Expr::Str(_) => Ok(Type::Str), Expr::Bool(_) => Ok(Type::Bool), Expr::Table(_) => Ok(Type::Table),
        Expr::Var(n) => env.iter().rev().find(|(k, _)| k == &**n).map(|(_, v)| v.clone()).ok_or_else(|| format!("unbound variable {n}")),
        Expr::List(items) => { let mut ts: Vec<Type> = Vec::new(); for it in items { let t = type_of(it, env)?; if !ts.contains(&t) { ts.push(t); } } Ok(Type::List(Box::new(if ts.len() == 1 { ts.into_iter().next().unwrap() } else { Type::Any }))) }
        Expr::Lambda(p, body) => { let mut env2 = env.to_vec(); env2.push((p.to_string(), Type::Any)); Ok(Type::Func(Box::new(type_of(body, &env2)?))) }
        Expr::Call(op, args) => {
            let mut ts = Vec::with_capacity(args.len()); for a in args { ts.push(type_of(a, env)?); }
            let num = ts.iter().all(|t| *t == Type::Number || *t == Type::Any);
            match op {
                Op::Add if num => Ok(Type::Number),
                Op::Add if ts.iter().all(|t| *t == Type::Str) => Ok(Type::Str),
                Op::Add => Err("+ requires all-number or all-string".into()),
                Op::Sub | Op::Mul | Op::Div if num => Ok(Type::Number),
                Op::Sub | Op::Mul | Op::Div => Err("arithmetic requires numbers".into()),
                Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Eq if ts.len() >= 2 && ts.iter().all(|t| *t == Type::Number || *t == Type::Str || *t == Type::Any) && (ts.iter().all(|t| *t == ts[0]) || ts.iter().any(|t| *t == Type::Any)) => Ok(Type::Bool),
                Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Eq => Err("comparison requires comparable same-type args".into()),
                Op::And | Op::Or if ts.iter().all(|t| *t == Type::Bool || *t == Type::Any) => Ok(Type::Bool),
                Op::And | Op::Or => Err("and/or require booleans".into()),
                Op::Not if ts == [Type::Bool] => Ok(Type::Bool), Op::Not => Err("not requires boolean".into()),
                Op::If if ts.len() == 3 && ts[0] == Type::Bool && ts[1] == ts[2] => Ok(ts[1].clone()), Op::If => Err("if requires (boolean, T, T)".into()),
                Op::Map => match (&ts[0], &ts[1]) { (Type::List(_), Type::Func(r)) => Ok(Type::List(r.clone())), _ => Err("map requires (list, function)".into()) },
                Op::Select => match (&ts[0], &ts[1]) { (Type::List(_), Type::Func(_)) => Ok(ts[0].clone()), (Type::Table, Type::Func(_)) => Ok(Type::Table), _ => Err("select requires (list|table, function)".into()) },
                Op::Field => if ts.len() == 2 && ts[1] == Type::Str { Ok(Type::Any) } else { Err("field requires (row, string)".into()) },
                Op::Project => if ts.len() == 2 && ts[0] == Type::Table && ts[1] == Type::Str { Ok(Type::List(Box::new(Type::Any))) } else { Err("project requires (table, string)".into()) },
                Op::Count => if ts.len() == 1 && (ts[0] == Type::Table || matches!(ts[0], Type::List(_))) { Ok(Type::Number) } else { Err("count requires table|list".into()) },
                Op::Sum | Op::Avg => if (ts.len() == 1 && matches!(ts[0], Type::List(_))) || (ts.len() == 2 && ts[0] == Type::Table && ts[1] == Type::Str) { Ok(Type::Number) } else { Err("sum/avg require (list) or (table, string)".into()) },
                Op::Argmax | Op::Argmin => if ts.len() == 2 && ts[0] == Type::Table && ts[1] == Type::Str { Ok(Type::Table) } else { Err("argmax/argmin require (table, string)".into()) },
                Op::Join => if ts.len() == 4 && ts[0] == Type::Table && ts[1] == Type::Table && ts[2] == Type::Str && ts[3] == Type::Str { Ok(Type::Table) } else { Err("join requires (table, table, string, string)".into()) },
            }
        }
    }
}
