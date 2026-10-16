//! Constructors for every diagnostic message, copied from `DefaultTranslationErrorHandler`.

use crate::diag::Diagnostic;
use crate::program::*;
use crate::types::{PType, TypeKind};
use pnext_syntax::{SourceMap, Span};

/// `FormatType` upstream: the "null" type is shown as "(no value)".
pub fn format_type(t: &PType, prog: &Program) -> String {
    let rep = t.original_repr(prog);
    if rep == "null" {
        "(no value)".to_string()
    } else {
        rep
    }
}

pub struct Errors<'a> {
    pub sm: &'a SourceMap,
}

impl<'a> Errors<'a> {
    fn loc(&self, span: Span) -> String {
        self.sm.location(span).to_string()
    }

    pub fn duplicate_start_state(&self, span: Span, dup: &str, orig: &str, machine: &str) -> Diagnostic {
        Diagnostic::new(span, format!("state '{}' tries to replace start state '{}' in machine '{}'", dup, orig, machine))
    }

    pub fn duplicate_event_action(&self, span: Span, existing: Span, state: &str) -> Diagnostic {
        Diagnostic::new(span, format!("event handler in state '{}' duplicates handler at {}", state, self.loc(existing)))
    }

    pub fn duplicate_state_exit(&self, span: Span, existing: Span, state: &str) -> Diagnostic {
        Diagnostic::new(span, format!("exit handler in state '{}' duplicates exit handler at {}", state, self.loc(existing)))
    }

    pub fn duplicate_state_entry(&self, span: Span, existing: Span, state: &str) -> Diagnostic {
        Diagnostic::new(span, format!("entry handler in state '{}' duplicates entry handler at {}", state, self.loc(existing)))
    }

    pub fn duplicate_declaration(&self, span: Span, dup: &str, existing: &str, existing_span: Span) -> Diagnostic {
        Diagnostic::new(span, format!("'{}' duplicates declaration '{}' at {}", dup, existing, self.loc(existing_span)))
    }

