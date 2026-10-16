//! P types and subtyping, mirroring `Plang.Compiler.TypeChecker.Types`.

use crate::program::{EnumId, EventId, EventSetId, InterfaceId, MachineId, Program, TypeDefId};

/// A P language type. `TypeDef` references are kept so that error messages can use the
/// name the user wrote (`OriginalRepresentation` upstream).
#[derive(Debug, Clone, PartialEq)]
pub enum PType {
    Bool,
    Int,
    Float,
    String,
    Event,
    Machine,
    Any,
    /// The type of "no value" (`null`), used for missing payloads and void returns.
    Null,
    Data,
    Enum(EnumId),
    Seq(Box<PType>),
    Set(Box<PType>),
    Map(Box<PType>, Box<PType>),
    Tuple(Vec<PType>),
    NamedTuple(Vec<(String, PType)>),
    Permission(PermissionOrigin),
    TypeDef(TypeDefId),
    Foreign(String),
    /// Result of an expression that already produced an error; assignable to anything.
    Error,
}

/// What a permission type refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionOrigin {
    Machine(MachineId),
    Interface(InterfaceId),
    EventSet(EventSetId),
}

/// Coarse classification used in `expected: ...` error messages (`TypeKind` upstream).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Base,
    Sequence,
    Map,
    Set,
    Tuple,
    NamedTuple,
    Foreign,
    Enum,
    TypeDef,
    Data,
}

impl TypeKind {
    pub fn name(self) -> &'static str {
        match self {
            TypeKind::Base => "base",
            TypeKind::Sequence => "sequence",
            TypeKind::Map => "map",
            TypeKind::Set => "set",
            TypeKind::Tuple => "tuple",
            TypeKind::NamedTuple => "namedtuple",
            TypeKind::Foreign => "foreign",
            TypeKind::Enum => "enum",
            TypeKind::TypeDef => "typedef",
            TypeKind::Data => "bounded",
        }
    }
}

impl PType {
    pub fn seq(t: PType) -> PType {
        PType::Seq(Box::new(t))
    }

    pub fn set(t: PType) -> PType {
        PType::Set(Box::new(t))
    }

