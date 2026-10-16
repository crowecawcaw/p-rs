//! The class names upstream's C# backend (`PCheckerNameManager`) gives declarations.
//!
//! Upstream's checker reports machines by their generated class name. Every declaration
//! other than events, states and interfaces draws its name from one shared pool, uniquified
//! with `_1`, `_2`, ... in the order code generation first asks for it. A machine therefore
//! loses its plain name when an earlier variable, parameter or field already took it (e.g.
//! a field `client` in a machine generated before machine `client`, which becomes
//! `client_1`).

use std::collections::HashMap;

use crate::program::{FunId, Program, VarId};

/// C# keywords upstream prefixes with `P_`.
const RESERVED: &[&str] = &[
    "bool", "byte", "sbyte", "short", "ushort", "int", "uint", "long", "ulong", "double", "float", "decimal",
    "string", "char", "void", "object", "typeof", "sizeof", "null", "true", "false", "if", "else", "while", "for",
    "foreach", "do", "switch", "case", "default", "lock", "try", "throw", "catch", "finally", "goto", "break",
    "continue", "return", "public", "private", "internal", "protected", "static", "readonly", "sealed", "const",
    "fixed", "stackalloc", "volatile", "new", "override", "abstract", "virtual", "event", "extern", "ref", "out",
    "in", "is", "as", "params", "__arglist", "__makeref", "__reftype", "__refvalue", "this", "base", "namespace",
    "using", "class", "struct", "interface", "enum", "delegate", "checked", "unchecked", "unsafe", "operator",
    "implicit", "explicit",
];

/// `NameManagerBase.UniquifyName`.
#[derive(Default)]
struct Names {
    usages: HashMap<String, u32>,
}

impl Names {
    fn uniquify(&mut self, base: &str) -> String {
        let mut name = base.to_string();
        while let Some(u) = self.usages.get_mut(&name) {
            let n = *u;
            *u += 1;
            name = format!("{base}_{n}");
        }
        self.usages.insert(name.clone(), 1);
        name
    }

    /// `PCheckerNameManager.ComputeNameForDecl` for a non-event, non-state declaration.
    fn decl(&mut self, name: &str) -> String {
        let name = if name.is_empty() { "Anon" } else { name };
        let name = if let Some(rest) = name.strip_prefix('$') {
            format!("TMP_{rest}")
        } else if RESERVED.contains(&name) {
            format!("P_{name}")
        } else {
            name.to_string()
        };
        self.uniquify(&name)
    }
}

/// The generated class name of every machine, indexed by `MachineId`.
///
/// Follows the generator's walk up to the machines: global parameters, global functions
/// (name, parameters, locals), then each machine in turn (name, fields, methods).
/// Declarations the type checker keeps distinct from machine names (functions, enums,
/// event sets, ...) cannot collide with them; they are still registered so that suffix
/// counts stay faithful.
pub fn machine_names(prog: &Program) -> Vec<String> {
    let mut names = Names::default();
    let mut vars: HashMap<VarId, ()> = HashMap::new();
    let mut var = |names: &mut Names, v: VarId| {
        if vars.insert(v, ()).is_none() {
            names.decl(&prog.var(v).name);
        }
    };
    for &v in &prog.global_params {
        var(&mut names, v);
    }
    for es in &prog.event_sets {
        names.decl(&es.name);
    }
    let fun = |names: &mut Names, var: &mut dyn FnMut(&mut Names, VarId), f: FunId| {
        let d = prog.fun(f);
        if d.role.foreign {
            return;
        }
        names.decl(&d.name);
        for &p in &d.params {
            var(names, p);
        }
        for &l in &d.locals {
            var(names, l);
        }
    };
    for (i, d) in prog.funs.iter().enumerate() {
        if d.owner.is_none() {
            fun(&mut names, &mut var, FunId(i as u32));
        }
    }
    let mut out = Vec::with_capacity(prog.machines.len());
    for (m, md) in prog.machines.iter().enumerate() {
        out.push(names.decl(&md.name));
        for &f in &md.fields {
            var(&mut names, f);
        }
        for (i, d) in prog.funs.iter().enumerate() {
            if d.owner.map(|o| o.idx()) == Some(m) {
                fun(&mut names, &mut var, FunId(i as u32));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::Names;

    #[test]
    fn uniquify() {
        let mut n = Names::default();
        assert_eq!(n.decl("client"), "client");
        assert_eq!(n.decl("client"), "client_1");
        assert_eq!(n.decl("client_1"), "client_1_1");
        assert_eq!(n.decl("client"), "client_2");
        assert_eq!(n.decl(""), "Anon");
        assert_eq!(n.decl("int"), "P_int");
        assert_eq!(n.decl("$tmp0"), "TMP_tmp0");
    }
}
