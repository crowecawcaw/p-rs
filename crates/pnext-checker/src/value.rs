//! Runtime values of P programs with the formatting and equality rules of the upstream
//! checker's `PChecker.Runtime.Values` classes.
//!
//! Values have copy semantics in P. Containers are reference counted and copied on write,
//! which makes assignment cheap while keeping the semantics.

use std::collections::HashMap;
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use pnext_sema::{EventId, PType, Program};

/// A reference to a machine instance as seen by P code (`PMachineValue`).
#[derive(Debug, Clone)]
pub struct MachineRef {
    /// Operation id of the machine (`StateMachineId.Value`).
    pub id: u64,
    /// `Name(id)`, the display name of the machine.
    pub name: Rc<str>,
    /// Events this reference may be sent (interface permissions), sorted.
    pub permissions: Rc<[EventId]>,
}

#[derive(Debug, Clone)]
pub struct NamedTuple {
    pub names: Rc<[String]>,
    pub values: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    /// Integers and enum values (`PInt`).
    Int(i64),
    Float(f64),
    Str(Rc<str>),
    Machine(Rc<MachineRef>),
    Event(EventId),
    Tuple(Rc<Vec<Value>>),
    NamedTuple(Rc<NamedTuple>),
    Seq(Rc<Vec<Value>>),
    Set(Rc<SlotSet>),
    Map(Rc<SlotMap>),
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

/// `PValues.SafeEquals`.
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Machine(a), Value::Machine(b)) => a.id == b.id,
            (Value::Event(a), Value::Event(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y),
            (Value::NamedTuple(a), Value::NamedTuple(b)) => {
                a.values.len() == b.values.len()
                    && a.names.iter().zip(b.names.iter()).all(|(x, y)| x == y)
                    && a.values.iter().zip(b.values.iter()).all(|(x, y)| x == y)
            }
            (Value::Seq(a), Value::Seq(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y),
            (Value::Set(a), Value::Set(b)) => a.len() == b.len() && a.iter().all(|x| b.contains(x)),
            (Value::Map(a), Value::Map(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).map(|w| w == v).unwrap_or(false))
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0u8.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Float(f) => {
                let f = if *f == 0.0 { 0.0 } else { *f };
                f.to_bits().hash(state)
            }
            Value::Str(s) => s.hash(state),
            Value::Machine(m) => m.id.hash(state),
            Value::Event(e) => e.hash(state),
            Value::Tuple(t) => {
                for v in t.iter() {
                    v.hash(state);
                }
            }
            Value::NamedTuple(t) => {
                for v in t.values.iter() {
                    v.hash(state);
                }
            }
            Value::Seq(s) => {
                for v in s.iter() {
                    v.hash(state);
                }
            }
            // order-independent hashes for sets and maps
            Value::Set(s) => {
                let mut h: u64 = 0;
                for v in s.iter() {
                    let mut hh = std::collections::hash_map::DefaultHasher::new();
                    v.hash(&mut hh);
                    h = h.wrapping_add(hh.finish());
                }
                h.hash(state)
            }
            Value::Map(m) => {
                let mut h: u64 = 0;
                for (k, _) in m.iter() {
                    let mut hh = std::collections::hash_map::DefaultHasher::new();
                    k.hash(&mut hh);
                    h = h.wrapping_add(hh.finish());
                }
                h.hash(state)
            }
        }
    }
}

/// A hash set that enumerates in the order .NET's `HashSet<T>` does: entries live in slots
/// in insertion order, and removed slots are reused (most recently freed first).
#[derive(Debug, Clone, Default)]
pub struct SlotSet {
    slots: Vec<Option<Value>>,
    index: HashMap<Value, usize>,
    free: Vec<usize>,
}

