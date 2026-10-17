//! Value formatting of upstream's Java backend (`p compile --mode pobserve`), which PObserve
//! runs: `java.text.MessageFormat` for `format(...)`, the `toString()` of the Java classes
//! P values map to, and the iteration order of `java.util.HashMap`.
//!
//! P types map to Java as: `int` and machines to `long`, `float` to `float`, `string` to
//! `String`, enums to Java enums, `seq` to `ArrayList`, `set` to `LinkedHashSet`, `map` to
//! `HashMap`, and tuples to generated `PTuple_*` classes. Formatting therefore needs the
//! static type of a value (an enum value is just an integer at run time).

use std::fmt::Write;

use pnext_sema::{EventId, PType, Program};

use crate::value::Value;

/// Resolves type definitions at the top level of `ty`.
fn resolve<'a>(ty: &'a PType, prog: &'a Program) -> &'a PType {
    let mut t = ty;
    while let PType::TypeDef(id) = t {
        t = &prog.typedefs[id.0 as usize].ty;
    }
    t
}

// ---- java.lang hash codes and HashMap order ----------------------------------------------

/// `Long.hashCode`.
fn long_hash(v: i64) -> i32 {
    (v ^ ((v as u64) >> 32) as i64) as i32
}

/// `String.hashCode` (over UTF-16 code units).
pub fn string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for u in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(u as i32);
    }
    h
}

/// `hashCode()` of the Java object a value maps to. Enum constants hash by identity in
/// Java (different on every run); their ordinal value stands in for it.
pub fn java_hash(v: &Value) -> i32 {
    match v {
        Value::Null => 0,
        Value::Bool(b) => {
            if *b {
                1231
            } else {
                1237
            }
        }
        Value::Int(i) => long_hash(*i),
        Value::Float(f) => {
            let f = *f as f32;
            // Float.floatToIntBits: all NaNs collapse to the canonical one
            if f.is_nan() {
                0x7fc0_0000
            } else {
                f.to_bits() as i32
            }
        }
        Value::Str(s) => string_hash(s),
        Value::Machine(m) => long_hash(m.id as i64),
        Value::Event(_) => 0,
        Value::Tuple(t) => objects_hash(t.iter()),
        Value::NamedTuple(t) => objects_hash(t.values.iter()),
        Value::Seq(s) => objects_hash(s.iter()),
        Value::Set(s) => s.iter().fold(0i32, |h, v| h.wrapping_add(java_hash(v))),
        Value::Map(m) => m.iter().fold(0i32, |h, (k, v)| h.wrapping_add(java_hash(k) ^ java_hash(v))),
        Value::Foreign(f) => f.hash_value() as i32,
    }
}

/// `Objects.hash(...)` / `List.hashCode()`.
fn objects_hash<'a>(it: impl Iterator<Item = &'a Value>) -> i32 {
    it.fold(1i32, |h, v| h.wrapping_mul(31).wrapping_add(java_hash(v)))
}

