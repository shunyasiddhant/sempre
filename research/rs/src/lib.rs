//! sempre-ir: typed IR for SEMPRE formulas.
//! Modules: core (AST+parser), types (type checker), exec (executor+World), verify (candidate verifier).
pub mod core;
pub mod types;
pub mod exec;
pub mod verify;
#[cfg(test)]
mod tests;
pub use core::{parse, Expr, Op, Parser};
pub use types::{type_of, Type};
pub use exec::{execute, execute_w, Value, World};
pub use verify::{first_verified, verify, Verdict};
