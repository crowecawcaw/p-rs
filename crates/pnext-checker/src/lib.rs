//! Systematic testing of P programs.
//!
//! The checker interprets a [`pnext_sema::Program`] directly (no code generation)
//! under a controlled, deterministic scheduler, reproducing the behaviour of
//! the upstream P checker (`PChecker`) closely enough that the same bugs are
//! found and the same trace lines are produced.
//!
//! The crate is `no_std`-friendly in spirit: it uses no threads, no files and
//! no process APIs, so it can be compiled to `wasm32`.

pub mod bytecode;
pub mod compile;
pub mod coverage;
pub mod engine;
pub mod exhaustive;
pub mod feedback;
pub mod foreign;
pub mod interp;
pub mod javafmt;
pub mod jsonlog;
pub mod observe;
pub mod random;
pub mod rl;
pub mod runtime;
pub mod statehash;
pub mod strategy;
pub mod time;
pub mod timeline;
pub mod value;
pub mod xmllog;