/// The bucket order a `java.util.HashMap` holding `n` entries iterates in: keys sorted by
/// bucket (`(h ^ h >>> 16) & (capacity - 1)`), keeping insertion order within a bucket.
/// `keys` must be in insertion order. The table is assumed never to have held more
/// entries than it does now.
pub fn hashmap_order<'a, T>(items: impl IntoIterator<Item = (&'a Value, T)>) -> Vec<(&'a Value, T)> {
    let items: Vec<(&'a Value, T)> = items.into_iter().collect();
    let mut cap: usize = 16;
    while items.len() > cap * 3 / 4 {
        cap *= 2;
    }
    let mut keyed: Vec<(usize, (&Value, T))> = items
        .into_iter()
        .map(|(k, t)| {
            let h = java_hash(k);
            let spread = (h ^ ((h as u32) >> 16) as i32) as u32 as usize;
            (spread & (cap - 1), (k, t))
        })
        .collect();
    keyed.sort_by_key(|(b, _)| *b);
    keyed.into_iter().map(|(_, x)| x).collect()
}

/// The keys of a map in `HashMap` iteration order.
pub fn map_keys_in_java_order(m: &crate::value::SlotMap) -> Vec<Value> {
    hashmap_order(m.iter_insertion()).into_iter().map(|(k, _)| k.clone()).collect()
}

/// The values of a map in `HashMap` iteration order.
pub fn map_values_in_java_order(m: &crate::value::SlotMap) -> Vec<Value> {
    hashmap_order(m.iter_insertion()).into_iter().map(|(_, v)| v.clone()).collect()
}

// ---- generated class names ----------------------------------------------------------------

/// `NameManager.AbbreviateTupleName`.
fn abbreviate_tuple_name(name: &str) -> String {
    if name.chars().count() <= 5 {
        return name.to_string();
    }
    let uppers: String = name.chars().filter(|c| c.is_uppercase()).collect();
    if uppers.chars().count() >= 3 {
        return uppers.to_lowercase();
    }
    let mut name = name.to_string();
    // ^is([A-Z].*)
    if let Some(rest) = name.strip_prefix("is") {
        if rest.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            name = rest.to_string();
        }
    }
    if name.len() > 2 && name.ends_with("Id") {
        name.truncate(name.len() - 2);
    } else if name == "Id" {
        name.clear();
    }
    if name.ends_with("Val") {
        name.truncate(name.len() - 3);
    }
    if name.chars().count() <= 5 {
        return name;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    let mut s: String = first.to_string();
    s.extend(chars.filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')));
    if s.chars().count() <= 5 {
        return s.to_lowercase();
    }
    s.chars().take(5).collect::<String>().to_lowercase()
}

/// The abbreviated class name of a named tuple type, before uniquification.
pub fn tuple_class_name<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let parts: Vec<String> = names.map(abbreviate_tuple_name).collect();
    format!("PTuple_{}", parts.join("_"))
}

/// `PLanguageType.CanonicalRepresentation`: what makes two tuple types the same Java class.
fn canonical(ty: &PType, prog: &Program) -> String {
    match resolve(ty, prog) {
        PType::Bool => "bool".into(),
        PType::Int => "int".into(),
        PType::Float => "float".into(),
        PType::String => "string".into(),
        PType::Event => "event".into(),
        PType::Machine => "machine".into(),
        PType::Any => "any".into(),
        PType::Null => "null".into(),
        PType::Data => "data".into(),
        PType::Error => "<error>".into(),
        PType::Enum(id) => prog.enums[id.idx()].name.clone(),
        PType::Seq(t) => format!("seq[{}]", canonical(t, prog)),
        PType::Set(t) => format!("set[{}]", canonical(t, prog)),
        PType::Map(k, v) => format!("map[{},{}]", canonical(k, prog), canonical(v, prog)),
        PType::Tuple(ts) => {
            let fs: Vec<String> =
                ts.iter().enumerate().map(|(i, t)| format!("arg_{}:{}", i, canonical(t, prog))).collect();
            format!("({})", fs.join(","))
        }
        PType::NamedTuple(fs) => {
            let fs: Vec<String> = fs.iter().map(|(n, t)| format!("{}:{}", n, canonical(t, prog))).collect();
            format!("({})", fs.join(","))
        }
        PType::Permission(o) => match o {
            pnext_sema::PermissionOrigin::Machine(m) => prog.machines[m.idx()].name.clone(),
            pnext_sema::PermissionOrigin::Interface(i) => prog.interfaces[i.idx()].name.clone(),
            pnext_sema::PermissionOrigin::EventSet(e) => prog.event_sets[e.idx()].name.clone(),
        },
        PType::Foreign(n) => n.clone(),
        PType::TypeDef(_) => unreachable!(),
    }
}

/// Tuple types in the order upstream's type resolver registers them in a scope: inner tuples
/// before the tuple containing them. A typedef name refers to an already resolved type.
fn collect_tuples(ty: &PType, prog: &Program, out: &mut Vec<(String, Vec<String>)>) {
    match ty {
        PType::Seq(t) | PType::Set(t) => collect_tuples(t, prog, out),
        PType::Map(k, v) => {
            collect_tuples(k, prog, out);
            collect_tuples(v, prog, out);
        }
        PType::Tuple(ts) => {
            ts.iter().for_each(|t| collect_tuples(t, prog, out));
            out.push((canonical(ty, prog), (0..ts.len()).map(|i| format!("arg_{}", i)).collect()));
        }
        PType::NamedTuple(fs) => {
            fs.iter().for_each(|(_, t)| collect_tuples(t, prog, out));
            out.push((canonical(ty, prog), fs.iter().map(|(n, _)| n.clone()).collect()));
        }
        _ => {}
    }
}

/// Generated class names of the program's tuple types (`NameManager.NameForNamedTuple`):
/// the abbreviated name, plus `_1`, `_2`, ... for later distinct types that abbreviate to the
/// same name. Upstream names them in the order `TypesGenerator` declares them: tuples of the
/// global scope (typedefs, events, interfaces, in source order), then of global functions,
/// then of each machine (its fields, then its functions).
fn tuple_names(prog: &Program) -> &std::collections::HashMap<String, String> {
    prog.java_tuple_names.get_or_init(|| {
        let mut tuples = Vec::new();
        let mut global: Vec<(pnext_syntax::Span, &PType)> = Vec::new();
        global.extend(prog.typedefs.iter().map(|t| (t.span, &t.ty)));
        global.extend(prog.events.iter().filter(|e| !e.is_builtin()).map(|e| (e.span, &e.payload)));
        global.extend(prog.interfaces.iter().filter(|i| i.implicit_for.is_none()).map(|i| (i.span, &i.payload)));
        global.extend(prog.global_params.iter().map(|v| (prog.var(*v).span, &prog.var(*v).ty)));
        global.sort_by_key(|(s, _)| (s.file, s.start));
        for (_, t) in global {
            collect_tuples(t, prog, &mut tuples);
        }
        let fun_tuples = |f: &pnext_sema::FunDecl, out: &mut Vec<(String, Vec<String>)>| {
            for v in f.params.iter() {
                collect_tuples(&prog.var(*v).ty, prog, out);
            }
            collect_tuples(&f.ret, prog, out);
            for v in f.locals.iter() {
                collect_tuples(&prog.var(*v).ty, prog, out);
            }
        };
        for f in prog.funs.iter().filter(|f| f.owner.is_none() && f.parent.is_none()) {
            fun_tuples(f, &mut tuples);
        }
        for m in prog.machines.iter() {
            for v in m.fields.iter() {
                collect_tuples(&prog.var(*v).ty, prog, &mut tuples);
            }
            for f in m.methods.iter() {
                fun_tuples(prog.fun(*f), &mut tuples);
            }
        }
        // NameManagerBase.UniquifyName
        let mut usages: std::collections::HashMap<String, usize> = Default::default();
        let mut names = std::collections::HashMap::new();
        for (canon, fields) in tuples {
            if names.contains_key(&canon) {
                continue;
            }
            let base = tuple_class_name(fields.iter().map(|n| n.as_str()));
            let mut name = base.clone();
            while let Some(n) = usages.get_mut(&name) {
                let used = *n;
                *n += 1;
                name = format!("{}_{}", base, used);
            }
            usages.insert(name.clone(), 1);
            names.insert(canon, name);
        }
        names
    })
}

/// The generated Java class of the tuple type `ty` with field names `names`.
fn tuple_class<'a>(ty: &PType, prog: &Program, names: impl Iterator<Item = &'a str>) -> String {
    match tuple_names(prog).get(&canonical(ty, prog)) {
        Some(n) => n.clone(),
        None => tuple_class_name(names),
    }
}

