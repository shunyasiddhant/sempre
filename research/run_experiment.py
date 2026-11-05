"""Experiment 01 (smoke, synthetic): how much does each verification layer prune? Deterministic noisy proposer: correct formula + syntax/type/unbound/semantic mutants. First-accepted wrong rate per filter. Denotation check is oracle-only."""
import random, sys, os
sys.path.insert(0, os.path.dirname(__file__))
from formula_ir import verify
random.seed(13)
LEVELS = ["none", "parse", "parse+type", "parse+type+exec", "oracle(denotation)"]

def accepted(v, level):
    if level == "none": return True
    if level == "parse": return v.parsed
    if level == "parse+type": return v.parsed and v.type_ok
    if level == "parse+type+exec": return v.parsed and v.type_ok and v.exec_ok
    return v.parsed and v.type_ok and v.exec_ok and not v.error

def run(n=200):
    stats = {l: {"wrong": 0, "none_accepted": 0} for l in LEVELS}
    for _ in range(n):
        a, b = random.randint(1, 9), random.randint(1, 9)
        gold = a + b
        correct = f"(call + (number {a}) (number {b}))"
        mutants = [f"(call + (number {a}) (number {b})", f"(call + (number {a}) (string x))", f"(call + (number {a}) (var q))", f"(call * (number {a}) (number {b}))", f"(call - (number {a}) (number {b}))"]
        cands = mutants + [correct]; random.shuffle(cands)
        for level in LEVELS:
            picked = None
            for c in cands:
                v = verify(c, check_expected=level.startswith("oracle"), expected_result=gold)
                if accepted(v, level): picked = v; break
            if picked is None: stats[level]["none_accepted"] += 1
            elif picked.result != gold: stats[level]["wrong"] += 1
    print(f"tasks={n}")
    for l in LEVELS:
        print(f"{l:22} wrong-accepted={stats[l]['wrong']:3}/{n}  none-accepted={stats[l]['none_accepted']}")
if __name__ == "__main__": run()
