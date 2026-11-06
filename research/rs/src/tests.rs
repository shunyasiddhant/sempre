#[cfg(test)]
mod tests {
    use crate::core::parse;
    use crate::exec::{execute, execute_w, Value, World};
    use crate::verify::{first_verified, verify};
    fn ev(s: &str) -> Value { execute(&parse(s).unwrap(), &mut Vec::new()).unwrap() }
    #[test] fn roundtrip() { for s in ["(call + (number 3) (number 4))", "(list (number 1) (number 2))"] { assert_eq!(parse(&parse(s).unwrap().to_string()).unwrap().to_string(), parse(s).unwrap().to_string()); } }
    #[test] fn arithmetic() { assert_eq!(ev("(call + (number 3) (number 4))"), Value::Number(7.0)); assert_eq!(ev("(call + (string 3) (string 4) (string 5))"), Value::Str("345".into())); }
    #[test] fn compare_if() { assert_eq!(ev("(call if (call < (number 3) (number 4)) (string yes) (string no))"), Value::Str("yes".into())); }
    #[test] fn list_lambda() { assert_eq!(ev("(call map (list (number 3) (number 4)) (lambda x (call * (var x) (var x))))"), Value::List(vec![Value::Number(9.0), Value::Number(16.0)])); assert_eq!(ev("(call select (list (number 3) (number 4)) (lambda x (call < (var x) (number 3.5))))"), Value::List(vec![Value::Number(3.0)])); }
    #[test] fn type_rejection() { for bad in ["(call + (number 3) (string hi))", "(call - (string a) (string b))", "(call + (number 1) (var x))", "(call nosuchfn (number 1))"] { let v = verify(bad, None, None, false); assert!(!(v.type_ok && v.exec_ok), "{bad}"); } }
    #[test] fn first_verified_skips_bad() { let v = first_verified(&["(call + (number 3) (string hi))", "(call + (number 3) (number 4))"], Some("number")).unwrap(); assert_eq!(v.result, Some(Value::Number(7.0))); }
    #[test] fn semantics_needs_denotation() { let v = verify("(call * (number 3) (number 4))", None, Some(&Value::Number(7.0)), true); assert!(v.exec_ok && v.error.contains("denotation")); }
    fn demo_world() -> World {
        let mut w = World::default();
        w.add_table("employees", vec!["id", "name", "dept", "salary"], vec![vec![Value::Number(1.0), Value::Str("Alice".into()), Value::Str("eng".into()), Value::Number(120000.0)], vec![Value::Number(2.0), Value::Str("Bob".into()), Value::Str("sales".into()), Value::Number(70000.0)], vec![Value::Number(3.0), Value::Str("Carol".into()), Value::Str("eng".into()), Value::Number(95000.0)], vec![Value::Number(4.0), Value::Str("Dan".into()), Value::Str("hr".into()), Value::Number(60000.0)], vec![Value::Number(5.0), Value::Str("Eve".into()), Value::Str("sales".into()), Value::Number(85000.0)]]);
        w.add_table("departments", vec!["dept", "budget"], vec![vec![Value::Str("eng".into()), Value::Number(1000000.0)], vec![Value::Str("sales".into()), Value::Number(500000.0)], vec![Value::Str("hr".into()), Value::Number(200000.0)]]);
        w
    }
    fn evw(s: &str) -> Value { let w = demo_world(); execute_w(&parse(s).unwrap(), &mut Vec::new(), &w).unwrap() }
    #[test] fn table_count_select_field() { assert_eq!(evw("(call count (call select (table employees) (lambda r (call > (call field (var r) (string salary)) (number 70000))))"), Value::Number(3.0)); }
    #[test] fn table_sum_project_select() { assert_eq!(evw("(call sum (call project (call select (table employees) (lambda r (call = (call field (var r) (string dept)) (string eng)))) (string salary)))"), Value::Number(215000.0)); }
    #[test] fn table_argmax_project() { assert_eq!(evw("(call project (call argmax (table employees) (string salary)) (string name))"), Value::List(vec![Value::Str("Alice".into())])); }
    #[test] fn table_join_count() { assert_eq!(evw("(call count (call join (table employees) (table departments) (string dept) (string dept)))"), Value::Number(5.0)); }
}