// ---- toString ---------------------------------------------------------------------------

/// `String.valueOf(o)` of the Java object for `v` of static type `ty`.
pub fn to_string(v: &Value, ty: &PType, prog: &Program) -> String {
    let mut s = String::new();
    write_value(&mut s, v, ty, prog);
    s
}

fn write_value(out: &mut String, v: &Value, ty: &PType, prog: &Program) {
    let ty = resolve(ty, prog);
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => match ty {
            PType::Enum(id) => match prog.enums[id.idx()].elems.iter().find(|e| e.value == *i) {
                Some(e) => out.push_str(&e.name),
                None => write!(out, "{}", i).unwrap(),
            },
            _ => write!(out, "{}", i).unwrap(),
        },
        Value::Float(f) => out.push_str(&float_to_string(*f as f32)),
        Value::Str(s) => out.push_str(s),
        Value::Machine(m) => write!(out, "{}", m.id).unwrap(),
        Value::Event(e) => out.push_str(&prog.event(*e).name),
        Value::Foreign(f) => out.push_str(&f.display()),
        Value::Seq(s) => {
            let elem = match ty {
                PType::Seq(t) => t.as_ref(),
                _ => &PType::Any,
            };
            write_list(out, s.iter(), elem, prog);
        }
        Value::Set(s) => {
            let elem = match ty {
                PType::Set(t) => t.as_ref(),
                _ => &PType::Any,
            };
            write_list(out, s.iter_insertion(), elem, prog);
        }
        Value::Map(m) => {
            let (kt, vt) = match ty {
                PType::Map(k, v) => (k.as_ref(), v.as_ref()),
                _ => (&PType::Any, &PType::Any),
            };
            out.push('{');
            let entries = hashmap_order(m.iter_insertion());
            for (i, (k, v)) in entries.into_iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(out, k, kt, prog);
                out.push('=');
                write_value(out, v, vt, prog);
            }
            out.push('}');
        }
        Value::NamedTuple(t) => {
            let types: Vec<PType> = match ty {
                PType::NamedTuple(fs) => fs.iter().map(|(_, t)| t.clone()).collect(),
                _ => vec![PType::Any; t.values.len()],
            };
            out.push_str(&tuple_class(ty, prog, t.names.iter().map(|n| n.as_str())));
            out.push('[');
            for (i, (n, v)) in t.names.iter().zip(t.values.iter()).enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(n);
                out.push('=');
                write_value(out, v, types.get(i).unwrap_or(&PType::Any), prog);
            }
            out.push(']');
        }
        Value::Tuple(t) => {
            // unnamed tuples become named tuples with fields arg_0, arg_1, ...
            let types: Vec<PType> = match ty {
                PType::Tuple(ts) => ts.clone(),
                _ => vec![PType::Any; t.len()],
            };
            let names: Vec<String> = (0..t.len()).map(|i| format!("arg_{}", i)).collect();
            out.push_str(&tuple_class(ty, prog, names.iter().map(|n| n.as_str())));
            out.push('[');
            for (i, v) in t.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write!(out, "arg_{}=", i).unwrap();
                write_value(out, v, types.get(i).unwrap_or(&PType::Any), prog);
            }
            out.push(']');
        }
    }
}

