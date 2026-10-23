//! Model lint policy, shared by the standalone lint command and pre-check gate.
use crate::host::{Color, Exit, Host, Stream};
use crate::project::Sources;

/// Print all unsuppressed lint findings. Does not execute models or write artifacts.
pub fn run(host: &mut dyn Host, sources: &Sources) -> Result<usize, Exit> {
    let sm = sources.source_map();
    let findings = snappea_lint::lint(&sm).map_err(|e| {
        host.write(Stream::Err, &format!("{}: {}\n", sm.location(e.span), e.message), None);
        Exit(2)
    })?;
    for f in &findings {
        host.write(
            Stream::Err,
            &format!("{}: error[{}]: {}\n", sm.location(f.span), f.rule, f.message),
            Some(Color::Red),
        );
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
