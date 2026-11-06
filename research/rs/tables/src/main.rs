use sempre_tables::{demo_table, eval, parse_query, Answer, Cell, RowSet, Table};
use std::time::Instant;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let t = demo_table();
    if args.len() > 1 && args[1] == "bench" { bench(); return; }
    let questions = ["(project (argmax (all) Population) Capital)", "(count (filter (all) Population gt (number 10)))", "(project (filter (all) State eq (string California)) Capital)", "(sum (all) Population)"];
    for q in questions { let parsed = parse_query(q, &t).unwrap(); println!("{q}\n  -> {:?}", eval(&parsed, &t).unwrap()); }
    let _ = Answer::Number(0.0); let _ = Cell::Bool(true); let _ = RowSet::all(1);
}
fn bench() {
    let n = 100_000usize; let mut rows = Vec::with_capacity(n);
    for i in 0..n { rows.push(vec![Cell::Str(format!("S{}", i % 50).into()), Cell::Number((i % 1000) as f64)]); }
    let t = Table { header: vec!["Name".into(), "Val".into()], rows };
    let q = parse_query("(count (filter (all) Val gt (number 500)))", &t).unwrap();
    let iters = 200; let t0 = Instant::now(); let mut acc = 0.0;
    for _ in 0..iters { if let Answer::Number(v) = eval(&q, &t).unwrap() { acc += v; } }
    let dt = t0.elapsed();
    println!("rows={n} iters={iters} checksum={acc} elapsed={:?} -> {:.0} eval/sec", dt, iters as f64 / dt.as_secs_f64());
}
