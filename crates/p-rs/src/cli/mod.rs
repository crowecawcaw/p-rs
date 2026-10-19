//! The `prs` and `prs-compat` command line tools.
//!
//! `prs-compat` is [`crate::compat::run`]: upstream's `p` command line, option for option.
//! `prs` ([`run`]) is prs's own command line on the same checker, with what upstream lacks
//! (parallel workers, time tracking) and options named for how prs works.

mod command;
#[cfg(not(target_arch = "wasm32"))]
mod native;

pub use command::run;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{run_threads, NativeHost};