fn write_list<'a>(out: &mut String, it: impl Iterator<Item = &'a Value>, elem: &PType, prog: &Program) {
    out.push('[');
    for (i, v) in it.enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write_value(out, v, elem, prog);
    }
    out.push(']');
}

/// `toString()` of a generated event class: `e[payload]`, or `e` without a payload.
pub fn event_to_string(prog: &Program, e: EventId, payload: &Value) -> String {
    let d = prog.event(e);
    if matches!(resolve(&d.payload, prog), PType::Null) {
        d.name.clone()
    } else {
        format!("{}[{}]", d.name, to_string(payload, &d.payload, prog))
    }
}

// ---- java.text.MessageFormat --------------------------------------------------------------

/// How `MessageFormat` renders one argument: numbers through `NumberFormat.getInstance()`
/// (grouping, at most three fraction digits), strings as they are, anything else through
/// `toString()`.
pub fn format_arg(v: &Value, ty: &PType, prog: &Program) -> String {
    match v {
        Value::Int(i) if !matches!(resolve(ty, prog), PType::Enum(_)) => {
            group_digits(&i.unsigned_abs().to_string(), *i < 0)
        }
        Value::Machine(m) => group_digits(&m.id.to_string(), false),
        Value::Float(f) => number_format_float(*f as f32),
        _ => to_string(v, ty, prog),
    }
}

