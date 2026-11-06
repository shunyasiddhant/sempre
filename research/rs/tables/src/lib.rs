//! sempre-tables: row-set = bitset (Vec<u64>); filter/intersect/union are word ops, count is popcount, superlative is scan+keep.
use std::fmt;
#[derive(Debug, Clone, PartialEq)]
pub enum Cell { Number(f64), Str(Box<str>), Bool(bool) }
impl Cell { pub fn as_num(&self) -> Option<f64> { if let Cell::Number(v) = self { Some(*v) } else { None } } }
impl fmt::Display for Cell { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self { Cell::Number(v) => write!(f, "{}", v), Cell::Str(s) => write!(f, "{}", s), Cell::Bool(b) => write!(f, "{}", b) } } }
#[derive(Debug, Clone)]
pub struct Table { pub header: Vec<String>, pub rows: Vec<Vec<Cell>> }
impl Table { pub fn col(&self, name: &str) -> Result<usize, String> { self.header.iter().position(|h| h == name).ok_or_else(|| format!("no column {name:?}; header={:?}", self.header)) } }
#[derive(Debug, Clone, PartialEq)]
pub struct RowSet { words: Vec<u64>, n: usize }
impl RowSet {
    pub fn all(n: usize) -> RowSet { let mut words = vec![u64::MAX; (n + 63) / 64]; let rem = n % 64; if rem != 0 { if let Some(w) = words.last_mut() { *w = (1u64 << rem) - 1; } } RowSet { words, n } }
    pub fn empty(n: usize) -> RowSet { RowSet { words: vec![0; (n + 63) / 64], n } }
    pub fn set(&mut self, i: usize) { self.words[i / 64] |= 1u64 << (i % 64); }
    pub fn get(&self, i: usize) -> bool { (self.words[i / 64] >> (i % 64)) & 1 == 1 }
    pub fn count(&self) -> usize { self.words.iter().map(|w| w.count_ones() as usize).sum() }
    pub fn intersect(&self, o: &RowSet) -> RowSet { RowSet { words: self.words.iter().zip(&o.words).map(|(a, b)| a & b).collect(), n: self.n } }
    pub fn union(&self, o: &RowSet) -> RowSet { RowSet { words: self.words.iter().zip(&o.words).map(|(a, b)| a | b).collect(), n: self.n } }
    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ { (0..self.n).filter(move |&i| self.get(i)) }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cmp { Eq, Gt, Lt, Ge, Le }
#[derive(Debug, Clone, PartialEq)]
pub enum Query { All, Filter(Box<Query>, usize, Cmp, Cell), ArgMax(Box<Query>, usize), ArgMin(Box<Query>, usize), Intersect(Box<Query>, Box<Query>), Union(Box<Query>, Box<Query>), Count(Box<Query>), Sum(Box<Query>, usize), Avg(Box<Query>, usize), Project(Box<Query>, usize) }
#[derive(Debug, Clone, PartialEq)]
pub enum Answer { Rows(RowSet), Number(f64), Cells(Vec<Cell>) }
pub fn eval(q: &Query, t: &Table) -> Result<Answer, String> {
    match q {
        Query::All => Ok(Answer::Rows(RowSet::all(t.rows.len()))),
        Query::Filter(sub, col, cmp, want) => { let rows = rows_of(eval(sub, t)?)?; let mut out = RowSet::empty(t.rows.len()); for i in rows.indices() { let c = &t.rows[i][*col]; let ok = match cmp { Cmp::Eq => c == want, Cmp::Gt => c.as_num().map_or(false, |v| want.as_num().map_or(false, |w| v > w)), Cmp::Lt => c.as_num().map_or(false, |v| want.as_num().map_or(false, |w| v < w)), Cmp::Ge => c.as_num().map_or(false, |v| want.as_num().map_or(false, |w| v >= w)), Cmp::Le => c.as_num().map_or(false, |v| want.as_num().map_or(false, |w| v <= w)) }; if ok { out.set(i); } } Ok(Answer::Rows(out)) }
        Query::ArgMax(sub, col) => Ok(Answer::Rows(superlative(sub, t, *col, true)?)),
        Query::ArgMin(sub, col) => Ok(Answer::Rows(superlative(sub, t, *col, false)?)),
        Query::Intersect(a, b) => Ok(Answer::Rows(rows_of(eval(a, t)?)?.intersect(&rows_of(eval(b, t)?)?))),
        Query::Union(a, b) => Ok(Answer::Rows(rows_of(eval(a, t)?)?.union(&rows_of(eval(b, t)?)?))),
        Query::Count(sub) => Ok(Answer::Number(rows_of(eval(sub, t)?)?.count() as f64)),
        Query::Sum(sub, col) => { let rows = rows_of(eval(sub, t)?)?; Ok(Answer::Number(rows.indices().filter_map(|i| t.rows[i][*col].as_num()).sum())) }
        Query::Avg(sub, col) => { let rows = rows_of(eval(sub, t)?)?; let vals: Vec<f64> = rows.indices().filter_map(|i| t.rows[i][*col].as_num()).collect(); if vals.is_empty() { Err("avg of empty".into()) } else { Ok(Answer::Number(vals.iter().sum::<f64>() / vals.len() as f64)) } }
        Query::Project(sub, col) => { let rows = rows_of(eval(sub, t)?)?; Ok(Answer::Cells(rows.indices().map(|i| t.rows[i][*col].clone()).collect())) }
    }
}
fn rows_of(a: Answer) -> Result<RowSet, String> { if let Answer::Rows(r) = a { Ok(r) } else { Err("expected row-set".into()) } }
fn superlative(sub: &Query, t: &Table, col: usize, max: bool) -> Result<RowSet, String> { let rows = rows_of(eval(sub, t)?)?; let mut best: Option<f64> = None; for i in rows.indices() { if let Some(v) = t.rows[i][col].as_num() { best = Some(best.map_or(v, |b: f64| if max { b.max(v) } else { b.min(v) })); } } let best = best.ok_or("superlative on empty/non-numeric")?; let mut out = RowSet::empty(t.rows.len()); for i in rows.indices() { if t.rows[i][col].as_num() == Some(best) { out.set(i); } } Ok(out) }
pub fn parse_query(s: &str, t: &Table) -> Result<Query, String> { let toks = tokenize(s); let mut p = P { toks, i: 0 }; let q = pq(&mut p, t)?; if p.i != p.toks.len() { return Err("trailing tokens".into()); } Ok(q) }
struct P { toks: Vec<String>, i: usize }
impl P { fn next(&mut self) -> Result<String, String> { let t = self.toks.get(self.i).cloned().ok_or("eof")?; self.i += 1; Ok(t) } fn expect(&mut self, s: &str) -> Result<(), String> { if self.next()? == s { Ok(()) } else { Err(format!("expected {s}")) } } }
fn tokenize(s: &str) -> Vec<String> { let mut out = Vec::new(); let b = s.as_bytes(); let mut i = 0; while i < b.len() { match b[i] { c if c.is_ascii_whitespace() => i += 1, b'(' | b')' => { out.push((b[i] as char).to_string()); i += 1; } b'"' => { let mut j = i + 1; while j < b.len() && b[j] != b'"' { j += 1; } out.push(s[i..=j].to_string()); i = j + 1; } _ => { let mut j = i; while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'(' && b[j] != b')' { j += 1; } out.push(s[i..j].to_string()); i = j; } } } out }
fn pq(p: &mut P, t: &Table) -> Result<Query, String> {
    p.expect("(")?; let head = p.next()?;
    let q = match head.as_str() {
        "all" => Query::All,
        "filter" => { let sub = pq(p, t)?; let col = t.col(&p.next()?)?; let cmp = match p.next()?.as_str() { "eq" => Cmp::Eq, "gt" => Cmp::Gt, "lt" => Cmp::Lt, "ge" => Cmp::Ge, "le" => Cmp::Le, o => return Err(format!("bad cmp {o}")) }; let cell = pcell(p)?; Query::Filter(Box::new(sub), col, cmp, cell) }
        "argmax" => { let sub = pq(p, t)?; let col = t.col(&p.next()?)?; Query::ArgMax(Box::new(sub), col) }
        "argmin" => { let sub = pq(p, t)?; let col = t.col(&p.next()?)?; Query::ArgMin(Box::new(sub), col) }
        "intersect" => { let a = pq(p, t)?; let b = pq(p, t)?; Query::Intersect(Box::new(a), Box::new(b)) }
        "union" => { let a = pq(p, t)?; let b = pq(p, t)?; Query::Union(Box::new(a), Box::new(b)) }
        "count" => Query::Count(Box::new(pq(p, t)?)),
        "sum" => { let sub = pq(p, t)?; let col = t.col(&p.next()?)?; Query::Sum(Box::new(sub), col) }
        "avg" => { let sub = pq(p, t)?; let col = t.col(&p.next()?)?; Query::Avg(Box::new(sub), col) }
        "project" => { let sub = pq(p, t)?; let col = t.col(&p.next()?)?; Query::Project(Box::new(sub), col) }
        o => return Err(format!("unknown query head {o:?}")),
    }; p.expect(")")?; Ok(q)
}
fn pcell(p: &mut P) -> Result<Cell, String> { p.expect("(")?; let head = p.next()?; let v = p.next()?; p.expect(")")?; Ok(match head.as_str() { "number" => Cell::Number(v.parse().map_err(|_| "bad number")?), "string" => Cell::Str(v.trim_matches('"').into()), "boolean" => Cell::Bool(v == "true"), o => return Err(format!("bad cell {o}")) }) }
pub fn demo_table() -> Table { let s = |x: &str| Cell::Str(x.into()); let n = |x: f64| Cell::Number(x); Table { header: vec!["State".into(), "Capital".into(), "Population".into()], rows: vec![vec![s("California"), s("Sacramento"), n(39.0)], vec![s("Texas"), s("Austin"), n(30.0)], vec![s("Oregon"), s("Salem"), n(4.0)], vec![s("Nevada"), s("Carson City"), n(3.0)], vec![s("Washington"), s("Olympia"), n(8.0)]] } }
#[cfg(test)]
mod tests {
    use super::*;
    fn run(q: &str) -> Answer { let t = demo_table(); eval(&parse_query(q, &t).unwrap(), &t).unwrap() }
    fn cells(a: Answer) -> Vec<String> { if let Answer::Cells(cs) = a { cs.iter().map(|c| c.to_string()).collect() } else { panic!("not cells") } }
    #[test] fn capital_of_largest_state() { assert_eq!(cells(run("(project (argmax (all) Population) Capital)")), vec!["Sacramento"]); }
    #[test] fn count_big_states() { assert_eq!(run("(count (filter (all) Population gt (number 10)))"), Answer::Number(2.0)); }
    #[test] fn capital_of_california() { assert_eq!(cells(run("(project (filter (all) State eq (string California)) Capital)")), vec!["Sacramento"]); }
    #[test] fn smallest_and_sum() { assert_eq!(cells(run("(project (argmin (all) Population) State)")), vec!["Nevada"]); assert_eq!(run("(sum (all) Population)"), Answer::Number(84.0)); }
    #[test] fn intersect_union() { assert_eq!(run("(count (intersect (filter (all) Population gt (number 5)) (filter (all) Capital eq (string Austin))))"), Answer::Number(1.0)); assert_eq!(run("(count (union (filter (all) State eq (string Nevada)) (filter (all) State eq (string Oregon))))"), Answer::Number(2.0)); }
    #[test] fn bad_column_rejected() { let t = demo_table(); assert!(parse_query("(project (all) Nope)", &t).is_err()); }
}