impl SlotSet {
    pub fn new() -> SlotSet {
        SlotSet::default()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn contains(&self, v: &Value) -> bool {
        self.index.contains_key(v)
    }

    /// Adds `v`; returns false if it was already present.
    pub fn insert(&mut self, v: Value) -> bool {
        if self.index.contains_key(&v) {
            return false;
        }
        let slot = match self.free.pop() {
            Some(s) => {
                self.slots[s] = Some(v.clone());
                s
            }
            None => {
                self.slots.push(Some(v.clone()));
                self.slots.len() - 1
            }
        };
        self.index.insert(v, slot);
        true
    }

    pub fn remove(&mut self, v: &Value) -> bool {
        match self.index.remove(v) {
            Some(slot) => {
                self.slots[slot] = None;
                self.free.push(slot);
                true
            }
            None => false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.slots.iter().filter_map(|s| s.as_ref())
    }

    /// `Enumerable.ElementAt(index)` over the enumeration order.
    pub fn element_at(&self, index: usize) -> Option<&Value> {
        self.iter().nth(index)
    }
}

/// A map with .NET `Dictionary<K,V>` enumeration order (see [`SlotSet`]).
#[derive(Debug, Clone, Default)]
pub struct SlotMap {
    slots: Vec<Option<(Value, Value)>>,
    index: HashMap<Value, usize>,
    free: Vec<usize>,
}

impl SlotMap {
    pub fn new() -> SlotMap {
        SlotMap::default()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn contains_key(&self, k: &Value) -> bool {
        self.index.contains_key(k)
    }

    pub fn get(&self, k: &Value) -> Option<&Value> {
        self.index.get(k).and_then(|&s| self.slots[s].as_ref()).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, k: &Value) -> Option<&mut Value> {
        match self.index.get(k) {
            Some(&s) => self.slots[s].as_mut().map(|(_, v)| v),
            None => None,
        }
    }

    /// `dict[k] = v`: overwrites in place or adds a new entry.
    pub fn set(&mut self, k: Value, v: Value) {
        if let Some(&s) = self.index.get(&k) {
            self.slots[s] = Some((k, v));
            return;
        }
        self.add_new(k, v);
    }

    /// `dict.Add(k, v)`: returns false (and does nothing) if the key exists.
    pub fn add(&mut self, k: Value, v: Value) -> bool {
        if self.index.contains_key(&k) {
            return false;
        }
        self.add_new(k, v);
        true
    }

    fn add_new(&mut self, k: Value, v: Value) {
        let slot = match self.free.pop() {
            Some(s) => {
                self.slots[s] = Some((k.clone(), v));
                s
            }
            None => {
                self.slots.push(Some((k.clone(), v)));
                self.slots.len() - 1
            }
        };
        self.index.insert(k, slot);
    }

    pub fn remove(&mut self, k: &Value) -> bool {
        match self.index.remove(k) {
            Some(slot) => {
                self.slots[slot] = None;
                self.free.push(slot);
                true
            }
            None => false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.slots.iter().filter_map(|s| s.as_ref()).map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &Value> {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.iter().map(|(_, v)| v)
    }

    pub fn key_at(&self, index: usize) -> Option<&Value> {
        self.keys().nth(index)
    }
}

impl Value {
    pub fn str(s: &str) -> Value {
        Value::Str(Rc::from(s))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false,
        }
    }

    pub fn as_int(&self) -> i64 {
        match self {
            Value::Int(i) => *i,
            Value::Float(f) => *f as i64,
            Value::Bool(b) => *b as i64,
            _ => 0,
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            Value::Float(f) => *f,
            Value::Int(i) => *i as f64,
            _ => 0.0,
        }
    }

    /// The .NET type name of the value's class, for exception messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "PBool",
            Value::Int(_) => "PInt",
            Value::Float(_) => "PFloat",
            Value::Str(_) => "PString",
            Value::Machine(_) => "PMachineValue",
            Value::Event(_) => "Event",
            Value::Tuple(_) => "PTuple",
            Value::NamedTuple(_) => "PNamedTuple",
            Value::Seq(_) => "PSeq",
            Value::Set(_) => "PSet",
            Value::Map(_) => "PMap",
        }
    }

    /// Number of elements of a collection (`Count`).
    pub fn count(&self) -> usize {
        match self {
            Value::Seq(s) => s.len(),
            Value::Set(s) => s.len(),
            Value::Map(m) => m.len(),
            Value::Tuple(t) => t.len(),
            Value::NamedTuple(t) => t.values.len(),
            Value::Str(s) => s.len(),
            _ => 0,
        }
    }

    /// `IPValue.ToString()` (null prints as the empty string, as in .NET string concatenation).
    pub fn to_display(&self, prog: &Program) -> String {
        let mut s = String::new();
        self.write_display(&mut s, prog);
        s
    }

    fn write_display(&self, out: &mut String, prog: &Program) {
        match self {
            Value::Null => {}
            Value::Bool(b) => out.push_str(if *b { "True" } else { "False" }),
            Value::Int(i) => write!(out, "{}", i).unwrap(),
            Value::Float(f) => out.push_str(&format_double(*f)),
            Value::Str(s) => out.push_str(s),
            Value::Machine(m) => out.push_str(&m.name),
            Value::Event(e) => out.push_str(event_cs_name(prog, *e)),
            Value::Tuple(t) => {
                out.push('<');
                for v in t.iter() {
                    v.write_display(out, prog);
                    out.push(',');
                }
                out.push('>');
            }
            Value::NamedTuple(t) => {
                out.push('<');
                for (n, v) in t.names.iter().zip(t.values.iter()) {
                    out.push_str(n);
                    out.push(':');
                    v.write_display(out, prog);
                    out.push_str(", ");
                }
                out.push('>');
            }
            Value::Seq(s) => {
                out.push('(');
                for (i, v) in s.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    v.write_display(out, prog);
                }
                out.push(')');
            }
            Value::Set(s) => {
                let mut items: Vec<String> = s.iter().map(|v| v.to_display(prog)).collect();
                items.sort();
                out.push('(');
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push('<');
                    out.push_str(v);
                    out.push('>');
                }
                out.push(')');
            }
            Value::Map(m) => {
                let mut items: Vec<(String, String)> =
                    m.iter().map(|(k, v)| (k.to_display(prog), v.to_display(prog))).collect();
                items.sort_by(|a, b| a.0.cmp(&b.0));
                out.push('(');
                for (i, (k, v)) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push('<');
                    out.push_str(k);
                    out.push_str("->");
                    out.push_str(v);
                    out.push('>');
                }
                out.push(')');
            }
        }
    }

    /// `IPValue.ToEscapedString()`: strings quoted, nested nulls as `null`, containers in
    /// enumeration order.
    pub fn to_escaped(&self, prog: &Program) -> String {
        let mut s = String::new();
        self.write_escaped(&mut s, prog);
        s
    }

    fn write_escaped(&self, out: &mut String, prog: &Program) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Str(s) => {
                out.push('"');
                out.push_str(&s.replace('"', "\\\""));
                out.push('"');
            }
            Value::Tuple(t) => {
                out.push('<');
                for v in t.iter() {
                    v.write_escaped(out, prog);
                    out.push(',');
                }
                out.push('>');
            }
            Value::NamedTuple(t) => {
                out.push('<');
                for (n, v) in t.names.iter().zip(t.values.iter()) {
                    out.push_str(n);
                    out.push(':');
                    v.write_escaped(out, prog);
                    out.push_str(", ");
                }
                out.push('>');
            }
            Value::Seq(s) => {
                out.push('(');
                for (i, v) in s.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    v.write_escaped(out, prog);
                }
                out.push(')');
            }
            Value::Set(s) => {
                out.push('(');
                for (i, v) in s.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push('<');
                    v.write_escaped(out, prog);
                    out.push('>');
                }
                out.push(')');
            }
            Value::Map(m) => {
                out.push('(');
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push('<');
                    k.write_escaped(out, prog);
                    out.push_str("->");
                    v.write_escaped(out, prog);
                    out.push('>');
                }
                out.push(')');
            }
            other => other.write_display(out, prog),
        }
    }
}