fn group_digits(digits: &str, negative: bool) -> String {
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    let n = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (n - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `NumberFormat.getInstance().format(f)` (pattern `#,##0.###`, `HALF_EVEN`).
fn number_format_float(f: f32) -> String {
    let d = f as f64;
    if d.is_nan() {
        return "NaN".into();
    }
    if d.is_infinite() {
        return if d > 0.0 { "\u{221e}".into() } else { "-\u{221e}".into() };
    }
    // the exact decimal value of the float, then HALF_EVEN to three fraction digits
    let exact = format!("{:.160}", d.abs());
    let (int_part, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut digits: Vec<u8> = int_part.bytes().map(|b| b - b'0').collect();
    let int_len = digits.len();
    let frac_digits: Vec<u8> = frac.bytes().map(|b| b - b'0').collect();
    digits.extend_from_slice(&frac_digits[..3.min(frac_digits.len())]);
    let rest = &frac_digits[3.min(frac_digits.len())..];
    let round_up = match rest.first() {
        Some(&r) if r > 5 => true,
        Some(&5) => rest[1..].iter().any(|&x| x != 0) || digits.last().is_some_and(|l| l % 2 == 1),
        _ => false,
    };
    let mut int_len = int_len;
    if round_up {
        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, 1);
                int_len += 1;
                break;
            }
            i -= 1;
            if digits[i] == 9 {
                digits[i] = 0;
            } else {
                digits[i] += 1;
                break;
            }
        }
    }
    let int_str: String = digits[..int_len].iter().map(|d| (b'0' + d) as char).collect();
    let int_str = int_str.trim_start_matches('0');
    let int_str = if int_str.is_empty() { "0" } else { int_str };
    let mut frac_str: String = digits[int_len..].iter().map(|d| (b'0' + d) as char).collect();
    while frac_str.ends_with('0') {
        frac_str.pop();
    }
    let is_zero = int_str == "0" && frac_str.is_empty();
    let mut out = group_digits(int_str, d < 0.0 && !is_zero);
    if d.is_sign_negative() && is_zero && d != 0.0 {
        out = format!("-{}", out);
    }
    if !frac_str.is_empty() {
        out.push('.');
        out.push_str(&frac_str);
    }
    out
}

