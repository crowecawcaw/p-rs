//! Lowering of the semantic IR to [`bytecode`](crate::bytecode).

use std::rc::Rc;

use pnext_sema::{
    BinOp, Expr, ExprKind, FunDecl, FunId, PType, Program, Stmt, StmtKind, UnOp, VarId, VarRole,
};

use crate::bytecode::{CodeBlock, CollKind, Op, PathElem, ReceiveCase, Root};

/// Compiled code for a whole program, indexed by [`FunId`].
#[derive(Debug, Clone, Default)]
pub struct Code {
    pub funs: Vec<Option<Rc<CodeBlock>>>,
    /// Receive set (permissions) of every interface, indexed by `InterfaceId`.
    pub permissions: Vec<Rc<[pnext_sema::EventId]>>,
}

impl Code {
    pub fn get(&self, f: FunId) -> Option<&Rc<CodeBlock>> {
        self.funs.get(f.idx()).and_then(|c| c.as_ref())
    }
}

/// Compiles every root function with a body.
pub fn compile_program(prog: &Program) -> Code {
    let mut funs = Vec::with_capacity(prog.funs.len());
    for (i, f) in prog.funs.iter().enumerate() {
        if f.parent.is_none() && f.body.is_some() {
            funs.push(Some(Rc::new(compile_fun(prog, FunId(i as u32)))));
        } else {
            funs.push(None);
        }
    }
    let permissions = (0..prog.interfaces.len())
        .map(|i| Rc::from(prog.interface_receives(pnext_sema::InterfaceId(i as u32))))
        .collect();
    Code { funs, permissions }
}

struct LoopCtx {
    break_patches: Vec<usize>,
    continue_target: usize,
}

struct Compiler<'a> {
    prog: &'a Program,
    ops: Vec<Op>,
    next_hidden: u32,
    loops: Vec<LoopCtx>,
}

pub fn compile_fun(prog: &Program, id: FunId) -> CodeBlock {
    let f = prog.fun(id);
    let mut c = Compiler { prog, ops: Vec::new(), next_hidden: f.frame_size, loops: Vec::new() };
    c.init_locals(f);
    if let Some(body) = &f.body {
        for s in &body.stmts {
            c.stmt(s);
        }
    }
    c.emit(Op::PushNull);
    c.emit(Op::Return);
    let param_slots = f.params.iter().map(|v| prog.var(*v).slot).collect();
    CodeBlock { ops: c.ops, frame_size: c.next_hidden, param_slots }
}

impl<'a> Compiler<'a> {
    fn emit(&mut self, op: Op) -> usize {
        self.ops.push(op);
        self.ops.len() - 1
    }

    fn here(&self) -> u32 {
        self.ops.len() as u32
    }

    fn patch(&mut self, at: usize, target: u32) {
        match &mut self.ops[at] {
            Op::Jump(t) | Op::JumpIfFalse(t) | Op::JumpIfTrue(t) => *t = target,
            Op::ForeachNext { exit, .. } => *exit = target,
            Op::Receive { exit, .. } => *exit = target,
            _ => unreachable!("patching a non-jump"),
        }
    }

    fn init_locals(&mut self, f: &FunDecl) {
        for v in &f.locals {
            let d = self.prog.var(*v);
            self.emit(Op::Default(crate::value::default_value(&d.ty, self.prog)));
            self.emit(Op::StoreLocal(d.slot));
        }
    }

    fn var_root(&self, v: VarId) -> Root {
        let d = self.prog.var(v);
        match d.role {
            VarRole::Field => Root::Field(d.slot),
            _ => Root::Local(d.slot),
        }
    }

    fn coll_kind(&self, ty: &PType) -> CollKind {
        match ty.canonicalize(self.prog) {
            PType::Seq(_) => CollKind::Seq,
            PType::Set(_) => CollKind::Set,
            PType::Map(..) => CollKind::Map,
            _ => CollKind::Seq,
        }
    }

