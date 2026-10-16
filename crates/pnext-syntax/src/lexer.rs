//! Hand written lexer mirroring `PLexer.g4`.

use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind};

/// A lexical error, reported with the upstream ANTLR-like message shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub span: Span,
    pub message: String,
}

/// Tokenizes the complete text of one file. Comments and whitespace are skipped.
pub fn lex(file: FileId, text: &str) -> Result<Vec<Token>, LexError> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let n = bytes.len();
    let tok = |kind: TokenKind, start: usize, end: usize| Token {
        kind,
        span: Span::new(file, start as u32, end as u32),
    };
    while i < n {
        let c = bytes[i];
        // whitespace
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' || c == 0x0c {
            i += 1;
            continue;
        }
        // comments
        if c == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            while i < n && bytes[i] != b'\n' && bytes[i] != b'\r' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            loop {
                if i + 1 >= n {
                    // unterminated block comment: ANTLR's non-greedy `.*?` fails at EOF
                    return Err(LexError {
                        span: Span::new(file, start as u32, n as u32),
                        message: "token recognition error at: '/*'".to_string(),
                    });
                }
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        let start = i;
        // identifiers / keywords
        if c.is_ascii_alphabetic() || c == b'_' {
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &text[start..i];
            // 'init-condition' keyword contains a dash
            if word == "init" && text[i..].starts_with("-condition") {
                let end = i + "-condition".len();
                // must not be followed by an identifier character
                if end >= n || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                    tokens.push(tok(TokenKind::InitCondition, start, end));
                    i = end;
                    continue;
                }
            }
            let kind = TokenKind::keyword(word).unwrap_or(TokenKind::Iden);
            tokens.push(tok(kind, start, i));
            continue;
        }
        // integer literals
        if c.is_ascii_digit() {
            while i < n && bytes[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push(tok(TokenKind::IntLiteral, start, i));
            continue;
        }
        // string literals
        if c == b'"' {
            i += 1;
            loop {
                if i >= n {
                    return Err(LexError {
                        span: Span::new(file, start as u32, n as u32),
                        message: format!("token recognition error at: '{}'", &text[start..n]),
                    });
                }
                let b = bytes[i];
                if b == b'\\' {
                    i += 2;
                    continue;
                }
                if b == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            tokens.push(tok(TokenKind::StringLiteral, start, i));
            continue;
        }
        // symbols; longest match first
        let rest = &text[i..];
        let (kind, len) = if rest.starts_with("<==>") {
            (TokenKind::LIff, 4)
        } else if rest.starts_with("==>") {
            (TokenKind::LThen, 3)
        } else if rest.starts_with("$$") {
            (TokenKind::FairNondet, 2)
        } else if rest.starts_with("&&") {
            (TokenKind::LAnd, 2)
        } else if rest.starts_with("||") {
            (TokenKind::LOr, 2)
        } else if rest.starts_with("==") {
            (TokenKind::Eq, 2)
        } else if rest.starts_with("!=") {
            (TokenKind::Ne, 2)
        } else if rest.starts_with("<=") {
            (TokenKind::Le, 2)
        } else if rest.starts_with(">=") {
            (TokenKind::Ge, 2)
        } else if rest.starts_with("->") {
            (TokenKind::RArrow, 2)
        } else if rest.starts_with("+=") {
            (TokenKind::Insert, 2)
        } else if rest.starts_with("-=") {
            (TokenKind::Remove, 2)
        } else {
            let k = match c {
                b'$' => TokenKind::Nondet,
                b'!' => TokenKind::LNot,
                b'<' => TokenKind::Lt,
                b'>' => TokenKind::Gt,
                b'=' => TokenKind::Assign,
                b'+' => TokenKind::Add,
                b'-' => TokenKind::Sub,
                b'*' => TokenKind::Mul,
                b'/' => TokenKind::Div,
                b'%' => TokenKind::Mod,
                b'{' => TokenKind::LBrace,
                b'}' => TokenKind::RBrace,
                b'[' => TokenKind::LBrack,
                b']' => TokenKind::RBrack,
                b'(' => TokenKind::LParen,
                b')' => TokenKind::RParen,
                b';' => TokenKind::Semi,
                b',' => TokenKind::Comma,
                b'.' => TokenKind::Dot,
                b':' => TokenKind::Colon,
                _ => {
                    let ch = text[i..].chars().next().unwrap();
                    return Err(LexError {
                        span: Span::new(file, start as u32, (start + ch.len_utf8()) as u32),
                        message: format!("token recognition error at: '{}'", ch),
                    });
                }
            };
            (k, 1)
        };
        tokens.push(tok(kind, start, start + len));
        i += len;
    }
    tokens.push(tok(TokenKind::Eof, n, n));
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_keywords_and_symbols() {
        let toks = lex(FileId(0), "machine M { var x: int; } // c\n/* b */ init-condition $$ <==>").unwrap();
        let kinds: Vec<_> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Machine,
                TokenKind::Iden,
                TokenKind::LBrace,
                TokenKind::Var,
                TokenKind::Iden,
                TokenKind::Colon,
                TokenKind::Int,
                TokenKind::Semi,
                TokenKind::RBrace,
                TokenKind::InitCondition,
                TokenKind::FairNondet,
                TokenKind::LIff,
                TokenKind::Eof
            ]
        );
    }
}