/// `MessageFormat.format(pattern, args)` for P's `{N}` placeholders. `Err` carries the
/// message of the `IllegalArgumentException` a malformed pattern throws.
pub fn message_format(pattern: &str, args: &[String]) -> Result<String, String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut in_quote = false;
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\'' {
            if i + 1 < chars.len() && chars[i + 1] == '\'' {
                out.push('\'');
                i += 2;
                continue;
            }
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if ch == '{' && !in_quote {
            // argument: {index[,type[,style]]}
            let mut segments = vec![String::new()];
            let mut depth = 0;
            let mut quoted = false;
            let mut j = i + 1;
            let mut closed = false;
            while j < chars.len() {
                let c = chars[j];
                if quoted {
                    segments.last_mut().unwrap().push(c);
                    if c == '\'' {
                        quoted = false;
                    }
                } else {
                    match c {
                        ',' if segments.len() < 3 => segments.push(String::new()),
                        '{' => {
                            depth += 1;
                            segments.last_mut().unwrap().push(c);
                        }
                        '}' if depth == 0 => {
                            closed = true;
                            break;
                        }
                        '}' => {
                            depth -= 1;
                            segments.last_mut().unwrap().push(c);
                        }
                        ' ' if segments.len() == 2 && segments[1].is_empty() => {}
                        '\'' => {
                            quoted = true;
                            segments.last_mut().unwrap().push(c);
                        }
                        _ => segments.last_mut().unwrap().push(c),
                    }
                }
                j += 1;
            }
            if !closed {
                return Err("Unmatched braces in the pattern.".into());
            }
            let index_text = segments[0].trim();
            let index: i64 = match index_text.parse::<i64>() {
                Ok(n) if !index_text.starts_with('+') => n,
                _ => return Err(format!("can't parse argument number: {}", segments[0])),
            };
            if index < 0 {
                return Err(format!("negative argument number: {}", index));
            }
            match args.get(index as usize) {
                Some(a) => out.push_str(a),
                None => write!(out, "{{{}}}", index).unwrap(),
            }
            i = j + 1;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    Ok(out)
}

// ---- Float.toString / Double.toString ----------------------------------------------------

/// `Float.toString(f)`.
pub fn float_to_string(f: f32) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    java_decimal(
        &format!("{:e}", f),
        f == 0.0,
        f.is_sign_negative(),
        (f.abs() as f64) >= 1e-3 && (f.abs() as f64) < 1e7,
    )
}

/// `Double.toString(d)`.
pub fn double_to_string(d: f64) -> String {
    if d.is_nan() {
        return "NaN".into();
    }
    if d.is_infinite() {
        return if d > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    java_decimal(&format!("{:e}", d), d == 0.0, d.is_sign_negative(), d.abs() >= 1e-3 && d.abs() < 1e7)
}

/// Renders Rust's shortest scientific representation (`1.2345e7`) in Java's style.
fn java_decimal(sci: &str, zero: bool, negative: bool, plain: bool) -> String {
    let sign = if negative { "-" } else { "" };
    if zero {
        return format!("{}0.0", sign);
    }
    let sci = sci.trim_start_matches('-');
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    if plain {
        let point = exp + 1; // digits before the decimal point
        let (int, frac) = if point <= 0 {
            ("0".to_string(), format!("{}{}", "0".repeat((-point) as usize), digits))
        } else if point as usize >= digits.len() {
            (format!("{}{}", digits, "0".repeat(point as usize - digits.len())), String::new())
        } else {
            (digits[..point as usize].to_string(), digits[point as usize..].to_string())
        };
        let frac = if frac.is_empty() { "0".to_string() } else { frac };
        format!("{}{}.{}", sign, int, frac)
    } else {
        let frac = if digits.len() > 1 { &digits[1..] } else { "0" };
        format!("{}{}.{}E{}", sign, &digits[..1], frac, exp)
    }
}

// ---- java.time.Instant -------------------------------------------------------------------

/// Civil date of a day count since 1970-01-01 (proleptic Gregorian).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn date_time(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, d, rem / 3600, rem % 3600 / 60, rem % 60)
}

/// An instant with `DateTimeFormatter`'s `appendInstant(9)`: always nine fraction digits.
pub fn instant_nanos9(secs: i64, nanos: u32) -> String {
    format!("{}.{:09}Z", date_time(secs), nanos)
}

