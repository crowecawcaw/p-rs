//! A small stack bytecode that the interpreter executes.
//!
//! Each root function of the program (methods, entry/exit actions, event handlers,
//! transition functions) is lowered to one [`CodeBlock`]. Receive-case handlers are
//! inlined into the enclosing root function and share its frame, exactly as the upstream
//! C# code generator inlines them into the enclosing method.

use std::rc::Rc;

use pnext_sema::{BinOp, EventId, FunId, InterfaceId, PType, StateId};

/// Where an l-value lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    Local(u32),
    Field(u32),
}

/// One step of an l-value path below its root. Keys for `SeqIndex`/`MapKey` are taken
/// from the operand stack in path order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathElem {
    TupleField(u32),
    SeqIndex,
    MapKey,
}

/// Which collection kind a `Contains`, `Insert`, `Remove` or `Foreach` operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollKind {
    Seq,
    Set,
    Map,
}

#[derive(Debug, Clone)]
pub struct ReceiveCase {
    pub event: EventId,
    /// Slot that receives the payload (`None` when the case takes no parameter).
    pub param_slot: Option<u32>,
    /// Program counter of the case body.
    pub target: u32,
}

#[derive(Debug, Clone)]
pub enum Op {
    PushNull,
    PushBool(bool),
    PushInt(i64),
    PushFloat(f64),
    PushStr(Rc<str>),
    PushEvent(EventId),
    PushThis,
    LoadLocal(u32),
    StoreLocal(u32),
    LoadField(u32),
    LoadGlobal(u32),
    Pop,
    Dup,
    MakeTuple(u32),
    MakeNamedTuple(Rc<[String]>),
    TupleGet(u32),
    SeqGet,
    SetGet,
    MapGet,
    Keys,
    Values,
    Sizeof,
    Contains(CollKind),
    /// Pushes a precomputed default value (values are copy-on-write, so sharing is safe).
    Default(crate::value::Value),
    Format { base: Rc<str>, nargs: u32 },
    Bin(BinOp),
    Neg,
    Not,
    Jump(u32),
    JumpIfFalse(u32),
    JumpIfTrue(u32),
    Cast(std::rc::Rc<PType>),
    CoerceFloat,
    CoerceInt,
    CoerceInterface(InterfaceId),
    ChooseBool,
    ChooseInt,
    ChooseSeq,
    ChooseSet,
    ChooseMap,
    Call { fun: FunId, nargs: u32 },
    Return,
    Store { root: Root, path: Rc<[PathElem]> },
    Insert { root: Root, path: Rc<[PathElem]>, kind: CollKind },
    Add { root: Root, path: Rc<[PathElem]> },
    Remove { root: Root, path: Rc<[PathElem]>, kind: CollKind },
    Assert,
    Print,
    Create { iface: InterfaceId, has_payload: bool },
    /// Pops payload (if any), event and target.
    Send { has_payload: bool },
    Raise { has_payload: bool },
    Announce { has_payload: bool },
    Goto { state: StateId, has_payload: bool },
    Receive { cases: Rc<[ReceiveCase]>, exit: u32 },
    /// Pops the collection, stores an iterator in `iter_slot`.
    ForeachInit { iter_slot: u32, kind: CollKind, root: Option<Root> },
    /// Advances the iterator; jumps to `exit` when exhausted, else stores the item.
    ForeachNext { iter_slot: u32, item_slot: u32, exit: u32 },
}

/// The compiled form of one root function.
#[derive(Debug, Clone, Default)]
pub struct CodeBlock {
    pub ops: Vec<Op>,
    /// Slots in a frame: declared variables plus hidden iterator slots.
    pub frame_size: u32,
    /// Slots the parameters occupy, in order.
    pub param_slots: Vec<u32>,
}
