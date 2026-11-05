use sempre_ir::{execute, parse, type_of, verify, Value};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "experiment" { experiment(); return; }
    let src = "(call + (number 3) (call * (number 4) (number 5)))";
    let n: u64 = if args.len() > 1 { args[1].parse().unwrap_or(1_000_000) } else { 1_000_000 };
    let t = Instant::now(); let mut acc = 0.0f64;
    for _ in 0..n {
        let e = parse(src).unwrap();
        let _ = type_of(&e, &[]).unwrap();
        if let Value::Number(v) = execute(&e, &mut Vec::new()).unwrap() { acc += v; }
    }
    let dt = t.elapsed();
    println!("formula: {src}");
    println!("iters={n} checksum={acc} elapsed={:?} -> {:.0} parse+type+exec/sec", dt, n as f64 / dt.as_secs_f64());
    let v = verify("(call + (number 3) (number 4))", Some("number"), None, false);
    println!("verify (call + 3 4) -> {:?} type={:?}", v.result, v.result_type);
}

fn experiment() {
    let levels = ["none", "parse", "parse+type", "parse+type+exec", "oracle(denotation)"];
    let mut wrong = [0usize; 5]; let total = 200usize; let mut task = 0usize;
    let mut seed: u64 = 0x9e3779b97f4a7c15;
    let mut next = move || { seed ^= seed >> 12; seed ^= seed << 25; seed ^= seed >> 27; seed.wrapping_mul(0x2545F4914F6CDD1D) };
    'outer: while task < total {
        for a in 1..=9i64 { for b in 1..=9i64 {
            if task >= total { break 'outer; }
            let gold = Value::Number((a + b) as f64);
            let correct = format!("(call + (number {a}) (number {b}))");
            let mut cands: Vec<String> = vec![format!("(call + (number {a}) (number {b})"), format!("(call + (number {a}) (string x))"), format!("(call + (number {a}) (var q))"), format!("(call * (number {a}) (number {b}))"), format!("(call - (number {a}) (number {b}))"), correct];
            for i in (1..cands.len()).rev() { let j = (next() % ((i + 1) as u64)) as usize; cands.swap(i, j); }
            for (li, level) in levels.iter().enumerate() {
                for c in &cands {
                    let v = verify(c, None, Some(&gold), level.starts_with("oracle"));
                    let ok = match *level { "none" => true, "parse" => v.parsed, "parse+type" => v.parsed && v.type_ok, "parse+type+exec" => v.parsed && v.type_ok && v.exec_ok, _ => v.parsed && v.type_ok && v.exec_ok && v.error.is_empty() };
                    if ok { if v.result.as_ref() != Some(&gold) { wrong[li] += 1; } break; }
                }
            }
            task += 1;
        } }
    }
    println!("tasks={total}");
    for (l, w) in levels.iter().zip(wrong.iter()) { println!("{l:22} wrong-accepted={w:3}/{total}"); }
}
