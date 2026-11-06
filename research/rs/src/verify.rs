use crate::core::parse;
use crate::exec::{execute, Value};
use crate::types::type_of;
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