    // ---- statements -------------------------------------------------------

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Block(b) => {
                for s in &b.stmts {
                    self.stmt(s);
                }
            }
            StmtKind::Assert { cond, message } | StmtKind::Assume { cond, message } => {
                // Upstream lowers `assert c, m` to `if (!c) { m-deps; assert c, m }` unless `c`
                // is a literal, so the message is only evaluated when the assertion fails.
                if matches!(cond.kind, ExprKind::BoolLit(_)) {
                    self.expr(cond);
                    self.expr(message);
                    self.emit(Op::Assert);
                } else {
                    self.expr(cond);
                    let skip = self.emit(Op::JumpIfTrue(0));
                    self.emit(Op::PushBool(false));
                    self.expr(message);
                    self.emit(Op::Assert);
                    let end = self.here();
                    self.patch(skip, end);
                }
            }
            StmtKind::Print(e) => {
                self.expr(e);
                self.emit(Op::Print);
            }
            StmtKind::Return(e) => {
                match e {
                    Some(e) => self.expr(e),
                    None => {
                        self.emit(Op::PushNull);
                    }
                }
                self.emit(Op::Return);
            }
            StmtKind::Break => {
                let at = self.emit(Op::Jump(0));
                if let Some(l) = self.loops.last_mut() {
                    l.break_patches.push(at);
                }
            }
            StmtKind::Continue => {
                let target = self.loops.last().map(|l| l.continue_target).unwrap_or(0) as u32;
                self.emit(Op::Jump(target));
            }
            StmtKind::Assign { lhs, rhs } => {
                let (root, path) = self.lvalue(lhs);
                self.expr(rhs);
                self.emit(Op::Store { root, path });
            }
            StmtKind::Insert { lhs, index, value } => {
                let kind = self.coll_kind(&lhs.ty);
                let (root, path) = self.lvalue(lhs);
                self.expr(index);
                self.expr(value);
                self.emit(Op::Insert { root, path, kind });
            }
            StmtKind::Add { lhs, value } => {
                let (root, path) = self.lvalue(lhs);
                self.expr(value);
                self.emit(Op::Add { root, path });
            }
            StmtKind::Remove { lhs, value } => {
                let kind = self.coll_kind(&lhs.ty);
                let (root, path) = self.lvalue(lhs);
                self.expr(value);
                self.emit(Op::Remove { root, path, kind });
            }
            StmtKind::While { cond, body } => {
                let start = self.here();
                self.expr(cond);
                let jf = self.emit(Op::JumpIfFalse(0));
                self.loops.push(LoopCtx { break_patches: Vec::new(), continue_target: start as usize });
                self.stmt(body);
                self.emit(Op::Jump(start));
                let end = self.here();
                self.patch(jf, end);
                let l = self.loops.pop().unwrap();
                for p in l.break_patches {
                    self.patch(p, end);
                }
            }
            StmtKind::Foreach { item, collection, body } => {
                let kind = self.coll_kind(&collection.ty);
                let root = match &collection.kind {
                    ExprKind::Var(v) if self.prog.var(*v).role != VarRole::GlobalParam => Some(self.var_root(*v)),
                    _ => None,
                };
                let iter_slot = self.next_hidden;
                self.next_hidden += 2;
                let item_slot = self.prog.var(*item).slot;
                self.expr(collection);
                self.emit(Op::ForeachInit { iter_slot, kind, root });
                let start = self.here();
                let next = self.emit(Op::ForeachNext { iter_slot, item_slot, exit: 0 });
                self.loops.push(LoopCtx { break_patches: Vec::new(), continue_target: start as usize });
                self.stmt(body);
                self.emit(Op::Jump(start));
                let end = self.here();
                self.patch(next, end);
                let l = self.loops.pop().unwrap();
                for p in l.break_patches {
                    self.patch(p, end);
                }
            }
            StmtKind::If { cond, then_branch, else_branch } => {
                self.expr(cond);
                let jf = self.emit(Op::JumpIfFalse(0));
                self.stmt(then_branch);
                if matches!(else_branch.kind, StmtKind::Nop) {
                    let end = self.here();
                    self.patch(jf, end);
                } else {
                    let j = self.emit(Op::Jump(0));
                    let else_start = self.here();
                    self.patch(jf, else_start);
                    self.stmt(else_branch);
                    let end = self.here();
                    self.patch(j, end);
                }
            }
            StmtKind::Ctor { interface, payload } => {
                let has_payload = self.opt_expr(payload.as_ref());
                self.emit(Op::Create { iface: *interface, has_payload });
                self.emit(Op::Pop);
            }
            StmtKind::FunCall { fun, args } => {
                for a in args {
                    self.expr(a);
                }
                self.emit(Op::Call { fun: *fun, nargs: args.len() as u32 });
                self.emit(Op::Pop);
            }
            StmtKind::Raise { event, payload } => {
                self.expr(event);
                let has_payload = self.opt_expr(payload.as_ref());
                self.emit(Op::Raise { has_payload });
            }
            StmtKind::Send { target, event, payload } => {
                self.expr(target);
                self.expr(event);
                let has_payload = self.opt_expr(payload.as_ref());
                self.emit(Op::Send { has_payload });
            }
            StmtKind::Announce { event, payload } => {
                self.expr(event);
                let has_payload = self.opt_expr(payload.as_ref());
                self.emit(Op::Announce { has_payload });
            }
            StmtKind::Goto { state, payload } => {
                let has_payload = self.opt_expr(payload.as_ref());
                self.emit(Op::Goto { state: *state, has_payload });
            }
            StmtKind::Receive { cases } => {
                let recv = self.emit(Op::Receive { cases: Rc::from(Vec::new()), exit: 0 });
                let mut compiled = Vec::new();
                let mut end_jumps = Vec::new();
                for (ev, fun) in cases {
                    let f = self.prog.fun(*fun);
                    let param_slot = f.params.first().map(|v| self.prog.var(*v).slot);
                    let target = self.here();
                    self.init_locals(f);
                    if let Some(body) = &f.body {
                        for s in &body.stmts {
                            self.stmt(s);
                        }
                    }
                    end_jumps.push(self.emit(Op::Jump(0)));
                    compiled.push(ReceiveCase { event: *ev, param_slot, target });
                }
                let end = self.here();
                for j in end_jumps {
                    self.patch(j, end);
                }
                self.ops[recv] = Op::Receive { cases: Rc::from(compiled), exit: end };
            }
            StmtKind::Nop => {}
        }
    }

    fn opt_expr(&mut self, e: Option<&Expr>) -> bool {
        match e {
            Some(e) => {
                self.expr(e);
                true
            }
            None => false,
        }
    }

    /// Emits the key expressions of an l-value and returns its root and path.
    fn lvalue(&mut self, e: &Expr) -> (Root, Rc<[PathElem]>) {
        let mut path = Vec::new();
        let root = self.lvalue_rec(e, &mut path);
        (root, Rc::from(path))
    }

    fn lvalue_rec(&mut self, e: &Expr, path: &mut Vec<PathElem>) -> Root {
        match &e.kind {
            ExprKind::Var(v) => self.var_root(*v),
            ExprKind::TupleAccess { base, index } | ExprKind::NamedTupleAccess { base, index } => {
                let r = self.lvalue_rec(base, path);
                path.push(PathElem::TupleField(*index));
                r
            }
            ExprKind::SeqAccess { base, index } => {
                let r = self.lvalue_rec(base, path);
                self.expr(index);
                path.push(PathElem::SeqIndex);
                r
            }
            ExprKind::MapAccess { base, key } => {
                let r = self.lvalue_rec(base, path);
                self.expr(key);
                path.push(PathElem::MapKey);
                r
            }
            // Not assignable (sets, globals, arbitrary expressions): evaluate into a
            // scratch slot so the store is harmless.
            _ => {
                self.expr(e);
                let slot = self.next_hidden;
                self.next_hidden += 1;
                self.emit(Op::StoreLocal(slot));
                Root::Local(slot)
            }
        }
    }

    // ---- expressions ------------------------------------------------------

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Var(v) => {
                let d = self.prog.var(*v);
                match d.role {
                    VarRole::Field => self.emit(Op::LoadField(d.slot)),
                    VarRole::GlobalParam => self.emit(Op::LoadGlobal(d.slot)),
                    _ => self.emit(Op::LoadLocal(d.slot)),
                };
            }
            ExprKind::EnumElem(en, i) => {
                let v = self.prog.enums[en.idx()].elems[*i as usize].value;
                self.emit(Op::PushInt(v));
            }
            ExprKind::EventRef(ev) => {
                self.emit(Op::PushEvent(*ev));
            }
            ExprKind::IntLit(i) => {
                self.emit(Op::PushInt(*i));
            }
            ExprKind::FloatLit(f) => {
                self.emit(Op::PushFloat(*f));
            }
            ExprKind::BoolLit(b) => {
                self.emit(Op::PushBool(*b));
            }
            ExprKind::NullLit | ExprKind::Error => {
                self.emit(Op::PushNull);
            }
            ExprKind::Str { base, args } => {
                for a in args {
                    self.expr(a);
                }
                let types: Rc<[PType]> = args.iter().map(|a| a.ty.clone()).collect();
                self.emit(Op::Format { base: Rc::from(base.as_str()), nargs: args.len() as u32, types });
            }
            ExprKind::Nondet | ExprKind::FairNondet => {
                self.emit(Op::ChooseBool);
            }
            ExprKind::This => {
                self.emit(Op::PushThis);
            }
            ExprKind::Tuple(es) => {
                for x in es {
                    self.expr(x);
                }
                self.emit(Op::MakeTuple(es.len() as u32));
            }
            ExprKind::NamedTuple(es) => {
                for x in es {
                    self.expr(x);
                }
                let names: Vec<String> = match e.ty.canonicalize(self.prog) {
                    PType::NamedTuple(fs) => fs.into_iter().map(|(n, _)| n).collect(),
                    _ => (0..es.len()).map(|i| i.to_string()).collect(),
                };
                self.emit(Op::MakeNamedTuple(Rc::from(names)));
            }
            ExprKind::TupleAccess { base, index } | ExprKind::NamedTupleAccess { base, index } => {
                self.expr(base);
                self.emit(Op::TupleGet(*index));
            }
            ExprKind::SeqAccess { base, index } => {
                self.expr(base);
                self.expr(index);
                self.emit(Op::SeqGet);
            }
            ExprKind::SetAccess { base, index } => {
                self.expr(base);
                self.expr(index);
                self.emit(Op::SetGet);
            }
            ExprKind::MapAccess { base, key } => {
                self.expr(base);
                self.expr(key);
                let mut ty = &e.ty;
                while let PType::TypeDef(id) = ty {
                    ty = &self.prog.typedefs[id.idx()].ty;
                }
                // Java keeps int, bool, float and machines as primitives (unboxing null throws)
                let java_null = !matches!(ty, PType::Int | PType::Bool | PType::Float | PType::Machine | PType::Permission(_));
                self.emit(Op::MapGet { java_null });
            }
            ExprKind::Keys(b) => {
                self.expr(b);
                self.emit(Op::Keys);
            }
            ExprKind::Values(b) => {
                self.expr(b);
                self.emit(Op::Values);
            }
            ExprKind::Sizeof(b) => {
                self.expr(b);
                self.emit(Op::Sizeof);
            }
            ExprKind::Default => {
                self.emit(Op::Default(crate::value::default_value(&e.ty, self.prog)));
            }
            ExprKind::Ctor { interface, payload } => {
                let has_payload = self.opt_expr(payload.as_deref());
                self.emit(Op::Create { iface: *interface, has_payload });
            }
            ExprKind::FunCall { fun, args } => {
                for a in args {
                    self.expr(a);
                }
                self.emit(Op::Call { fun: *fun, nargs: args.len() as u32 });
            }
            ExprKind::Unary { op, operand } => {
                self.expr(operand);
                match op {
                    UnOp::Neg => self.emit(Op::Neg),
                    UnOp::Not => self.emit(Op::Not),
                };
            }
            ExprKind::Binary { op, lhs, rhs } => match op {
                BinOp::And => {
                    self.expr(lhs);
                    self.emit(Op::Dup);
                    let j = self.emit(Op::JumpIfFalse(0));
                    self.emit(Op::Pop);
                    self.expr(rhs);
                    let end = self.here();
                    self.patch(j, end);
                }
                BinOp::Or => {
                    self.expr(lhs);
                    self.emit(Op::Dup);
                    let j = self.emit(Op::JumpIfTrue(0));
                    self.emit(Op::Pop);
                    self.expr(rhs);
                    let end = self.here();
                    self.patch(j, end);
                }
                BinOp::Then => {
                    self.expr(lhs);
                    self.emit(Op::Not);
                    self.emit(Op::Dup);
                    let j = self.emit(Op::JumpIfTrue(0));
                    self.emit(Op::Pop);
                    self.expr(rhs);
                    let end = self.here();
                    self.patch(j, end);
                }
                BinOp::Iff => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.emit(Op::Bin(BinOp::Eq));
                }
                other => {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.emit(Op::Bin(*other));
                }
            },
            ExprKind::Contains { item, collection } => {
                let kind = self.coll_kind(&collection.ty);
                self.expr(item);
                self.expr(collection);
                self.emit(Op::Contains(kind));
            }
            ExprKind::Cast(inner) => {
                self.expr(inner);
                self.emit(Op::Cast(std::rc::Rc::new(e.ty.clone())));
            }
            ExprKind::Coerce(inner) => {
                self.expr(inner);
                match e.ty.canonicalize(self.prog) {
                    PType::Float => {
                        self.emit(Op::CoerceFloat);
                    }
                    PType::Int => {
                        self.emit(Op::CoerceInt);
                    }
                    PType::Permission(pnext_sema::PermissionOrigin::Interface(i)) => {
                        self.emit(Op::CoerceInterface(i));
                    }
                    _ => {}
                }
            }
            ExprKind::Choose(None) => {
                self.emit(Op::ChooseBool);
            }
            ExprKind::Choose(Some(inner)) => {
                self.expr(inner);
                let op = match inner.ty.canonicalize(self.prog) {
                    PType::Seq(_) => Op::ChooseSeq,
                    PType::Set(_) => Op::ChooseSet,
                    PType::Map(..) => Op::ChooseMap,
                    _ => Op::ChooseInt,
                };
                self.emit(op);
            }
        }
    }
}
