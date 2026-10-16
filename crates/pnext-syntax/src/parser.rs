//! Recursive descent parser for P, equivalent to `PParser.g4`.

use crate::ast::*;
use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind};
use crate::token::TokenKind as T;

/// A syntax error. `message` follows the ANTLR style used upstream
/// (`mismatched input 'x' expecting 'y'`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

type PResult<T> = Result<T, ParseError>;

/// Parses one file into a [`Program`].
pub fn parse(file: FileId, text: &str) -> PResult<Program> {
    let tokens = crate::lexer::lex(file, text).map_err(|e| ParseError { span: e.span, message: e.message })?;
    let mut p = Parser { text, tokens, pos: 0 };
    p.program()
}

struct Parser<'a> {
    text: &'a str,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    // ----- token helpers -----

    fn peek(&self) -> Token {
        self.tokens[self.pos]
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens[self.pos].kind
    }

    fn peek_at(&self, n: usize) -> TokenKind {
        let i = (self.pos + n).min(self.tokens.len() - 1);
        self.tokens[i].kind
    }

    fn prev_span(&self) -> Span {
        self.tokens[self.pos.saturating_sub(1)].span
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at(&self, k: TokenKind) -> bool {
        self.peek_kind() == k
    }

    fn eat(&mut self, k: TokenKind) -> bool {
        if self.at(k) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn tok_text(&self, t: Token) -> &'a str {
        &self.text[t.span.start as usize..t.span.end as usize]
    }

    fn err_at<R>(&self, tok: Token, message: String) -> PResult<R> {
        Err(ParseError { span: tok.span, message })
    }

    fn offending(&self, tok: Token) -> String {
        if tok.kind == T::Eof {
            "<EOF>".to_string()
        } else {
            self.tok_text(tok).to_string()
        }
    }

    fn expect(&mut self, k: TokenKind) -> PResult<Token> {
        if self.at(k) {
            Ok(self.bump())
        } else {
            let t = self.peek();
            self.err_at(t, format!("mismatched input '{}' expecting {}", self.offending(t), k.display()))
        }
    }

    fn expect_one_of(&mut self, kinds: &[TokenKind]) -> PResult<Token> {
        if kinds.contains(&self.peek_kind()) {
            Ok(self.bump())
        } else {
            let t = self.peek();
            let expected: Vec<&str> = kinds.iter().map(|k| k.display()).collect();
            self.err_at(t, format!("mismatched input '{}' expecting {{{}}}", self.offending(t), expected.join(", ")))
        }
    }

    fn no_viable<R>(&self) -> PResult<R> {
        let t = self.peek();
        self.err_at(t, format!("no viable alternative at input '{}'", self.offending(t)))
    }

    fn ident(&mut self) -> PResult<Ident> {
        let t = self.expect(T::Iden)?;
        Ok(Ident { name: self.tok_text(t).to_string(), span: t.span })
    }

    fn ident_list(&mut self) -> PResult<Vec<Ident>> {
        let mut v = vec![self.ident()?];
        while self.eat(T::Comma) {
            v.push(self.ident()?);
        }
        Ok(v)
    }

    // ----- program -----

    fn program(&mut self) -> PResult<Program> {
        let mut decls = Vec::new();
        while !self.at(T::Eof) {
            decls.push(self.top_decl()?);
        }
        Ok(Program { decls })
    }

    fn top_decl(&mut self) -> PResult<TopDecl> {
        let start = self.peek().span;
        match self.peek_kind() {
            T::Type => {
                self.bump();
                let name = self.ident()?;
                let ty = if self.eat(T::Assign) { Some(self.ty()?) } else { None };
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::TypeDef { name, ty, span: start.to(end) })
            }
            T::Enum => {
                self.bump();
                let name = self.ident()?;
                self.expect(T::LBrace)?;
                let mut elems = Vec::new();
                let numbered = self.peek_at(1) == T::Assign;
                loop {
                    let n = self.ident()?;
                    let value = if numbered {
                        self.expect(T::Assign)?;
                        let v = self.expect(T::IntLiteral)?;
                        Some((self.tok_text(v).to_string(), v.span))
                    } else {
                        None
                    };
                    elems.push(EnumElem { name: n, value });
                    if !self.eat(T::Comma) {
                        break;
                    }
                }
                let end = self.expect(T::RBrace)?.span;
                Ok(TopDecl::Enum { name, elems, span: start.to(end) })
            }
            T::Event => {
                self.bump();
                let name = self.ident()?;
                let payload = if self.eat(T::Colon) { Some(self.ty()?) } else { None };
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::Event { name, payload, span: start.to(end) })
            }
            T::EventSet => {
                self.bump();
                let name = self.ident()?;
                self.expect(T::Assign)?;
                self.expect(T::LBrace)?;
                let events = self.non_default_event_list()?;
                self.expect(T::RBrace)?;
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::EventSet { name, events, span: start.to(end) })
            }
            T::Interface => {
                self.bump();
                let name = self.ident()?;
                self.expect(T::LParen)?;
                let payload = if self.at(T::RParen) { None } else { Some(self.ty()?) };
                self.expect(T::RParen)?;
                let receives = if self.eat(T::Receives) {
                    if self.at(T::Semi) {
                        Some(Vec::new())
                    } else {
                        Some(self.non_default_event_list()?)
                    }
                } else {
                    None
                };
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::Interface { name, payload, receives, span: start.to(end) })
            }
            T::Machine => self.machine_decl(MachineKind::Impl).map(TopDecl::Machine),
            T::Spec => self.machine_decl(MachineKind::Spec).map(TopDecl::Machine),
            T::Scenario => self.machine_decl(MachineKind::Scenario).map(TopDecl::Machine),
            T::Fun => self.fun_decl().map(TopDecl::Fun),
            T::Pure => {
                self.bump();
                let name = self.ident()?;
                self.expect(T::LParen)?;
                let params = if self.at(T::RParen) { Vec::new() } else { self.fun_param_list()? };
                self.expect(T::RParen)?;
                self.expect(T::Colon)?;
                let ret = self.ty()?;
                let body = if self.eat(T::Assign) { Some(self.expr()?) } else { None };
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::Pure(PureDecl { name, params, ret, body, span: start.to(end) }))
            }
            T::Module => {
                self.bump();
                let name = self.ident()?;
                self.expect(T::Assign)?;
                let expr = self.mod_expr()?;
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::Module { name, expr, span: start.to(end) })
            }
            T::Test => self.test_decl(),
            T::Implementation => {
                self.bump();
                let name = self.ident()?;
                let main = if self.eat(T::LBrack) {
                    self.expect(T::Main)?;
                    self.expect(T::Assign)?;
                    let m = self.ident()?;
                    self.expect(T::RBrack)?;
                    Some(m)
                } else {
                    None
                };
                self.expect(T::Colon)?;
                let expr = self.mod_expr()?;
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::Implementation { name, main, expr, span: start.to(end) })
            }
            T::Param => {
                self.bump();
                let names = self.ident_list()?;
                self.expect(T::Colon)?;
                let ty = self.ty()?;
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::GlobalParam { names, ty, span: start.to(end) })
            }
            T::Invariant => self.invariant_decl().map(TopDecl::Invariant),
            T::Lemma | T::Theorem => {
                let is_theorem = self.bump().kind == T::Theorem;
                let name = self.ident()?;
                self.expect(T::LBrace)?;
                let mut invariants = Vec::new();
                while self.at(T::Invariant) {
                    invariants.push(self.invariant_decl()?);
                }
                let end = self.expect(T::RBrace)?.span;
                Ok(TopDecl::InvariantGroup { name, is_theorem, invariants, span: start.to(end) })
            }
            T::Axiom => {
                self.bump();
                let body = self.expr()?;
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::Axiom { body, span: start.to(end) })
            }
            T::InitCondition => {
                self.bump();
                let body = self.expr()?;
                let end = self.expect(T::Semi)?.span;
                Ok(TopDecl::AssumeOnStart { body, span: start.to(end) })
            }
            T::Proof => {
                self.bump();
                let name = if self.at(T::Iden) { Some(self.ident()?) } else { None };
                self.expect(T::LBrace)?;
                let mut items = Vec::new();
                while self.at(T::Prove) {
                    items.push(self.proof_item()?);
                }
                let end = self.expect(T::RBrace)?.span;
                Ok(TopDecl::ProofBlock { name, items, span: start.to(end) })
            }
            _ => self.no_viable(),
        }
    }

    fn invariant_decl(&mut self) -> PResult<InvariantDecl> {
        let start = self.expect(T::Invariant)?.span;
        let name = self.ident()?;
        self.expect(T::Colon)?;
        let body = self.expr()?;
        let end = self.expect(T::Semi)?.span;
        Ok(InvariantDecl { name, body, span: start.to(end) })
    }

    fn proof_item(&mut self) -> PResult<ProofItem> {
        let start = self.expect(T::Prove)?.span;
        let mut item = ProofItem {
            targets: Vec::new(),
            targets_all: false,
            targets_default: false,
            premises: Vec::new(),
            premises_all: false,
            excludes: Vec::new(),
            span: start,
        };
        if self.eat(T::Mul) {
            item.targets_all = true;
        } else if self.eat(T::Default) {
            item.targets_default = true;
        } else {
            item.targets.push(self.expr()?);
            while self.eat(T::Comma) {
                item.targets.push(self.expr()?);
            }
        }
        if self.eat(T::Using) {
            if self.eat(T::Mul) {
                item.premises_all = true;
            } else {
                item.premises.push(self.expr()?);
                while self.eat(T::Comma) {
                    item.premises.push(self.expr()?);
                }
            }
        }
        if self.eat(T::Except) {
            item.excludes.push(self.expr()?);
            while self.eat(T::Comma) {
                item.excludes.push(self.expr()?);
            }
        }
        let end = self.expect(T::Semi)?.span;
        item.span = start.to(end);
        Ok(item)
    }

    fn test_decl(&mut self) -> PResult<TopDecl> {
        let start = self.expect(T::Test)?.span;
        // refinement test: TEST name [main=M] : modExpr REFINES modExpr ;
        // safety test: TEST (PARAM param)? (ASSUME expr)? twise? name [main=M] : modExpr ;
        let mut params = None;
        let mut assume = None;
        let mut twise = None;
        if self.at(T::Param) {
            let pstart = self.bump().span;
            self.expect(T::LParen)?;
            let mut bindings = Vec::new();
            loop {
                let bstart = self.peek().span;
                let name = self.ident()?;
                self.expect(T::In)?;
                let values = self.seq_literal()?;
                bindings.push(ParamBinding { name, values, span: bstart.to(self.prev_span()) });
                if !self.eat(T::Comma) {
                    break;
                }
            }
            let pend = self.expect(T::RParen)?.span;
            params = Some((bindings, pstart.to(pend)));
        }
        if self.eat(T::Assume) {
            assume = Some(self.expr()?);
        }
        if self.at(T::Pairwise) {
            twise = Some(Twise::Pairwise(self.bump().span));
        } else if self.at(T::LParen) && self.peek_at(1) == T::IntLiteral && self.peek_at(2) == T::Wise {
            let s = self.bump().span;
            let n = self.bump();
            self.bump();
            let e = self.expect(T::RParen)?.span;
            twise = Some(Twise::N(self.tok_text(n).to_string(), s.to(e)));
        }
        let name = self.ident()?;
        self.expect(T::LBrack)?;
        self.expect(T::Main)?;
        self.expect(T::Assign)?;
        let main = self.ident()?;
        self.expect(T::RBrack)?;
        self.expect(T::Colon)?;
        let expr = self.mod_expr()?;
        if self.at(T::Refines) {
            if params.is_some() || assume.is_some() || twise.is_some() {
                return self.no_viable();
            }
            self.bump();
            let rhs = self.mod_expr()?;
            let end = self.expect(T::Semi)?.span;
            return Ok(TopDecl::RefinementTest { name, main, lhs: expr, rhs, span: start.to(end) });
        }
        let end = self.expect(T::Semi)?.span;
        Ok(TopDecl::SafetyTest(SafetyTestDecl { name, main, expr, params, assume, twise, span: start.to(end) }))
    }

    fn seq_literal(&mut self) -> PResult<Vec<SeqPrimitive>> {
        self.expect(T::LBrack)?;
        let mut v = Vec::new();
        loop {
            let t = self.peek();
            match t.kind {
                T::BoolLiteral => {
                    self.bump();
                    v.push(SeqPrimitive::Bool(self.tok_text(t) == "true", t.span));
                }
                T::IntLiteral => {
                    self.bump();
                    v.push(SeqPrimitive::Int { negative: false, text: self.tok_text(t).to_string(), span: t.span });
                }
                T::Sub => {
                    self.bump();
                    let n = self.expect(T::IntLiteral)?;
                    v.push(SeqPrimitive::Int { negative: true, text: self.tok_text(n).to_string(), span: t.span.to(n.span) });
                }
                _ => return self.no_viable(),
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.expect(T::RBrack)?;
        Ok(v)
    }

    // ----- module expressions -----

    fn mod_expr(&mut self) -> PResult<ModExpr> {
        let start = self.peek().span;
        let kind = match self.peek_kind() {
            T::LParen => {
                self.bump();
                let inner = self.mod_expr()?;
                self.expect(T::RParen)?;
                ModExprKind::Paren(Box::new(inner))
            }
            T::LBrace => {
                self.bump();
                let mut binds = Vec::new();
                loop {
                    let bstart = self.peek().span;
                    let machine = self.ident()?;
                    let interface = if self.eat(T::RArrow) { Some(self.ident()?) } else { None };
                    binds.push(BindExpr { machine, interface, span: bstart.to(self.prev_span()) });
                    if !self.eat(T::Comma) {
                        break;
                    }
                }
                self.expect(T::RBrace)?;
                ModExprKind::Bind(binds)
            }
            T::Iden => ModExprKind::Named(self.ident()?),
            T::Compose | T::Union => {
                let is_compose = self.bump().kind == T::Compose;
                let mut mods = vec![self.mod_expr()?];
                self.expect(T::Comma)?;
                mods.push(self.mod_expr()?);
                while self.eat(T::Comma) {
                    mods.push(self.mod_expr()?);
                }
                if is_compose {
                    ModExprKind::Compose(mods)
                } else {
                    ModExprKind::Union(mods)
                }
            }
            T::HideE => {
                self.bump();
                let events = self.non_default_event_list()?;
                self.expect(T::In)?;
                let module = Box::new(self.mod_expr()?);
                ModExprKind::HideEvents { events, module }
            }
            T::HideI => {
                self.bump();
                let interfaces = self.ident_list()?;
                self.expect(T::In)?;
                let module = Box::new(self.mod_expr()?);
                ModExprKind::HideInterfaces { interfaces, module }
            }
            T::Assert => {
                self.bump();
                let monitors = self.ident_list()?;
                self.expect(T::In)?;
                let module = Box::new(self.mod_expr()?);
                ModExprKind::Assert { monitors, module }
            }
            T::Rename => {
                self.bump();
                let old = self.ident()?;
                self.expect(T::To)?;
                let new = self.ident()?;
                self.expect(T::In)?;
                let module = Box::new(self.mod_expr()?);
                ModExprKind::Rename { old, new, module }
            }
            T::Main => {
                self.bump();
                let main = self.ident()?;
                self.expect(T::In)?;
                let module = Box::new(self.mod_expr()?);
                ModExprKind::Main { main, module }
            }
            _ => return self.no_viable(),
        };
        Ok(ModExpr { kind, span: start.to(self.prev_span()) })
    }

    // ----- machines -----

    fn machine_decl(&mut self, kind: MachineKind) -> PResult<MachineDecl> {
        let start = self.bump().span;
        let name = self.ident()?;
        let mut receives = Vec::new();
        let mut sends = Vec::new();
        let mut observes = Vec::new();
        match kind {
            MachineKind::Impl => {
                while self.at(T::Receives) || self.at(T::Sends) {
                    let s = self.peek().span;
                    let is_recv = self.bump().kind == T::Receives;
                    let events = if self.at(T::Semi) { Vec::new() } else { self.non_default_event_list()? };
                    let e = self.expect(T::Semi)?.span;
                    if is_recv {
                        receives.push((events, s.to(e)));
                    } else {
                        sends.push((events, s.to(e)));
                    }
                }
            }
            MachineKind::Spec | MachineKind::Scenario => {
                self.expect(T::Observes)?;
                observes = self.non_default_event_list()?;
            }
        }
        self.expect(T::LBrace)?;
        let mut vars = Vec::new();
        let mut funs = Vec::new();
        let mut states = Vec::new();
        loop {
            match self.peek_kind() {
                T::Var => vars.push(self.var_decl()?),
                T::Fun => funs.push(self.fun_decl()?),
                T::Start | T::Hot | T::Cold | T::State => states.push(self.state_decl()?),
                T::RBrace => break,
                _ => return self.no_viable(),
            }
        }
        let end = self.expect(T::RBrace)?.span;
        Ok(MachineDecl { kind, name, receives, sends, observes, vars, funs, states, span: start.to(end) })
    }

    fn var_decl(&mut self) -> PResult<VarDecl> {
        let start = self.expect(T::Var)?.span;
        let names = self.ident_list()?;
        self.expect(T::Colon)?;
        let ty = self.ty()?;
        let end = self.expect(T::Semi)?.span;
        Ok(VarDecl { names, ty, span: start.to(end) })
    }

    fn fun_param(&mut self) -> PResult<Param> {
        let start = self.peek().span;
        let name = self.ident()?;
        self.expect(T::Colon)?;
        let ty = self.ty()?;
        Ok(Param { name, ty, span: start.to(self.prev_span()) })
    }

    fn fun_param_list(&mut self) -> PResult<Vec<Param>> {
        let mut v = vec![self.fun_param()?];
        while self.eat(T::Comma) {
            v.push(self.fun_param()?);
        }
        Ok(v)
    }

    fn fun_decl(&mut self) -> PResult<FunDecl> {
        let start = self.expect(T::Fun)?.span;
        let name = self.ident()?;
        self.expect(T::LParen)?;
        let params = if self.at(T::RParen) { Vec::new() } else { self.fun_param_list()? };
        self.expect(T::RParen)?;
        let mut decl = FunDecl {
            name,
            params,
            ret: None,
            body: None,
            creates: Vec::new(),
            return_param: None,
            requires: Vec::new(),
            ensures: Vec::new(),
            span: start,
        };
        if self.at(T::Return) {
            // foreign with verifier contract: RETURN (param) ; (REQUIRES e;)* (ENSURES e;)*
            self.bump();
            self.expect(T::LParen)?;
            decl.return_param = Some(self.fun_param()?);
            self.expect(T::RParen)?;
            self.expect(T::Semi)?;
            self.requires_ensures(&mut decl)?;
            decl.span = start.to(self.prev_span());
            return Ok(decl);
        }
        if self.eat(T::Colon) {
            decl.ret = Some(self.ty()?);
        }
        if self.at(T::LBrace) {
            decl.body = Some(self.fun_body()?);
            decl.span = start.to(self.prev_span());
            return Ok(decl);
        }
        if self.eat(T::Creates) {
            decl.creates.push(self.ident()?);
        }
        if self.at(T::Semi) {
            self.bump();
            decl.span = start.to(self.prev_span());
            return Ok(decl);
        }
        if decl.ret.is_none() && decl.creates.is_empty() && (self.at(T::Requires) || self.at(T::Ensures)) {
            self.requires_ensures(&mut decl)?;
            decl.span = start.to(self.prev_span());
            return Ok(decl);
        }
        let t = self.peek();
        self.err_at(t, format!("mismatched input '{}' expecting {{';', '{{', 'creates'}}", self.offending(t)))
    }

    fn requires_ensures(&mut self, decl: &mut FunDecl) -> PResult<()> {
        while self.eat(T::Requires) {
            decl.requires.push(self.expr()?);
            self.expect(T::Semi)?;
        }
        while self.eat(T::Ensures) {
            decl.ensures.push(self.expr()?);
            self.expect(T::Semi)?;
        }
        Ok(())
    }

    fn fun_body(&mut self) -> PResult<FunBody> {
        let start = self.expect(T::LBrace)?.span;
        let mut vars = Vec::new();
        while self.at(T::Var) {
            vars.push(self.var_decl()?);
        }
        let mut stmts = Vec::new();
        while !self.at(T::RBrace) {
            if self.at(T::Eof) {
                return self.expect(T::RBrace).map(|_| unreachable!());
            }
            stmts.push(self.stmt()?);
        }
        let end = self.expect(T::RBrace)?.span;
        Ok(FunBody { vars, stmts, span: start.to(end) })
    }

    fn anon_handler(&mut self, allow_param: bool) -> PResult<AnonHandler> {
        let start = self.peek().span;
        let param = if allow_param && self.at(T::LParen) {
            self.bump();
            let p = self.fun_param()?;
            self.expect(T::RParen)?;
            Some(p)
        } else {
            None
        };
        let body = self.fun_body()?;
        Ok(AnonHandler { param, body, span: start.to(self.prev_span()) })
    }

    fn state_decl(&mut self) -> PResult<StateDecl> {
        let start = self.peek().span;
        let is_start = self.eat(T::Start);
        let temperature = if self.eat(T::Hot) {
            Temperature::Hot
        } else if self.eat(T::Cold) {
            Temperature::Cold
        } else {
            Temperature::Warm
        };
        self.expect(T::State)?;
        let name = self.ident()?;
        self.expect(T::LBrace)?;
        let mut items = Vec::new();
        while !self.at(T::RBrace) {
            items.push(self.state_item()?);
        }
        let end = self.expect(T::RBrace)?.span;
        Ok(StateDecl { name, is_start, temperature, items, span: start.to(end) })
    }

    fn handler_or_named(&mut self, allow_param: bool) -> PResult<Handler> {
        if self.at(T::Iden) {
            let name = self.ident()?;
            self.expect(T::Semi)?;
            Ok(Handler::Named(name))
        } else {
            Ok(Handler::Anon(self.anon_handler(allow_param)?))
        }
    }

    fn state_item(&mut self) -> PResult<StateItem> {
        let start = self.peek().span;
        let kind = match self.peek_kind() {
            T::Entry => {
                self.bump();
                StateItemKind::Entry(self.handler_or_named(true)?)
            }
            T::Exit => {
                self.bump();
                StateItemKind::Exit(self.handler_or_named(false)?)
            }
            T::Defer => {
                self.bump();
                let events = self.non_default_event_list()?;
                self.expect(T::Semi)?;
                StateItemKind::Defer(events)
            }
            T::Ignore => {
                self.bump();
                let events = self.non_default_event_list()?;
                self.expect(T::Semi)?;
                StateItemKind::Ignore(events)
            }
            T::On => {
                self.bump();
                let events = self.event_list()?;
                if self.eat(T::Do) {
                    let handler = self.handler_or_named(true)?;
                    StateItemKind::OnDo { events, handler }
                } else {
                    self.expect(T::Goto)?;
                    let target = self.ident()?;
                    let with = if self.eat(T::With) {
                        Some(self.handler_or_named(true)?)
                    } else {
                        self.expect(T::Semi)?;
                        None
                    };
                    StateItemKind::OnGoto { events, target, with }
                }
            }
            _ => return self.no_viable(),
        };
        Ok(StateItem { kind, span: start.to(self.prev_span()) })
    }

    fn non_default_event(&mut self) -> PResult<Ident> {
        let t = self.peek();
        match t.kind {
            T::Halt => {
                self.bump();
                Ok(Ident { name: "halt".to_string(), span: t.span })
            }
            T::Iden => self.ident(),
            _ => self.expect_one_of(&[T::Halt, T::Iden]).map(|_| unreachable!()),
        }
    }

    fn non_default_event_list(&mut self) -> PResult<Vec<Ident>> {
        let mut v = vec![self.non_default_event()?];
        while self.eat(T::Comma) {
            v.push(self.non_default_event()?);
        }
        Ok(v)
    }

    fn event_id(&mut self) -> PResult<Ident> {
        let t = self.peek();
        match t.kind {
            T::NullLiteral => {
                self.bump();
                Ok(Ident { name: "null".to_string(), span: t.span })
            }
            T::Halt => {
                self.bump();
                Ok(Ident { name: "halt".to_string(), span: t.span })
            }
            T::Iden => self.ident(),
            _ => self.expect_one_of(&[T::NullLiteral, T::Halt, T::Iden]).map(|_| unreachable!()),
        }
    }

    fn event_list(&mut self) -> PResult<Vec<Ident>> {
        let mut v = vec![self.event_id()?];
        while self.eat(T::Comma) {
            v.push(self.event_id()?);
        }
        Ok(v)
    }

    // ----- types -----

    fn ty(&mut self) -> PResult<Type> {
        let start = self.peek().span;
        let kind = match self.peek_kind() {
            T::Seq => {
                self.bump();
                self.expect(T::LBrack)?;
                let t = self.ty()?;
                self.expect(T::RBrack)?;
                TypeKind::Seq(Box::new(t))
            }
            T::Set => {
                self.bump();
                self.expect(T::LBrack)?;
                let t = self.ty()?;
                self.expect(T::RBrack)?;
                TypeKind::Set(Box::new(t))
            }
            T::Map => {
                self.bump();
                self.expect(T::LBrack)?;
                let k = self.ty()?;
                self.expect(T::Comma)?;
                let v = self.ty()?;
                self.expect(T::RBrack)?;
                TypeKind::Map(Box::new(k), Box::new(v))
            }
            T::LParen => {
                self.bump();
                if self.at(T::Iden) && self.peek_at(1) == T::Colon {
                    let mut fields = Vec::new();
                    loop {
                        let n = self.ident()?;
                        self.expect(T::Colon)?;
                        let t = self.ty()?;
                        fields.push((n, t));
                        if !self.eat(T::Comma) {
                            break;
                        }
                    }
                    self.expect(T::RParen)?;
                    TypeKind::NamedTuple(fields)
                } else {
                    let mut tys = vec![self.ty()?];
                    while self.eat(T::Comma) {
                        tys.push(self.ty()?);
                    }
                    self.expect(T::RParen)?;
                    TypeKind::Tuple(tys)
                }
            }
            T::Bool => {
                self.bump();
                TypeKind::Bool
            }
            T::Int => {
                self.bump();
                TypeKind::Int
            }
            T::Float => {
                self.bump();
                TypeKind::Float
            }
            T::String => {
                self.bump();
                TypeKind::String
            }
            T::Event => {
                self.bump();
                TypeKind::Event
            }
            T::Machine => {
                self.bump();
                TypeKind::Machine
            }
            T::Data => {
                self.bump();
                TypeKind::Data
            }
            T::Any => {
                self.bump();
                TypeKind::Any
            }
            T::Iden => TypeKind::Named(self.ident()?),
            _ => return self.no_viable(),
        };
        Ok(Type { kind, span: start.to(self.prev_span()) })
    }

    // ----- statements -----

    fn stmt(&mut self) -> PResult<Stmt> {
        let start = self.peek().span;
        let kind = match self.peek_kind() {
            T::LBrace => {
                self.bump();
                let mut stmts = Vec::new();
                while !self.at(T::RBrace) {
                    if self.at(T::Eof) {
                        self.expect(T::RBrace)?;
                    }
                    stmts.push(self.stmt()?);
                }
                self.expect(T::RBrace)?;
                StmtKind::Compound(stmts)
            }
            T::Assert | T::Assume => {
                let is_assert = self.bump().kind == T::Assert;
                let cond = self.expr()?;
                let message = if self.eat(T::Comma) { Some(self.expr()?) } else { None };
                self.expect(T::Semi)?;
                if is_assert {
                    StmtKind::Assert { cond, message }
                } else {
                    StmtKind::Assume { cond, message }
                }
            }
            T::Print => {
                self.bump();
                let e = self.expr()?;
                self.expect(T::Semi)?;
                StmtKind::Print(e)
            }
            T::Return => {
                self.bump();
                let e = if self.at(T::Semi) { None } else { Some(self.expr()?) };
                self.expect(T::Semi)?;
                StmtKind::Return(e)
            }
            T::Break => {
                self.bump();
                self.expect(T::Semi)?;
                StmtKind::Break
            }
            T::Continue => {
                self.bump();
                self.expect(T::Semi)?;
                StmtKind::Continue
            }
            T::While => {
                self.bump();
                self.expect(T::LParen)?;
                let cond = self.expr()?;
                self.expect(T::RParen)?;
                let body = Box::new(self.stmt()?);
                StmtKind::While { cond, body }
            }
            T::Foreach => {
                self.bump();
                self.expect(T::LParen)?;
                let item = self.ident()?;
                self.expect(T::In)?;
                let collection = self.expr()?;
                self.expect(T::RParen)?;
                let mut invariants = Vec::new();
                while self.eat(T::Invariant) {
                    invariants.push(self.expr()?);
                    self.expect(T::Semi)?;
                }
                let body = Box::new(self.stmt()?);
                StmtKind::Foreach { item, collection, invariants, body }
            }
            T::If => {
                self.bump();
                self.expect(T::LParen)?;
                let cond = self.expr()?;
                self.expect(T::RParen)?;
                let then_branch = Box::new(self.stmt()?);
                let else_branch = if self.eat(T::Else) { Some(Box::new(self.stmt()?)) } else { None };
                StmtKind::If { cond, then_branch, else_branch }
            }
            T::New => {
                self.bump();
                let interface = self.ident()?;
                self.expect(T::LParen)?;
                let args = if self.at(T::RParen) { Vec::new() } else { self.rvalue_list()? };
                self.expect(T::RParen)?;
                self.expect(T::Semi)?;
                StmtKind::Ctor { interface, args }
            }
            T::Raise => {
                self.bump();
                let event = self.expr()?;
                let args = if self.eat(T::Comma) { self.rvalue_list()? } else { Vec::new() };
                self.expect(T::Semi)?;
                StmtKind::Raise { event, args }
            }
            T::Send => {
                self.bump();
                let target = self.expr()?;
                self.expect(T::Comma)?;
                let event = self.expr()?;
                let args = if self.eat(T::Comma) { self.rvalue_list()? } else { Vec::new() };
                self.expect(T::Semi)?;
                StmtKind::Send { target, event, args }
            }
            T::Announce => {
                self.bump();
                let event = self.expr()?;
                let args = if self.eat(T::Comma) { self.rvalue_list()? } else { Vec::new() };
                self.expect(T::Semi)?;
                StmtKind::Announce { event, args }
            }
            T::Goto => {
                self.bump();
                let state = self.ident()?;
                let args = if self.eat(T::Comma) { self.rvalue_list()? } else { Vec::new() };
                self.expect(T::Semi)?;
                StmtKind::Goto { state, args }
            }
            T::Receive => {
                self.bump();
                self.expect(T::LBrace)?;
                let mut cases = Vec::new();
                while self.at(T::Case) {
                    let cstart = self.bump().span;
                    let events = self.event_list()?;
                    self.expect(T::Colon)?;
                    let handler = self.anon_handler(true)?;
                    cases.push(RecvCase { events, handler, span: cstart.to(self.prev_span()) });
                }
                if cases.is_empty() {
                    self.expect(T::Case)?;
                }
                self.expect(T::RBrace)?;
                StmtKind::Receive { cases }
            }
            T::Semi => {
                self.bump();
                StmtKind::NoStmt
            }
            T::Iden => {
                if self.peek_at(1) == T::LParen {
                    let fun = self.ident()?;
                    self.expect(T::LParen)?;
                    let args = if self.at(T::RParen) { Vec::new() } else { self.rvalue_list()? };
                    self.expect(T::RParen)?;
                    self.expect(T::Semi)?;
                    StmtKind::FunCall { fun, args }
                } else {
                    let lhs = self.lvalue()?;
                    match self.peek_kind() {
                        T::Assign => {
                            self.bump();
                            let rhs = self.expr()?;
                            self.expect(T::Semi)?;
                            StmtKind::Assign { lhs, rhs }
                        }
                        T::Insert => {
                            self.bump();
                            self.expect(T::LParen)?;
                            let first = self.expr()?;
                            if self.eat(T::Comma) {
                                let value = self.expr()?;
                                self.expect(T::RParen)?;
                                self.expect(T::Semi)?;
                                StmtKind::Insert { lhs, index: first, value }
                            } else {
                                self.expect(T::RParen)?;
                                self.expect(T::Semi)?;
                                StmtKind::Add { lhs, value: first }
                            }
                        }
                        T::Remove => {
                            self.bump();
                            let value = self.expr()?;
                            self.expect(T::Semi)?;
                            StmtKind::Remove { lhs, value }
                        }
                        _ => {
                            let t = self.peek();
                            return self.err_at(
                                t,
                                format!("mismatched input '{}' expecting {{'=', '+=', '-='}}", self.offending(t)),
                            );
                        }
                    }
                }
            }
            _ => return self.no_viable(),
        };
        Ok(Stmt { kind, span: start.to(self.prev_span()) })
    }

    fn lvalue(&mut self) -> PResult<Expr> {
        let start = self.peek().span;
        let name = self.ident()?;
        let mut e = Expr { kind: ExprKind::Name(name), span: start, inner: start };
        loop {
            match self.peek_kind() {
                T::Dot => {
                    self.bump();
                    if self.at(T::IntLiteral) {
                        let t = self.bump();
                        e = Expr {
                            kind: ExprKind::TupleAccess {
                                base: Box::new(e),
                                index: self.tok_text(t).to_string(),
                                index_span: t.span,
                            },
                            span: start.to(t.span),
                            inner: start.to(t.span),
                        };
                    } else {
                        let field = self.ident()?;
                        let sp = start.to(field.span);
                        e = Expr { kind: ExprKind::NamedTupleAccess { base: Box::new(e), field }, span: sp, inner: sp };
                    }
                }
                T::LBrack => {
                    self.bump();
                    let index = self.expr()?;
                    let end = self.expect(T::RBrack)?.span;
                    e = Expr { kind: ExprKind::Index { base: Box::new(e), index: Box::new(index) }, span: start.to(end), inner: start.to(end) };
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn rvalue_list(&mut self) -> PResult<Vec<Expr>> {
        let mut v = vec![self.expr()?];
        while self.eat(T::Comma) {
            v.push(self.expr()?);
        }
        Ok(v)
    }

    // ----- expressions -----
    //
    // Precedence (highest first), mirroring the alternative order in PParser.g4:
    //   postfix: `.f`, `.n`, `is`, `targets`, `[i]`
    //   prefix: inflight, sent, -, !
    //   * / %
    //   + -
    //   as / to (postfix cast)
    //   < > >= <= in
    //   == !=
    //   &&
    //   ||
    //   ==>
    //   <==>

    fn expr(&mut self) -> PResult<Expr> {
        self.binary(0)
    }

    fn binop_at_level(kind: TokenKind, level: u8) -> Option<BinOp> {
        match (level, kind) {
            (0, T::LIff) => Some(BinOp::Iff),
            (1, T::LThen) => Some(BinOp::Then),
            (2, T::LOr) => Some(BinOp::Or),
            (3, T::LAnd) => Some(BinOp::And),
            (4, T::Eq) => Some(BinOp::Eq),
            (4, T::Ne) => Some(BinOp::Ne),
            (5, T::Lt) => Some(BinOp::Lt),
            (5, T::Gt) => Some(BinOp::Gt),
            (5, T::Ge) => Some(BinOp::Ge),
            (5, T::Le) => Some(BinOp::Le),
            (5, T::In) => Some(BinOp::In),
            // level 6 is the cast
            (7, T::Add) => Some(BinOp::Add),
            (7, T::Sub) => Some(BinOp::Sub),
            (8, T::Mul) => Some(BinOp::Mul),
            (8, T::Div) => Some(BinOp::Div),
            (8, T::Mod) => Some(BinOp::Mod),
            _ => None,
        }
    }

    fn binary(&mut self, level: u8) -> PResult<Expr> {
        if level == 6 {
            // cast level: unary followed by any number of `as`/`to` casts
            let mut e = self.binary(7)?;
            while self.at(T::As) || self.at(T::To) {
                let kind = if self.bump().kind == T::As { CastKind::As } else { CastKind::To };
                let ty = self.ty()?;
                let sp = e.span.to(ty.span);
                e = Expr { kind: ExprKind::Cast { kind, expr: Box::new(e), ty }, span: sp, inner: sp };
            }
            return Ok(e);
        }
        if level > 8 {
            return self.unary();
        }
        let mut lhs = self.binary(level + 1)?;
        loop {
            let t = self.peek();
            let Some(op) = Self::binop_at_level(t.kind, level) else { break };
            self.bump();
            let rhs = self.binary(level + 1)?;
            let sp = lhs.span.to(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op, op_span: t.span, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span: sp, inner: sp };
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let t = self.peek();
        match t.kind {
            T::Sub | T::LNot => {
                self.bump();
                let op = if t.kind == T::Sub { UnOp::Neg } else { UnOp::Not };
                let operand = self.unary()?;
                let sp = t.span.to(operand.span);
                Ok(Expr { kind: ExprKind::Unary { op, operand: Box::new(operand) }, span: sp, inner: sp })
            }
            T::Inflight => {
                self.bump();
                let e = self.unary()?;
                let sp = t.span.to(e.span);
                Ok(Expr { kind: ExprKind::Flying(Box::new(e)), span: sp, inner: sp })
            }
            T::Sent => {
                self.bump();
                let e = self.unary()?;
                let sp = t.span.to(e.span);
                Ok(Expr { kind: ExprKind::Sent(Box::new(e)), span: sp, inner: sp })
            }
            _ => self.postfix(),
        }
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let mut e = self.primary()?;
        let start = e.span;
        loop {
            match self.peek_kind() {
                T::Dot => {
                    self.bump();
                    if self.at(T::IntLiteral) {
                        let t = self.bump();
                        e = Expr {
                            kind: ExprKind::TupleAccess {
                                base: Box::new(e),
                                index: self.tok_text(t).to_string(),
                                index_span: t.span,
                            },
                            span: start.to(t.span),
                            inner: start.to(t.span),
                        };
                    } else {
                        let field = self.ident()?;
                        let sp = start.to(field.span);
                        e = Expr { kind: ExprKind::NamedTupleAccess { base: Box::new(e), field }, span: sp, inner: sp };
                    }
                }
                T::Is => {
                    self.bump();
                    let kind = self.ident()?;
                    let sp = start.to(kind.span);
                    e = Expr { kind: ExprKind::Test { instance: Box::new(e), kind }, span: sp, inner: sp };
                }
                T::Targets => {
                    self.bump();
                    let target = self.postfix()?;
                    let sp = start.to(target.span);
                    e = Expr { kind: ExprKind::Targets { instance: Box::new(e), target: Box::new(target) }, span: sp, inner: sp };
                }
                T::LBrack => {
                    self.bump();
                    let index = self.expr()?;
                    let end = self.expect(T::RBrack)?.span;
                    e = Expr { kind: ExprKind::Index { base: Box::new(e), index: Box::new(index) }, span: start.to(end), inner: start.to(end) };
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn args_in_parens(&mut self) -> PResult<Vec<Expr>> {
        self.expect(T::LParen)?;
        let args = if self.at(T::RParen) { Vec::new() } else { self.rvalue_list()? };
        self.expect(T::RParen)?;
        Ok(args)
    }

    fn primary(&mut self) -> PResult<Expr> {
        let t = self.peek();
        let start = t.span;
        let kind = match t.kind {
            T::LParen => {
                self.bump();
                if self.at(T::Iden) && self.peek_at(1) == T::Assign {
                    // named tuple
                    let mut fields = Vec::new();
                    loop {
                        let n = self.ident()?;
                        self.expect(T::Assign)?;
                        let v = self.expr()?;
                        fields.push((n, v));
                        if !self.eat(T::Comma) {
                            break;
                        }
                        if self.at(T::RParen) {
                            break; // trailing comma, single-field tuple
                        }
                    }
                    self.expect(T::RParen)?;
                    ExprKind::NamedTuple(fields)
                } else {
                    let first = self.expr()?;
                    if self.eat(T::Comma) {
                        let mut fields = vec![first];
                        if !self.at(T::RParen) {
                            loop {
                                fields.push(self.expr()?);
                                if !self.eat(T::Comma) {
                                    break;
                                }
                                if self.at(T::RParen) {
                                    break;
                                }
                            }
                        }
                        self.expect(T::RParen)?;
                        ExprKind::UnnamedTuple(fields)
                    } else {
                        self.expect(T::RParen)?;
                        // parenthesized expression: keep inner node, widen span
                        return Ok(Expr { kind: first.kind, span: start.to(self.prev_span()), inner: first.inner });
                    }
                }
            }
            T::Keys | T::Values | T::Sizeof => {
                self.bump();
                self.expect(T::LParen)?;
                let e = Box::new(self.expr()?);
                self.expect(T::RParen)?;
                match t.kind {
                    T::Keys => ExprKind::Keys(e),
                    T::Values => ExprKind::Values(e),
                    _ => ExprKind::Sizeof(e),
                }
            }
            T::Default => {
                self.bump();
                self.expect(T::LParen)?;
                let ty = self.ty()?;
                self.expect(T::RParen)?;
                ExprKind::Default(ty)
            }
            T::New => {
                self.bump();
                let interface = self.ident()?;
                let args = self.args_in_parens()?;
                ExprKind::Ctor { interface, args }
            }
            T::Iden => {
                if self.peek_at(1) == T::LParen {
                    let fun = self.ident()?;
                    let args = self.args_in_parens()?;
                    ExprKind::FunCall { fun, args }
                } else {
                    ExprKind::Name(self.ident()?)
                }
            }
            T::Forall | T::Exists => {
                self.bump();
                let quant = if t.kind == T::Forall { Quantifier::Forall } else { Quantifier::Exists };
                let diff = self.eat(T::New);
                self.expect(T::LParen)?;
                let bound = self.fun_param_list()?;
                self.expect(T::RParen)?;
                self.expect(T::Colon)?;
                self.expect(T::Colon)?;
                let body = Box::new(self.expr()?);
                ExprKind::Quant { quant, diff, bound, body }
            }
            T::Choose => {
                self.bump();
                self.expect(T::LParen)?;
                let e = if self.at(T::RParen) { None } else { Some(Box::new(self.expr()?)) };
                self.expect(T::RParen)?;
                ExprKind::Choose(e)
            }
            T::StringLiteral => {
                self.bump();
                ExprKind::Str { raw: self.tok_text(t).to_string(), raw_span: t.span, args: Vec::new(), is_format: false }
            }
            T::Format => {
                self.bump();
                self.expect(T::LParen)?;
                let s = self.expect(T::StringLiteral)?;
                let args = if self.eat(T::Comma) { self.rvalue_list()? } else { Vec::new() };
                self.expect(T::RParen)?;
                ExprKind::Str { raw: self.tok_text(s).to_string(), raw_span: s.span, args, is_format: true }
            }
            T::Float => {
                // float(base, exp)
                self.bump();
                self.expect(T::LParen)?;
                let b = self.expect(T::IntLiteral)?;
                self.expect(T::Comma)?;
                let e = self.expect(T::IntLiteral)?;
                self.expect(T::RParen)?;
                ExprKind::FloatLit(format!("{}e{}", self.tok_text(b), self.tok_text(e)))
            }
            T::IntLiteral => {
                self.bump();
                if self.at(T::Dot) && self.peek_at(1) == T::IntLiteral {
                    self.bump();
                    let post = self.bump();
                    ExprKind::FloatLit(format!("{}.{}", self.tok_text(t), self.tok_text(post)))
                } else {
                    ExprKind::IntLit(self.tok_text(t).to_string())
                }
            }
            T::Dot => {
                // `.5`
                self.bump();
                let post = self.expect(T::IntLiteral)?;
                ExprKind::FloatLit(format!(".{}", self.tok_text(post)))
            }
            T::BoolLiteral => {
                self.bump();
                ExprKind::BoolLit(self.tok_text(t) == "true")
            }
            T::NullLiteral => {
                self.bump();
                ExprKind::NullLit
            }
            T::Nondet => {
                self.bump();
                ExprKind::Nondet
            }
            T::FairNondet => {
                self.bump();
                ExprKind::FairNondet
            }
            T::Halt => {
                self.bump();
                ExprKind::Halt
            }
            T::This => {
                self.bump();
                ExprKind::This
            }
            _ => return self.no_viable(),
        };
        let span = start.to(self.prev_span());
        Ok(Expr { kind, span, inner: span })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_precedence() {
        let p = parse(FileId(0), "machine M { start state S { entry { x = -a.0 * b + c as int < d && e || f; } } }").unwrap();
        let TopDecl::Machine(m) = &p.decls[0] else { panic!() };
        let StateItemKind::Entry(Handler::Anon(h)) = &m.states[0].items[0].kind else { panic!() };
        let StmtKind::Assign { rhs, .. } = &h.body.stmts[0].kind else { panic!() };
        let ExprKind::Binary { op: BinOp::Or, lhs, .. } = &rhs.kind else { panic!("{:?}", rhs.kind) };
        let ExprKind::Binary { op: BinOp::And, lhs, .. } = &lhs.kind else { panic!() };
        let ExprKind::Binary { op: BinOp::Lt, lhs, .. } = &lhs.kind else { panic!() };
        let ExprKind::Cast { expr, .. } = &lhs.kind else { panic!() };
        let ExprKind::Binary { op: BinOp::Add, lhs, .. } = &expr.kind else { panic!() };
        let ExprKind::Binary { op: BinOp::Mul, lhs, .. } = &lhs.kind else { panic!() };
        let ExprKind::Unary { op: UnOp::Neg, operand } = &lhs.kind else { panic!() };
        assert!(matches!(operand.kind, ExprKind::TupleAccess { .. }));
    }

    #[test]
    fn parses_tuples_and_floats() {
        let p = parse(FileId(0), "fun f() { x = (1,); y = (a=1, b=2.5); z = (1); w = .5; }").unwrap();
        let TopDecl::Fun(f) = &p.decls[0] else { panic!() };
        let b = f.body.as_ref().unwrap();
        assert!(matches!(&b.stmts[0].kind, StmtKind::Assign { rhs, .. } if matches!(rhs.kind, ExprKind::UnnamedTuple(_))));
        assert!(matches!(&b.stmts[1].kind, StmtKind::Assign { rhs, .. } if matches!(rhs.kind, ExprKind::NamedTuple(_))));
        assert!(matches!(&b.stmts[2].kind, StmtKind::Assign { rhs, .. } if matches!(rhs.kind, ExprKind::IntLit(_))));
        assert!(matches!(&b.stmts[3].kind, StmtKind::Assign { rhs, .. } if matches!(rhs.kind, ExprKind::FloatLit(_))));
    }

    #[test]
    fn reports_error() {
        let e = parse(FileId(0), "machine M { start state S { entry { x = ; } } }").unwrap_err();
        assert!(e.message.contains("no viable alternative"), "{}", e.message);
    }
}