    pub fn map(k: PType, v: PType) -> PType {
        PType::Map(Box::new(k), Box::new(v))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, PType::Error)
    }

    /// The type with all typedefs expanded.
    pub fn canonicalize(&self, prog: &Program) -> PType {
        match self {
            PType::TypeDef(id) => prog.typedefs[id.0 as usize].ty.canonicalize(prog),
            PType::Seq(t) => PType::seq(t.canonicalize(prog)),
            PType::Set(t) => PType::set(t.canonicalize(prog)),
            PType::Map(k, v) => PType::map(k.canonicalize(prog), v.canonicalize(prog)),
            PType::Tuple(ts) => PType::Tuple(ts.iter().map(|t| t.canonicalize(prog)).collect()),
            PType::NamedTuple(fs) => {
                PType::NamedTuple(fs.iter().map(|(n, t)| (n.clone(), t.canonicalize(prog))).collect())
            }
            other => other.clone(),
        }
    }

    pub fn kind(&self, prog: &Program) -> TypeKind {
        match self.canonicalize(prog) {
            PType::Seq(_) => TypeKind::Sequence,
            PType::Set(_) => TypeKind::Set,
            PType::Map(..) => TypeKind::Map,
            PType::Tuple(_) => TypeKind::Tuple,
            PType::NamedTuple(_) => TypeKind::NamedTuple,
            PType::Enum(_) => TypeKind::Enum,
            PType::Foreign(_) => TypeKind::Foreign,
            PType::Data => TypeKind::Data,
            PType::TypeDef(_) => TypeKind::TypeDef,
            _ => TypeKind::Base,
        }
    }

    pub fn is_kind(&self, prog: &Program, kind: TypeKind) -> bool {
        self.kind(prog) == kind
    }

    /// The representation as written by the user (`OriginalRepresentation`).
    pub fn original_repr(&self, prog: &Program) -> String {
        match self {
            PType::Bool => "bool".into(),
            PType::Int => "int".into(),
            PType::Float => "float".into(),
            PType::String => "string".into(),
            PType::Event => "event".into(),
            PType::Machine => "machine".into(),
            PType::Any => "any".into(),
            PType::Null => "null".into(),
            PType::Data => "data".into(),
            PType::Enum(id) => prog.enums[id.0 as usize].name.clone(),
            PType::Seq(t) => format!("seq[{}]", t.original_repr(prog)),
            PType::Set(t) => format!("set[{}]", t.original_repr(prog)),
            PType::Map(k, v) => format!("map[{},{}]", k.original_repr(prog), v.original_repr(prog)),
            PType::Tuple(ts) => {
                format!("({})", ts.iter().map(|t| t.original_repr(prog)).collect::<Vec<_>>().join(","))
            }
            PType::NamedTuple(fs) => format!(
                "({})",
                fs.iter().map(|(n, t)| format!("{}:{}", n, t.original_repr(prog))).collect::<Vec<_>>().join(",")
            ),
            PType::Permission(o) => prog.permission_name(*o).to_string(),
            PType::TypeDef(id) => prog.typedefs[id.0 as usize].name.clone(),
            PType::Foreign(n) => n.clone(),
            PType::Error => "<error>".into(),
        }
    }

    /// Fully expanded representation (`CanonicalRepresentation`).
    pub fn canonical_repr(&self, prog: &Program) -> String {
        self.canonicalize(prog).original_repr(prog)
    }

    /// The events a value of this type may carry as permissions. `None` means "unbounded"
    /// (`any`, `machine`, foreign types and containers of those).
    pub fn allowed_permissions(&self, prog: &Program) -> Option<Vec<EventId>> {
        match self {
            PType::Any | PType::Machine | PType::Foreign(_) => None,
            PType::Bool
            | PType::Int
            | PType::Float
            | PType::String
            | PType::Event
            | PType::Null
            | PType::Data
            | PType::Enum(_)
            | PType::Error => Some(Vec::new()),
            PType::Seq(t) | PType::Set(t) => t.allowed_permissions(prog),
            PType::Map(k, v) => {
                let mut a = k.allowed_permissions(prog)?;
                a.extend(v.allowed_permissions(prog)?);
                Some(a)
            }
            PType::Tuple(ts) => {
                let mut a = Vec::new();
                for t in ts {
                    a.extend(t.allowed_permissions(prog)?);
                }
                Some(a)
            }
            PType::NamedTuple(fs) => {
                let mut a = Vec::new();
                for (_, t) in fs {
                    a.extend(t.allowed_permissions(prog)?);
                }
                Some(a)
            }
            PType::Permission(o) => Some(prog.permission_events(*o)),
            PType::TypeDef(id) => prog.typedefs[id.0 as usize].ty.canonicalize(prog).allowed_permissions(prog),
        }
    }

    /// `this.IsAssignableFrom(other)`.
    pub fn is_assignable_from(&self, other: &PType, prog: &Program) -> bool {
        if self.is_error() || other.is_error() {
            return true;
        }
        match self {
            PType::TypeDef(id) => prog.typedefs[id.0 as usize].ty.is_assignable_from(other, prog),
            PType::Any => true,
            PType::Machine => {
                let oc = other.canonical_repr(prog);
                oc == "machine" || oc == "null" || matches!(other.canonicalize(prog), PType::Permission(_))
            }
            PType::Int => other.canonical_repr(prog) == "int",
            PType::String => other.canonical_repr(prog) == "string",
            PType::Event => {
                let oc = other.canonical_repr(prog);
                oc == "event" || oc == "null"
            }
            PType::Bool | PType::Float | PType::Null => self.canonical_repr(prog) == other.canonical_repr(prog),
            PType::Data => match other.allowed_permissions(prog) {
                None => false,
                Some(p) => p.is_empty(),
            },
            PType::Enum(id) => matches!(other.canonicalize(prog), PType::Enum(o) if o == *id),
            PType::Seq(t) => match other.canonicalize(prog) {
                PType::Seq(o) => t.is_assignable_from(&o, prog),
                _ => false,
            },
            PType::Set(t) => match other.canonicalize(prog) {
                PType::Set(o) => t.is_assignable_from(&o, prog),
                _ => false,
            },
            PType::Map(k, v) => match other.canonicalize(prog) {
                PType::Map(ok, ov) => k.is_assignable_from(&ok, prog) && v.is_assignable_from(&ov, prog),
                _ => false,
            },
            PType::Tuple(ts) => match other.canonicalize(prog) {
                PType::Tuple(os) => {
                    ts.len() == os.len() && ts.iter().zip(os.iter()).all(|(a, b)| a.is_assignable_from(b, prog))
                }
                _ => false,
            },
            PType::NamedTuple(fs) => match other.canonicalize(prog) {
                PType::NamedTuple(os) => {
                    fs.len() == os.len()
                        && fs.iter().zip(os.iter()).all(|((n1, t1), (n2, t2))| n1 == n2 && t1.is_assignable_from(t2, prog))
                }
                _ => false,
            },
            PType::Permission(o) => match other.canonicalize(prog) {
                PType::Permission(oo) => {
                    let mut a = prog.permission_events(*o);
                    let mut b = prog.permission_events(oo);
                    a.sort();
                    a.dedup();
                    b.sort();
                    b.dedup();
                    a == b
                }
                _ => false,
            },
            PType::Foreign(n) => matches!(other.canonicalize(prog), PType::Foreign(o) if &o == n),
            PType::Error => true,
        }
    }

    pub fn is_same_type_as(&self, other: &PType, prog: &Program) -> bool {
        if self.is_error() || other.is_error() {
            return true;
        }
        self.is_assignable_from(other, prog) && other.is_assignable_from(self, prog)
    }

    /// Type equality in the sense of upstream `Equals`: same class and mutually assignable.
    pub fn equals(&self, other: &PType, prog: &Program) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other) && self.is_same_type_as(other, prog)
    }
}