/// The C# class name of an event, as it appears in logs (`PHalt`, `DefaultEvent`).
pub fn event_cs_name(prog: &Program, e: EventId) -> &str {
    let d = prog.event(e);
    if d.is_halt() {
        "PHalt"
    } else if d.is_null() {
        "DefaultEvent"
    } else {
        &d.name
    }
}

/// The fully qualified C# name of an event (`PImplementation.e`).
pub fn event_cs_full_name(prog: &Program, e: EventId) -> String {
    let d = prog.event(e);
    if d.is_halt() {
        "PChecker.Runtime.Events.PHalt".to_string()
    } else if d.is_null() {
        "PChecker.Runtime.Events.DefaultEvent".to_string()
    } else {
        format!("PImplementation.{}", d.name)
    }
}

/// The default value of a type (`GetDefaultValue` in the code generator).
pub fn default_value(ty: &PType, prog: &Program) -> Value {
    match ty.canonicalize(prog) {
        PType::Bool => Value::Bool(false),
        PType::Int => Value::Int(0),
        PType::Float => Value::Float(0.0),
        PType::String => Value::str(""),
        PType::Enum(id) => Value::Int(prog.enums[id.idx()].elems.iter().map(|e| e.value).min().unwrap_or(0)),
        PType::Seq(_) => Value::Seq(Rc::new(Vec::new())),
        PType::Set(_) => Value::Set(Rc::new(SlotSet::new())),
        PType::Map(..) => Value::Map(Rc::new(SlotMap::new())),
        PType::Tuple(ts) => Value::Tuple(Rc::new(ts.iter().map(|t| default_value(t, prog)).collect())),
        PType::NamedTuple(fs) => Value::NamedTuple(Rc::new(NamedTuple {
            names: fs.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().into(),
            values: fs.iter().map(|(_, t)| default_value(t, prog)).collect(),
        })),
        _ => Value::Null,
    }
}

