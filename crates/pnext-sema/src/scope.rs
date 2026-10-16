//! Lexical scopes with upstream's namespace and conflict rules (`Scope.cs`).

use std::collections::HashMap;

use crate::program::*;
use pnext_syntax::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// What a name in a given namespace refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleRef {
    Named(u32),
    Test(TestId),
    Refinement(u32),
    Impl(ImplId),
}

#[derive(Debug, Default)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub enum_elems: HashMap<String, (EnumId, u32)>,
    pub enums: HashMap<String, EnumId>,
    pub events: HashMap<String, EventId>,
    pub event_sets: HashMap<String, EventSetId>,
    pub functions: HashMap<String, FunId>,
    pub interfaces: HashMap<String, InterfaceId>,
    pub machines: HashMap<String, MachineId>,
    pub states: HashMap<String, StateId>,
    pub typedefs: HashMap<String, TypeDefId>,
    pub variables: HashMap<String, VarId>,
    pub named_modules: HashMap<String, u32>,
    pub safety_tests: HashMap<String, TestId>,
    pub refinement_tests: HashMap<String, u32>,
    pub implementations: HashMap<String, ImplId>,
    pub pures: HashMap<String, u32>,
    pub invariants: HashMap<String, u32>,
    pub invariant_groups: HashMap<String, u32>,
    /// Machines declared in this scope, in declaration order (for state lookup).
    pub machine_list: Vec<MachineId>,
    /// Variables in declaration order.
    pub var_list: Vec<VarId>,
}

/// Arena of scopes.
#[derive(Debug, Default)]
pub struct Scopes {
    pub scopes: Vec<Scope>,
}

impl Scopes {
    pub fn new() -> Scopes {
        let mut s = Scopes::default();
        s.scopes.push(Scope::default());
        s
    }

    pub fn global(&self) -> ScopeId {
        ScopeId(0)
    }

    pub fn child(&mut self, parent: ScopeId) -> ScopeId {
        self.scopes.push(Scope { parent: Some(parent), ..Default::default() });
        ScopeId(self.scopes.len() as u32 - 1)
    }

    pub fn get(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0 as usize]
    }

    pub fn get_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.scopes[id.0 as usize]
    }

    fn chain(&self, id: ScopeId) -> impl Iterator<Item = &Scope> {
        let mut cur = Some(id);
        std::iter::from_fn(move || {
            let s = &self.scopes[cur?.0 as usize];
            cur = s.parent;
            Some(s)
        })
    }

    pub fn lookup_var(&self, id: ScopeId, name: &str) -> Option<VarId> {
        self.chain(id).find_map(|s| s.variables.get(name).copied())
    }
    pub fn lookup_event(&self, id: ScopeId, name: &str) -> Option<EventId> {
        self.chain(id).find_map(|s| s.events.get(name).copied())
    }
    pub fn lookup_enum_elem(&self, id: ScopeId, name: &str) -> Option<(EnumId, u32)> {
        self.chain(id).find_map(|s| s.enum_elems.get(name).copied())
    }
    pub fn lookup_enum(&self, id: ScopeId, name: &str) -> Option<EnumId> {
        self.chain(id).find_map(|s| s.enums.get(name).copied())
    }
    pub fn lookup_event_set(&self, id: ScopeId, name: &str) -> Option<EventSetId> {
        self.chain(id).find_map(|s| s.event_sets.get(name).copied())
    }
    pub fn lookup_fun(&self, id: ScopeId, name: &str) -> Option<FunId> {
        self.chain(id).find_map(|s| s.functions.get(name).copied())
    }
    pub fn lookup_interface(&self, id: ScopeId, name: &str) -> Option<InterfaceId> {
        self.chain(id).find_map(|s| s.interfaces.get(name).copied())
    }
    pub fn lookup_machine(&self, id: ScopeId, name: &str) -> Option<MachineId> {
        self.chain(id).find_map(|s| s.machines.get(name).copied())
    }
    pub fn lookup_typedef(&self, id: ScopeId, name: &str) -> Option<TypeDefId> {
        self.chain(id).find_map(|s| s.typedefs.get(name).copied())
    }
    pub fn lookup_named_module(&self, id: ScopeId, name: &str) -> Option<u32> {
        self.chain(id).find_map(|s| s.named_modules.get(name).copied())
    }
    pub fn lookup_pure(&self, id: ScopeId, name: &str) -> Option<u32> {
        self.chain(id).find_map(|s| s.pures.get(name).copied())
    }
    pub fn lookup_invariant(&self, id: ScopeId, name: &str) -> Option<u32> {
        self.chain(id).find_map(|s| s.invariants.get(name).copied())
    }
    pub fn lookup_invariant_group(&self, id: ScopeId, name: &str) -> Option<u32> {
        self.chain(id).find_map(|s| s.invariant_groups.get(name).copied())
    }

    /// State lookup also searches the scopes of machines declared along the chain.
    pub fn lookup_state(&self, id: ScopeId, name: &str, machine_scopes: &[ScopeId]) -> Option<StateId> {
        for s in self.chain(id) {
            if let Some(st) = s.states.get(name) {
                return Some(*st);
            }
            for m in &s.machine_list {
                let ms = machine_scopes[m.idx()];
                if let Some(st) = self.scopes[ms.0 as usize].states.get(name) {
                    return Some(*st);
                }
            }
        }
        None
    }
}

