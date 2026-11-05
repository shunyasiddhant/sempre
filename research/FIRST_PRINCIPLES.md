# First Principles

Forget the codebase. The problem: Given utterance x, world W, find program f with Execute(f, W) = y, such that f generalizes.

Six primitives, each chosen in 2013, each re-choosable in 2026:
1. Representation: Formula tree (interpreted) -> typed IR, compilable to WASM / native / SQL / Datalog.
2. Search: chart over hand grammar -> LLM proposal + symbolic pruning, AOT-compiled parsers, distilled small models.
3. Scoring: hand features -> LLM prior; execution as score.
4. Execution: Java reflection tree-walk -> columnar/bitset execution, hash-consed formulas, denotation caching.
5. Learning signal: denotation only -> execution traces, user definitions, counterexamples; RL with a verifier.
6. Knowledge: static Freebase (dead) -> live schemas (DB, OpenAPI, code).

Powers unlocked: reliability that compounds; systematic composition from few examples; learning from cheap outcomes; a personal language that grows (definitions to macros) and accelerates; governance by construction (effect types = policy); audit via derivations.

Limits (honest paper must include):
- No verifier / no executable world -> no power (open-ended text).
- Type+execution checks reject invalid programs, not wrong ones. Semantic errors need a spec: tests, constraints, or a denotation.
- A spurious program can execute to the right answer for the wrong reason (SEMPRE's weak-supervision problem).
