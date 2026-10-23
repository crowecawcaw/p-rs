//! The [P modeling language](https://p-org.github.io/P/) in Rust.
//!
//! Application code emits events with [`emit`]; tests check them against P specs, in
//! process, with [`test`](mod@test); the `snappea` binary checks P models and production logs.
//!
//! | Feature | Enables |
//! | --- | --- |
//! | `emit` | [`emit`] only: no checker, depends only on `serde` and `serde_json`. For production code. |
//! | `test` (default) | [`test`](mod@test), plus `emit` and the checker library |
//! | `cli` (default) | the `snappea` and `snappea-compat` binaries, and the checker library |
//!
//! Production code that only emits:
//!
//! ```toml
//! [dependencies]
//! snappea = { version = "0.1", default-features = false, features = ["emit"] }
//!
//! [dev-dependencies]
//! snappea = { version = "0.1", default-features = false, features = ["test"] }
//! ```
//!
//! The checker library is [`syntax`] (parser), [`sema`] (type checker), [`checker`]
//! (systematic testing and log checking), [`project`] (`.pproj` files and P sources),
//! [`driver`] (the check and observe commands, over a [`host`]) and [`compat`] (upstream
//! P's command line on top of [`driver`]).
//! See `docs/library.md` in the repository.

#[cfg(feature = "emit")]
pub mod emit;

#[cfg(feature = "test")]
pub mod test;

#[cfg(any(feature = "test", feature = "cli"))]
pub use snappea_syntax as syntax;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod sema;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod checker;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod host;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod project;

#[cfg(any(feature = "test", feature = "cli"))]
pub mod driver;

#[cfg(feature = "cli")]
pub mod compat;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