/// `Instant.toString()`: the fraction in groups of three digits, omitted when zero.
pub fn instant_to_string(secs: i64, nanos: u32) -> String {
    let frac = if nanos == 0 {
        String::new()
    } else if nanos.is_multiple_of(1_000_000) {
        format!(".{:03}", nanos / 1_000_000)
    } else if nanos.is_multiple_of(1000) {
        format!(".{:06}", nanos / 1000)
    } else {
        format!(".{:09}", nanos)
    };
    format!("{}{}Z", date_time(secs), frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_print_like_java() {
        assert_eq!(float_to_string(1.0), "1.0");
        assert_eq!(float_to_string(2.5), "2.5");
        assert_eq!(float_to_string(0.0001), "1.0E-4");
        assert_eq!(float_to_string(12345678.9), "1.2345679E7");
        assert_eq!(float_to_string(-0.0), "-0.0");
        assert_eq!(float_to_string(100.0), "100.0");
        assert_eq!(float_to_string(0.001), "0.001");
        assert_eq!(float_to_string(3.8333333E-4), "3.8333333E-4");
        assert_eq!(float_to_string(5.0E-4), "5.0E-4");
        assert_eq!(double_to_string(1e7), "1.0E7");
        assert_eq!(double_to_string(9999999.0), "9999999.0");
    }

    #[test]
    fn number_format_matches_java() {
        assert_eq!(number_format_float(1234.5678), "1,234.568");
        assert_eq!(number_format_float(0.5), "0.5");
        assert_eq!(number_format_float(2.0), "2");
        assert_eq!(number_format_float(1.0005), "1");
        assert_eq!(number_format_float(0.0), "0");
        assert_eq!(number_format_float(-1234567.0), "-1,234,567");
        assert_eq!(group_digits("1234567", false), "1,234,567");
        assert_eq!(group_digits("123", true), "-123");
    }

    #[test]
    fn message_format_quotes_and_missing_args() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(message_format("isn't {0} '{1}'", &a(&["x", "y"])).unwrap(), "isnt {0} y");
        assert_eq!(message_format("a '' b {1}", &a(&["x"])).unwrap(), "a ' b {1}");
        assert_eq!(message_format("{0} and {0}", &a(&["2"])).unwrap(), "2 and 2");
        assert!(message_format("{0", &a(&["x"])).is_err());
        assert_eq!(message_format("{x}", &a(&["x"])).unwrap_err(), "can't parse argument number: x");
    }

    #[test]
    fn tuple_names_are_abbreviated() {
        let n = |v: &[&str]| tuple_class_name(v.iter().copied());
        assert_eq!(n(&["accountId", "amount", "rId"]), "PTuple_accnt_amnt_rId");
        assert_eq!(n(&["status", "accountId", "balance", "rId"]), "PTuple_stts_accnt_blnc_rId");
        assert_eq!(n(&["client", "lockId", "rId"]), "PTuple_clnt_lock_rId");
        assert_eq!(
            n(&["timestamp", "key", "node1Count", "node2Count", "totalValues"]),
            "PTuple_tmstm_key_nd1cn_nd2cn_ttlvl"
        );
        assert_eq!(n(&["first", "second"]), "PTuple_first_scnd");
        assert_eq!(n(&["arg_0", "arg_1"]), "PTuple_arg_0_arg_1");
    }

    #[test]
    fn hashmap_order_matches_java() {
        let keys = [Value::Int(3), Value::Int(1), Value::Int(17), Value::Int(-2)];
        let order: Vec<i64> =
            hashmap_order(keys.iter().map(|k| (k, ()))).into_iter().map(|(k, _)| k.as_int()).collect();
        assert_eq!(order, vec![1, 17, -2, 3]);
        let keys = [Value::str("b"), Value::str("a"), Value::str("zz"), Value::str("hello")];
        let order: Vec<String> = hashmap_order(keys.iter().map(|k| (k, ())))
            .into_iter()
            .map(|(k, _)| match k {
                Value::Str(s) => s.to_string(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(order, vec!["zz", "a", "b", "hello"]);
    }

    #[test]
    fn instants() {
        assert_eq!(instant_nanos9(1706894854, 309_000_000), "2024-02-02T17:27:34.309000000Z");
        assert_eq!(instant_to_string(-1, 999_000_000), "1969-12-31T23:59:59.999Z");
        assert_eq!(instant_to_string(0, 0), "1970-01-01T00:00:00Z");
    }
}