/// Formats a double the way .NET Core's `double.ToString()` does (shortest round-trip
/// digits, scientific notation outside `1e-5 ..= 1e15`).
pub fn format_double(f: f64) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if f == 0.0 {
        return if f.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    // Shortest round-trip digits via Rust's `{:e}` formatting.
    let sci = format!("{:e}", f.abs());
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let neg = if f < 0.0 { "-" } else { "" };
    if exp >= 15 || exp < -4 {
        let mut m = String::new();
        m.push_str(&digits[..1]);
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let sign = if exp < 0 { '-' } else { '+' };
        return format!("{}{}E{}{:02}", neg, m, sign, exp.abs());
    }
    let mut out = String::from(neg);
    if exp >= 0 {
        let int_len = exp as usize + 1;
        if digits.len() <= int_len {
            out.push_str(digits);
            for _ in digits.len()..int_len {
                out.push('0');
            }
        } else {
            out.push_str(&digits[..int_len]);
            out.push('.');
            out.push_str(&digits[int_len..]);
        }
    } else {
        out.push_str("0.");
        for _ in 0..(-exp - 1) {
            out.push('0');
        }
        out.push_str(digits);
    }
    out
}

/// .NET composite formatting (`String.Format`) restricted to `{n}` holes, `{{`/`}}`
/// escapes and an optional alignment/format suffix which is ignored.
pub fn format_composite(base: &str, args: &[String]) -> String {
    let mut out = String::new();
    let chars: Vec<char> = base.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                out.push('{');
                i += 2;
                continue;
            }
            let mut j = i + 1;
            let mut num = 0usize;
            let mut has = false;
            while j < chars.len() && chars[j].is_ascii_digit() {
                num = num * 10 + chars[j].to_digit(10).unwrap() as usize;
                has = true;
                j += 1;
            }
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if has {
                if let Some(a) = args.get(num) {
                    out.push_str(a);
                }
            }
            i = j + 1;
        } else if c == '}' {
            if i + 1 < chars.len() && chars[i + 1] == '}' {
                i += 2;
            } else {
                i += 1;
            }
            out.push('}');
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        assert_eq!(format_double(1.0), "1");
        assert_eq!(format_double(0.1), "0.1");
        assert_eq!(format_double(1.5), "1.5");
        assert_eq!(format_double(100.0), "100");
        assert_eq!(format_double(1e15), "1E+15");
        assert_eq!(format_double(1e14), "100000000000000");
        assert_eq!(format_double(0.0001), "0.0001");
        assert_eq!(format_double(0.00001), "1E-05");
        assert_eq!(format_double(-2.5), "-2.5");
        assert_eq!(format_double(123456.789), "123456.789");
    }

    #[test]
    fn composite() {
        assert_eq!(format_composite("a {0} b {1}", &["x".into(), "y".into()]), "a x b y");
        assert_eq!(format_composite("{{0}} {0}", &["x".into()]), "{0} x");
    }

    #[test]
    fn slot_order() {
        let mut s = SlotSet::new();
        for i in 0..4 {
            s.insert(Value::Int(i));
        }
        s.remove(&Value::Int(1));
        s.remove(&Value::Int(2));
        s.insert(Value::Int(9));
        let got: Vec<i64> = s.iter().map(|v| v.as_int()).collect();
        assert_eq!(got, vec![0, 9, 3]);
    }
}