    pub fn redeclare_global_param(&self, span: Span, dup: &str, existing: &str, existing_span: Span) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("'{}' redeclares global param '{}' (originally declared at {})", dup, existing, self.loc(existing_span)),
        )
    }

    pub fn undeclared_global_param(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(span, format!("undeclared global param '{}'", name))
    }

    pub fn modify_global_param(&self, span: Span, name: &str, decl_span: Span) -> Diagnostic {
        Diagnostic::new(span, format!("cannot modify global param '{}' (declared at {})", name, self.loc(decl_span)))
    }

    pub fn invalid_twise(&self, span: Span, test_span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid twise number at {}: {}", self.loc(test_span), msg))
    }

    pub fn incorrect_argument_count(&self, span: Span, actual: usize, expected: usize) -> Diagnostic {
        Diagnostic::new(span, format!("goto, function or constructor call expected {} arguments, got {}", expected, actual))
    }

    pub fn missing_declaration(&self, span: Span, kind: &str, name: &str) -> Diagnostic {
        Diagnostic::new(span, format!("could not find {} '{}'", kind, name))
    }

    pub fn circular_typedef(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(span, format!("typedef '{}' has a circular dependency", name))
    }

    pub fn duplicate_named_tuple_entry(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(span, format!("named tuple has duplicate field '{}'", name))
    }

    pub fn illegal_type_in_coerce(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "coerce expects an interface, int, or float type")
    }

    pub fn illegal_interface_coerce(&self, span: Span, old: &str, new: &str, outlier: &str) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("illegal coerce: permissions of '{}' are not a superset of '{}' (e.g., event '{}')", old, new, outlier),
        )
    }

    pub fn static_function_not_allowed_as_handler(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("function '{}' cannot be used as a handler: global and foreign functions are not allowed as handlers", name),
        )
    }

    pub fn type_mismatch(&self, span: Span, actual: &PType, expected: &[&PType], prog: &Program) -> Diagnostic {
        Diagnostic::new(
            span,
            format!(
                "got type: {}, expected: {}",
                format_type(actual, prog),
                expected.iter().map(|t| format_type(t, prog)).collect::<Vec<_>>().join("; ")
            ),
        )
    }

    pub fn type_mismatch_kinds(&self, span: Span, actual: &PType, expected: &[TypeKind], prog: &Program) -> Diagnostic {
        Diagnostic::new(
            span,
            format!(
                "got type: {}, expected: {}",
                format_type(actual, prog),
                expected.iter().map(|k| k.name()).collect::<Vec<_>>().join(", ")
            ),
        )
    }

    pub fn missing_named_tuple_entry(&self, span: Span, tuple: &PType, field: &str, prog: &Program) -> Diagnostic {
        Diagnostic::new(span, format!("named tuple type {} has no '{}' field", tuple.original_repr(prog), field))
    }

    pub fn out_of_bounds_tuple_access(&self, span: Span, tuple: &PType, index: &str, prog: &Program) -> Diagnostic {
        Diagnostic::new(span, format!("tuple type {} has no '{}' field", tuple.original_repr(prog), index))
    }

    pub fn incomparable_types(&self, span: Span, lhs: &PType, rhs: &PType, prog: &Program) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("types {} and {} are incomparable", lhs.original_repr(prog), rhs.original_repr(prog)),
        )
    }

    pub fn misplaced_this(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "keyword 'this' used outside a machine")
    }

    pub fn binop_type_mismatch(&self, op_span: Span, op: &str, lhs: &PType, rhs: &PType, prog: &Program) -> Diagnostic {
        Diagnostic::new(
            op_span,
            format!(
                "operator '{}' requires both operands to be int or both float; got {} and {}",
                op,
                lhs.original_repr(prog),
                rhs.original_repr(prog)
            ),
        )
    }

    pub fn more_than_one_parameter_for_handlers(&self, span: Span, count: usize) -> Diagnostic {
        Diagnostic::new(span, format!("entry, exit, do, and goto handlers take at most 1 parameter; function expects {}", count))
    }

    pub fn illegal_choose_sub_expr_type(&self, span: Span, t: &PType, prog: &Program) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("'choose' expects int (max value) or a collection (seq, set, or map); got type {}", t.original_repr(prog)),
        )
    }

    pub fn illegal_choose_sub_expr_value(&self, span: Span, n: i64) -> Diagnostic {
        Diagnostic::new(span, format!("'choose' expects at most 10000 choices; got {}", n))
    }

    pub fn illegal_function_used_in_spec(&self, span: Span, fun: &str, monitor: &str) -> Diagnostic {
        Diagnostic::new(
            span,
            format!(
                "function '{}' is non-deterministic or has side effects (new/send/receive/announce); cannot be called from spec monitor '{}'",
                fun, monitor
            ),
        )
    }

    pub fn emitted_null_event(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "cannot send null events")
    }

    pub fn missing_start_state(&self, span: Span, machine: &str) -> Diagnostic {
        Diagnostic::new(span, format!("machine '{}' has no start state", machine))
    }

    pub fn changed_state_mid_transition(&self, span: Span, fun: &str) -> Diagnostic {
        Diagnostic::new(span, format!("function '{}' is used as a transition handler but might change state here", fun))
    }

    pub fn invalid_print_format(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "print format placeholders must contain only digits; escape braces by doubling them")
    }

    pub fn invalid_string_expr_format(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "string format placeholders must contain only digits; escape braces by doubling them")
    }

    pub fn invalid_bind(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid bind: {}", msg))
    }

    pub fn not_closed(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("module not closed: {}", msg))
    }

    pub fn no_main(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("illegal main machine: {}", msg))
    }

    pub fn invalid_assert_event(&self, span: Span, monitor: &str, event: &str) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("invalid assert: event '{}' in observes set of '{}' is not in the sends set of the module", event, monitor),
        )
    }

    pub fn invalid_assert_monitor(&self, span: Span, monitor: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid assert: spec monitor '{}' is already attached in the module", monitor))
    }

    pub fn invalid_hide_event(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid hide event: {}", msg))
    }

    pub fn invalid_hide_interface(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid hide interface: {}", msg))
    }

    pub fn invalid_rename(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid rename: {}", msg))
    }

    pub fn invalid_composition(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("invalid composition: {}", msg))
    }

    pub fn internal_error(&self, span: Span, msg: &str) -> Diagnostic {
        Diagnostic::new(span, format!("internal compiler error (please report): {}", msg))
    }

    pub fn two_start_states(&self, span: Span, machine: &str, first: &str, second: &str) -> Diagnostic {
        Diagnostic::new(span, format!("machine '{}' has two start states: '{}' and '{}'", machine, first, second))
    }

    pub fn value_out_of_range(&self, span: Span, ty: &str) -> Diagnostic {
        Diagnostic::new(span, format!("value is out of range for type '{}'", ty))
    }

    pub fn null_transition_in_monitor(&self, span: Span, monitor: &str) -> Diagnostic {
        Diagnostic::new(span, format!("cannot transition on null event in spec monitor '{}'", monitor))
    }

    pub fn illegal_monitor_operation(&self, span: Span, monitor: &str) -> Diagnostic {
        Diagnostic::new(
            span,
            format!("'$', '$$', 'this', 'new', 'send', 'announce', 'receive', and 'pop' are not allowed in spec monitor '{}'", monitor),
        )
    }

    pub fn deferred_event_in_monitor(&self, span: Span, monitor: &str) -> Diagnostic {
        Diagnostic::new(span, format!("events cannot be deferred in spec monitor '{}'", monitor))
    }

    pub fn not_all_paths_return(&self, span: Span, fun: &str) -> Diagnostic {
        Diagnostic::new(span, format!("not all paths in function '{}' return a value", fun))
    }

    pub fn expected_monitor(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(span, format!("expected a spec monitor; got '{}'", name))
    }

    pub fn raise_in_non_void(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "'raise' is only allowed in functions that do not return a value")
    }

    pub fn change_state_in_non_void(&self, span: Span) -> Diagnostic {
        Diagnostic::new(span, "state changes are only allowed in functions that do not return a value")
    }

    pub fn duplicate_receive_case(&self, span: Span, event: &str) -> Diagnostic {
        Diagnostic::new(span, format!("event '{}' appears twice in receive statement", event))
    }

    pub fn bare_loop_control_flow(&self, span: Span, stmt: &str) -> Diagnostic {
        Diagnostic::new(span, format!("'{}' cannot appear outside a loop body", stmt))
    }

    pub fn exit_function_cannot_take_parameters(&self, span: Span, count: usize) -> Diagnostic {
        Diagnostic::new(span, format!("exit handlers cannot take parameters; function expects {}", count))
    }

    pub fn duplicate_bindings(&self, span: Span, name: &str) -> Diagnostic {
        Diagnostic::new(span, format!("interface or machine '{}' is mentioned or bound multiple times in the module", name))
    }

    pub fn spec_observes_incomplete_warning(&self, span: Span, event: &str, machine: &str) -> String {
        format!(
            "[Warning:] [{}] event '{}' is not in the observes list of spec monitor '{}'; the handler will never be triggered",
            self.loc(span),
            event,
            machine
        )
    }
}