/// A conflicting declaration found by [`Scopes::conflict`].
pub struct Conflict {
    pub existing: DeclRef,
}

impl Scope {
    /// Checks `name` against the given namespaces; returns the existing declaration if any.
    pub fn conflict(&self, name: &str, namespaces: &[Namespace]) -> Option<DeclRef> {
        for ns in namespaces {
            let hit = match ns {
                Namespace::Typedefs => self.typedefs.get(name).map(|t| DeclRef::TypeDef(*t)),
                Namespace::Enums => self.enums.get(name).map(|t| DeclRef::Enum(*t)),
                Namespace::Interfaces => self.interfaces.get(name).map(|t| DeclRef::Interface(*t)),
                Namespace::Machines => self.machines.get(name).map(|t| DeclRef::Machine(*t)),
                Namespace::Events => self.events.get(name).map(|t| DeclRef::Event(*t)),
                Namespace::EnumElems => self.enum_elems.get(name).map(|(e, i)| DeclRef::EnumElem(*e, *i)),
                Namespace::EventSets => self.event_sets.get(name).map(|t| DeclRef::EventSet(*t)),
                Namespace::Functions => self.functions.get(name).map(|t| DeclRef::Fun(*t)),
                Namespace::States => self.states.get(name).map(|t| DeclRef::State(*t)),
                Namespace::Variables => self.variables.get(name).map(|t| DeclRef::Var(*t)),
                Namespace::NamedModules => self.named_modules.get(name).map(|t| DeclRef::Module(*t)),
                Namespace::Implementations => self.implementations.get(name).map(|t| DeclRef::Impl(*t)),
                Namespace::SafetyTests => self.safety_tests.get(name).map(|t| DeclRef::Test(*t)),
                Namespace::RefinementTests => self.refinement_tests.get(name).map(|_| DeclRef::Other),
                Namespace::Pures => self.pures.get(name).map(|t| DeclRef::Pure(*t)),
                Namespace::Invariants => self.invariants.get(name).map(|t| DeclRef::Invariant(*t)),
                Namespace::InvariantGroups => self.invariant_groups.get(name).map(|_| DeclRef::Other),
            };
            if hit.is_some() {
                return hit;
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Namespace {
    Typedefs,
    Enums,
    Interfaces,
    Machines,
    Events,
    EnumElems,
    EventSets,
    Functions,
    States,
    Variables,
    NamedModules,
    Implementations,
    SafetyTests,
    RefinementTests,
    Pures,
    Invariants,
    InvariantGroups,
}

/// Dummy span helper for synthesized declarations.
pub fn no_span() -> Span {
    Span::dummy()
}
