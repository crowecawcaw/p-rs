//! The `prs` and `prs-compat` command line tools.
//!
//! `prs` ([`run`]) is prs's own command line on the commands of [`crate::driver`], with what
//! upstream lacks (parallel workers, time tracking) and options named for how prs works.
//! `prs-compat` is upstream's `p` command line, option for option, on the same commands
//! (`prs::compat`); it shares this module's [`NativeHost`].

mod command;
#[cfg(not(target_arch = "wasm32"))]
mod native;

pub use command::run;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{run_threads, NativeHost};
