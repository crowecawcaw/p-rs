//! The `prs` and `prs-compat` command line tools.
//!
//! `prs` ([`run`]) is prs's own command line and `prs-compat` upstream's `p`, option for
//! option. Both run the commands of [`crate::driver`]; `prs` adds
//! what upstream lacks (parallel workers, time tracking) and options named for how prs works.

mod command;
#[cfg(not(target_arch = "wasm32"))]
mod native;
mod observe;

pub use command::run;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{run_threads, NativeHost};
