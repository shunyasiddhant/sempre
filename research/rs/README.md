# sempre-ir (Rust)

Fast, std-only rewrite of the Formula IR. No legacy Java patterns: byte-cursor parser (no token allocations), operators as a Copy enum (no string hashing in the hot path), small-Vec environments.

## Run
    cargo test --release
    cargo run --release -- 1000000
    cargo run --release -- experiment

## Preliminary microbenchmark (1 vCPU sandbox, 2026-11-05)
Formula: `(call + (number 3) (call * (number 4) (number 5)))`, parse + type-check + execute, tight loop:
- Rust (this crate, -O3, LTO): 2,259,243 / sec
- Python (`../formula_ir.py`): 68,563 / sec

Ratio ~33x. Single-formula microbenchmark, not a system claim.

## Experiment replication (deterministic shuffle, 200 tasks)
    none                wrong-accepted 165/200
    parse               wrong-accepted 158/200
    parse+type          wrong-accepted 131/200
    parse+type+exec     wrong-accepted 131/200
    oracle (denotation) wrong-accepted   0/200

Consistent with Python (166/159/136/136/0): static layers prune invalid programs; semantic errors require a specification layer.
