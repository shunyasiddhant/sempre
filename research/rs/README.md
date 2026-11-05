# sempre-ir (Rust)

Fast, std-only rewrite of the Formula IR. No legacy Java patterns: byte-cursor parser (no token allocations), operators as a Copy enum (no string hashing in the hot path), small-Vec environments.

## Run
    cargo test --release
    cargo run --release -- 1000000
    cargo run --release -- experiment
    cargo run --release -- tables

## Preliminary microbenchmark (1 vCPU sandbox, 2026-11-05)
Formula: `(call + (number 3) (call * (number 4) (number 5)))`, parse + type-check + execute:
- Rust: 2,259,243 / sec
- Python (`../formula_ir.py`): 68,563 / sec (~33x)

Single-formula microbenchmark, not a system claim.

## Experiment replication (200 tasks)
    none 165/200, parse 158/200, parse+type 131/200, parse+type+exec 131/200, oracle 0/200
Consistent with Python (166/159/136/136/0).

## Tables extension (WikiTableQuestions-style, reinvented)
World = named tables. New typed ops: `field`, `project`, `count`, `sum`, `avg`, `argmax`, `join`, and `select` over tables. Rows are single-row tables inside lambdas, so `field` is uniform.

Demo (employees/departments): count salary>70000 -> 3; sum eng salaries -> 215000; argmax salary -> [Alice]; join on dept -> count 5.

Table-query bench (filter+project+sum, parse+type+execute): 215,431/sec on 1 vCPU. Tests: 11 passing (7 core + 4 table).
