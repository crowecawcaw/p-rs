//! Front end for the P modeling language: lexer, parser and AST.
//!
//! The crate is platform independent (no I/O) so it can be used from a CLI, a library
//! consumer or a WebAssembly build alike.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use parser::{parse, ParseError};
pub use span::{FileId, Location, SourceFile, SourceMap, Span};
