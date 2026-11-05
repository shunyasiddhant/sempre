import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from formula_ir import parse, execute, verify, first_verified

def ev(s):
    return execute(parse(s))

def test_roundtrip():
    for s in ["(call + (number 3) (number 4))", "(call * (number 2) (call + (number 1) (number 2)))", "(list (number 1) (number 2))"]:
        assert parse(parse(s).to_string()).to_string() == parse(s).to_string()

def test_arithmetic():
    assert ev("(call + (number 3) (number 4))") == 7
    assert ev("(call * (number 2) (call + (number 1) (number 2)))") == 6
    assert ev("(call + (string 3) (string 4) (string 5))") == "345"

def test_compare_and_if():
    assert ev("(call < (number 3) (number 4))") is True
    assert ev("(call if (call < (number 3) (number 4)) (string yes) (string no))") == "yes"

def test_list_lambda():
    assert ev("(call map (list (number 3) (number 4)) (lambda x (call * (var x) (var x))))") == [9, 16]
    assert ev("(call select (list (number 3) (number 4)) (lambda x (call < (var x) (number 3.5))))") == [3]

def test_type_rejection():
    for bad in ["(call + (number 3) (string hi))", "(call - (string a) (string b))", "(call + (number 1) (var x))", "(call nosuchfn (number 1))"]:
        v = verify(bad)
        assert not (v.type_ok and v.exec_ok), bad

def test_first_verified_skips_bad():
    cands = ["(call + (number 3) (string hi))", "(call + (number 3) (number 4))"]
    v = first_verified(cands, expected_type="number")
    assert v is not None and v.result == 7

def test_type_only_cannot_catch_semantics():
    v = verify("(call * (number 3) (number 4))", check_expected=True, expected_result=7)
    assert v.exec_ok and "denotation" in v.error
