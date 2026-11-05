# Research: Semantic Parsing in the LLM Era — First Principles

**Branch:** `research/first-principles` (fork of percyliang/sempre)
**Objective:** a publishable paper, built on working code, not speculation.

## Thesis
An agent needs: propose, constrain, execute, verify, remember.
An LLM only proposes: `P(next token | context)`.
SEMPRE constrained, executed, learned from denotations, and grew via macros — but its proposer (hand grammar + log-linear features) was too weak.
Combined: `Posterior(f|x) ∝ LLM(f|x) · 1[TypeCheck(f)] · 1[Execute(f,W) ≈ y]`
The LLM proposes. The symbolic system disposes.

## What is dead / what is alive
Dead: hand grammars for open domains, feature templates, Freebase/Virtuoso, the Java 8 / Ant / Ruby stack as a product.
Alive: typed compositional programs, learning from denotations (now: RL with verifiers), macro/library learning, interactive language growth.

## This branch
- [x] Typed Formula IR prototype (`formula_ir.py`): parse → type-check → execute → verify SEMPRE formulas. 7 tests pass.
- [x] Experiment 01 (synthetic): verification layers prune invalid programs but not semantic errors. See `run_experiment.py`, `PAPER.md`.
- [ ] Step 0: resurrect SEMPRE 2.4 on modern JDK; golden-master outputs.
- [ ] Step 3: Overnight domain baseline with classic parser.
- [ ] Step 4: schema → grammar induction prototype.
- [ ] Step 5: LLM proposer + verifier vs LLM-only vs classic parser.
- [ ] Step 6: macro compression.

## Run it
    cd research
    python3 -m pytest test_formula_ir.py -q
    python3 run_experiment.py
