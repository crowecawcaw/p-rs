use crate::{visit, Context, Finding, Rule};
use snappea_syntax::{ast::*, lexer, FileId, Span};

pub fn apply(ctx: &Context<'_>, rules: &[Box<dyn Rule>], findings: &mut Vec<Finding>) {
    let mut scopes = Vec::new();
    for d in ctx.declarations() {
        match d {
            TopDecl::Machine(m) => {
                scopes.push(m.span);
                for s in &m.states {
                    scopes.push(s.span);
                    for i in &s.items {
                        scopes.push(i.span);
                    }
                }
                for b in visit::bodies(m) {
                    visit::stmts(&b.stmts, &mut |s| scopes.push(s.span), &mut |_| {});
                }
                for f in &m.funs {
                    scopes.push(f.span);
                }
            }
            TopDecl::Fun(f) => {
                scopes.push(f.span);
                if let Some(b) = &f.body {
                    visit::stmts(&b.stmts, &mut |s| scopes.push(s.span), &mut |_| {});
                }
            }
            _ => {}
        }
    }
    let mut allowed = Vec::new();
    let mut invalid = Vec::new();
    for (i, file) in ctx.sources.files().iter().enumerate() {
        let id = FileId(i as u32);
        let Ok(tokens) = lexer::lex(id, &file.text) else {
            continue;
        };
        // Inspect only lexer gaps: strings and other tokens cannot forge directives.
        let mut end = 0;
        for token in &tokens {
            if token.span.start > end {
                comments(&file.text, end as usize, token.span.start as usize, &mut |start, text| {
                    let Some(text) = text.trim().strip_prefix("snappea-lint:") else {
                        return;
                    };
                    let span = Span::new(id, start as u32, (start + text.len()) as u32);
                    let bad = |message: &str| Finding::new("invalid-suppression", span, message);
                    let Some(ids) = text.trim().strip_prefix("allow ") else {
                        invalid.push(bad("expected '// snappea-lint: allow rule-id[, rule-id] -- reason'"));
                        return;
                    };
                    let ids = ids.split_once("--").map_or(ids, |(ids, _)| ids).trim();
                    let ids: Vec<_> = ids.split(',').map(str::trim).collect();
                    if ids.iter().any(|id| !rules.iter().any(|r| r.id() == *id)) {
                        invalid.push(bad("unknown or empty lint rule ID in suppression"));
                        return;
                    }
                    // Suppression must be a standalone comment before the node.
                    let line_start = file.text[..start].rfind('\n').map_or(0, |n| n + 1);
                    if !file.text[line_start..start].trim().is_empty() {
                        invalid
                            .push(bad("put the lint suppression on its own line before the declaration or statement"));
                        return;
                    }
                    let target =
                        scopes.iter().filter(|s| s.file == id && s.start == token.span.start).max_by_key(|s| s.end);
                    if let Some(target) = target {
                        for rule in ids {
                            allowed.push((*target, rule.to_string()));
                        }
                    } else {
                        invalid.push(bad(
                            "lint suppression must precede a machine, state, handler, function, or statement",
                        ));
                    }
                });
            }
            end = token.span.end;
        }
    }
    findings.retain(|f| {
        !allowed
            .iter()
            .any(|(s, id)| id == f.rule && s.file == f.span.file && s.start <= f.span.start && s.end >= f.span.end)
    });
    findings.extend(invalid);
}

fn comments(text: &str, start: usize, end: usize, f: &mut impl FnMut(usize, &str)) {
    let bytes = text.as_bytes();
    let mut i = start;
    while i + 1 < end {
        if &bytes[i..i + 2] == b"//" {
            let a = i;
            i += 2;
            let content = i;
            while i < end && !matches!(bytes[i], b'\n' | b'\r') {
                i += 1;
            }
            f(a, &text[content..i]);
        } else if &bytes[i..i + 2] == b"/*" {
            i += 2;
            while i + 1 < end && &bytes[i..i + 2] != b"*/" {
                i += 1;
            }
            i += 2;
        } else {
            i += 1;
        }
    }
}
