//! The [P modeling language](https://p-org.github.io/P/) in Rust.
//!
//! Application code emits events with [`emit`]; tests check them against P specs, in
//! process, with [`test`](mod@test); the `prs` binary checks P models and production logs.
//!
//! | Feature | Enables |
//! | --- | --- |
//! | `emit` | [`emit`] only: no checker, depends only on `serde` and `serde_json`. For production code. |
//! | `test` (default) | [`test`](mod@test), plus `emit` and the checker library |
//! | `cli` (default) | the `prs` and `prs-compat` binaries, and the checker library |
//!
//! Production code that only emits:
//!
//! ```toml
//! [dependencies]
//! p-rs = { version = "0.1", default-features = false, features = ["emit"] }
//!
//! [dev-dependencies]
//! p-rs = { version = "0.1", default-features = false, features = ["test"] }
//! ```
//!
//! The checker library is [`syntax`] (parser), [`sema`] (type checker), [`checker`]
//! (systematic testing and log checking) and [`compat`] (upstream P's command line).
//! See `docs/library.md` in the repository.

#[cfg(feature = "emit")]
pub mod emit;

#[cfg(feature = "test")]
pub mod test;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod syntax;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod sema;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod checker;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod compat;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
