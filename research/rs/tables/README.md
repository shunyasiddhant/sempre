# sempre-tables (Rust)

Compositional table semantics, reinvented from SEMPRE / Lambda DCS (Pasupat & Liang, ACL 2015): a unary denotation is a set of rows, represented as a bitset (`Vec<u64>`). Filter/intersect/union are word-level operations, count is popcount, superlatives (argmax/argmin) are scan + bit-keep.

## Query language
    (project (argmax (all) Population) Capital)   ; capital of most populous state
    (count (filter (all) Population gt (number 10)))
    (project (filter (all) State eq (string California)) Capital)
    (sum (all) Population)  (intersect <q> <q>)  (union <q> <q>)

Column names are resolved against the table header at parse time; unknown columns are rejected before execution — the same propose/constrain/execute discipline as the Formula IR.

## Run
    cargo test --release
    cargo run --release
    cargo run --release -- bench

## Preliminary benchmark (1 vCPU sandbox)
100,000-row synthetic table, `(count (filter (all) Val gt (number 500)))`:
2,296 eval/sec (~230M row-checks/sec). Filter is a scan by necessity; set operations on its output are bitset ops.
