//! Model lint policy, shared by the standalone lint command and pre-check gate.
use crate::host::{Color, Exit, Host, Stream};
use crate::project::Sources;
use crate::sema::diag::{render_coded, render_snippet};

/// Print all unsuppressed lint findings. Does not execute models or write artifacts.
pub fn run(host: &mut dyn Host, sources: &Sources) -> Result<usize, Exit> {
    let sm = sources.source_map();
    let findings = snappea_lint::lint(&sm).map_err(|e| {
        let help = crate::project::parse_help(&e.message);
        let text = render_snippet(&sm, Some(e.span), &format!("syntax error: {}", e.message), Some(&help));
        host.write(Stream::Err, &text, None);
        Exit(2)
    })?;
    for f in &findings {
        let help = format!(
            "if this is intended, suppress it with `// snappea-lint: allow {} -- reason` just before the declaration or statement",
            f.rule
        );
        host.write(
            Stream::Err,
            &render_coded(&sm, Some(f.rule), Some(f.span), &f.message, Some(&help)),
            Some(Color::Red),
        );
        host.write(Stream::Err, "\n", None);
    }
    if !findings.is_empty() {
        host.write(
            Stream::Err,
            &format!(
                "Lint failed: {} finding(s). See docs/linting.md for fixes and scoped suppression comments.\n",
                findings.len()
            ),
            None,
        );
    }
    Ok(findings.len())
}
