use sempre_ir::{execute, execute_w, parse, type_of, verify, Value, World};
use std::time::Instant;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "experiment" { experiment(); return; }
    if args.len() > 1 && args[1] == "tables" { tables(); return; }
    let src = "(call + (number 3) (call * (number 4) (number 5)))";
    let n: u64 = if args.len() > 1 { args[1].parse().unwrap_or(1_000_000) } else { 1_000_000 };
    let t = Instant::now(); let mut acc = 0.0f64;
    for _ in 0..n { let e = parse(src).unwrap(); let _ = type_of(&e, &[]).unwrap(); if let Value::Number(v) = execute(&e, &mut Vec::new()).unwrap() { acc += v; } }
    let dt = t.elapsed();
    println!("formula: {src}");
    println!("iters={n} checksum={acc} elapsed={:?} -> {:.0} parse+type+exec/sec", dt, n as f64 / dt.as_secs_f64());
    let v = verify("(call + (number 3) (number 4))", Some("number"), None, false);
    println!("verify (call + 3 4) -> {:?} type={:?}", v.result, v.result_type);
}
fn demo_world() -> World {
    let mut w = World::default();
    w.add_table("employees", vec!["id", "name", "dept", "salary"], vec![vec![Value::Number(1.0), Value::Str("Alice".into()), Value::Str("eng".into()), Value::Number(120000.0)], vec![Value::Number(2.0), Value::Str("Bob".into()), Value::Str("sales".into()), Value::Number(70000.0)], vec![Value::Number(3.0), Value::Str("Carol".into()), Value::Str("eng".into()), Value::Number(95000.0)], vec![Value::Number(4.0), Value::Str("Dan".into()), Value::Str("hr".into()), Value::Number(60000.0)], vec![Value::Number(5.0), Value::Str("Eve".into()), Value::Str("sales".into()), Value::Number(85000.0)]]);
    w.add_table("departments", vec!["dept", "budget"], vec![vec![Value::Str("eng".into()), Value::Number(1000000.0)], vec![Value::Str("sales".into()), Value::Number(500000.0)], vec![Value::Str("hr".into()), Value::Number(200000.0)]]);
    w
}
fn tables() {
    let w = demo_world();
    let queries = [("how many employees earn > 70000?", "(call count (call select (table employees) (lambda r (call > (call field (var r) (string salary)) (number 70000))))"), ("total eng salary?", "(call sum (call project (call select (table employees) (lambda r (call = (call field (var r) (string dept)) (string eng)))) (string salary)))"), ("highest-paid employee?", "(call project (call argmax (table employees) (string salary)) (string name))"), ("employees joined with departments: how many rows?", "(call count (call join (table employees) (table departments) (string dept) (string dept)))")];
    for (q, src) in queries { let e = parse(src).unwrap(); let t = type_of(&e, &[]).unwrap(); let v = execute_w(&e, &mut Vec::new(), &w).unwrap(); println!("Q: {q}\n  formula: {src}\n  type: {} -> {:?}", t.name(), v); }
    let src = queries[1].1; let n = 200_000u64; let t0 = Instant::now(); let mut acc = 0.0;
    for _ in 0..n { let e = parse(src).unwrap(); let _ = type_of(&e, &[]).unwrap(); if let Value::Number(v) = execute_w(&e, &mut Vec::new(), &w).unwrap() { acc += v; } }
    let dt = t0.elapsed(); println!("table-query bench: iters={n} checksum={acc} elapsed={dt:?} -> {:.0}/sec", n as f64 / dt.as_secs_f64());
}
fn experiment() {
    let levels = ["none", "parse", "parse+type", "parse+type+exec", "oracle(denotation)"];
    let mut wrong = [0usize; 5]; let total = 200usize; let mut task = 0usize;
    let mut seed: u64 = 0x9e3779b97f4a7c15;
    let mut next = move || { seed ^= seed >> 12; seed ^= seed << 25; seed ^= seed >> 27; seed.wrapping_mul(0x2545F4914F6CDD1D) };
    'outer: while task < total { for a in 1..=9i64 { for b in 1..=9i64 { if task >= total { break 'outer; } let gold = Value::Number((a + b) as f64); let correct = format!("(call + (number {a}) (number {b}))"); let mut cands: Vec<String> = vec![format!("(call + (number {a}) (number {b})"), format!("(call + (number {a}) (string x))"), format!("(call + (number {a}) (var q))"), format!("(call * (number {a}) (number {b}))"), format!("(call - (number {a}) (number {b}))"), correct]; for i in (1..cands.len()).rev() { let j = (next() % ((i + 1) as u64)) as usize; cands.swap(i, j); } for (li, level) in levels.iter().enumerate() { for c in &cands { let v = verify(c, None, Some(&gold), level.starts_with("oracle")); let ok = match *level { "none" => true, "parse" => v.parsed, "parse+type" => v.parsed && v.type_ok, "parse+type+exec" => v.parsed && v.type_ok && v.exec_ok, _ => v.parsed && v.type_ok && v.exec_ok && v.error.is_empty() }; if ok { if v.result.as_ref() != Some(&gold) { wrong[li] += 1; } break; } } } task += 1; } } }
    println!("tasks={total}"); for (l, w) in levels.iter().zip(wrong.iter()) { println!("{l:22} wrong-accepted={w:3}/{total}"); }
}
