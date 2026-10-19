//! Diagnostics. Messages reproduce `DefaultTranslationErrorHandler` upstream verbatim so
//! that tooling built around the original compiler keeps working.

use crate::syntax::{SourceMap, Span};

/// One compiler error. `message` does not include the location prefix; see [`Diagnostic::render`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Location shown in the `[file:line:col]` prefix. `None` for errors without a location
    /// (parse errors carry their own prefix in `message`).
    pub span: Option<Span>,
    pub message: String,
}

impl Diagnostic {
    pub fn new(span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic { span: Some(span), message: message.into() }
    }

    pub fn bare(message: impl Into<String>) -> Diagnostic {
        Diagnostic { span: None, message: message.into() }
    }

    /// The message with its `[file:line:col]` prefix, as upstream prints it after `[Error:] `.
    pub fn render(&self, sm: &SourceMap) -> String {
        match self.span {
            Some(sp) => format!("[{}] {}", sm.location(sp), self.message),
            None => self.message.clone(),
        }
    }
}

/// Collects errors. In collecting mode (`continue_on_error`) reporting appends; in strict mode
/// (`--strict-errors`) the first error is returned as `Err` so analysis aborts.
#[derive(Debug, Default)]
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<String>,
    pub continue_on_error: bool,
}

impl Diagnostics {
    pub fn new(continue_on_error: bool) -> Diagnostics {
        Diagnostics { errors: Vec::new(), warnings: Vec::new(), continue_on_error }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// `handler.Diagnostics.Report(...)`: append, or abort when strict.
    pub fn report(&mut self, d: Diagnostic) -> Result<(), Diagnostic> {
        if self.continue_on_error {
            self.errors.push(d);
            Ok(())
        } else {
            Err(d)
        }
    }

    /// Runs one tolerant step: an error thrown inside is recorded and analysis continues
    /// with the next item (`TolerantStep` upstream).
    pub fn tolerant<T>(&mut self, r: Result<T, Diagnostic>) -> Result<Option<T>, Diagnostic> {
        match r {
            Ok(v) => Ok(Some(v)),
            Err(d) if self.continue_on_error => {
                self.errors.push(d);
                Ok(None)
            }
            Err(d) => Err(d),
        }
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    /// Errors sorted the way upstream prints them: by file name, line and column, with
    /// errors lacking a location last; insertion order breaks ties.
    pub fn sorted_errors(&self, sm: &SourceMap) -> Vec<Diagnostic> {
        let mut v: Vec<(usize, Diagnostic)> = self.errors.iter().cloned().enumerate().collect();
        v.sort_by(|(ia, a), (ib, b)| {
            let ka = sort_key(a, sm);
            let kb = sort_key(b, sm);
            ka.cmp(&kb).then(ia.cmp(ib))
        });
        v.into_iter().map(|(_, d)| d).collect()
    }
}

fn sort_key(d: &Diagnostic, sm: &SourceMap) -> (String, i64, i64) {
    match d.span {
        Some(sp) if !sp.is_dummy() => {
            let l = sm.location(sp);
            (l.file.unwrap_or_default(), l.line, l.col)
        }
        _ => ("\u{ffff}".to_string(), 0, 0),
    }
}

/// Convenience type for "thrown" errors.
pub type Fatal = Diagnostic;
