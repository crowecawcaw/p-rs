//! Reading observed events from JSON: ISO-8601 instants, P values from JSON payloads, and
//! the built-in parsers of `prs observe` ([`JsonLinesParser`], [`PTraceParser`]). Shared
//! with [`crate::test`].

use std::rc::Rc;

use crate::checker::observe::{ObservedEvent, ParsedEntry, Parser, Spec, Timestamp};
use crate::checker::value::{MachineRef, NamedTuple, SlotMap, SlotSet, Value};
use crate::date::days_from_civil;
use crate::sema::{PType, Program};
use serde_json::Value as Json;

/// Parses an ISO-8601 instant (`2024-02-02T17:27:34.309Z`, offsets allowed).
pub fn parse_instant(s: &str) -> Option<Timestamp> {
    let s = s.trim();
    let b = s.as_bytes();
    let num = |a: usize, n: usize| -> Option<i64> {
        let t = s.get(a..a + n)?;
        t.chars().all(|c| c.is_ascii_digit()).then(|| t.parse().ok())?
    };
    let y = num(0, 4)?;
    if b.get(4) != Some(&b'-') || b.get(7) != Some(&b'-') || !matches!(b.get(10), Some(b'T') | Some(b't') | Some(b' '))
    {
        return None;
    }
    let (mo, d, h, mi) = (num(5, 2)?, num(8, 2)?, num(11, 2)?, num(14, 2)?);
    if b.get(13) != Some(&b':') {
        return None;
    }
    let (sec, mut i) = if b.get(16) == Some(&b':') { (num(17, 2)?, 19) } else { (0, 16) };
    let mut nanos: u32 = 0;
    if b.get(i) == Some(&b'.') || b.get(i) == Some(&b',') {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        let frac = &s[start..i];
        if frac.is_empty() {
            return None;
        }
        let mut f: String = frac.chars().take(9).collect();
        while f.len() < 9 {
            f.push('0');
        }
        nanos = f.parse().ok()?;
    }
    let offset = match &s[i..] {
        "Z" | "z" | "" => 0,
        o if (o.starts_with('+') || o.starts_with('-')) && o.len() >= 3 => {
            let sign = if o.starts_with('-') { -1 } else { 1 };
            let rest = o[1..].replace(':', "");
            let hh: i64 = rest.get(0..2)?.parse().ok()?;
            let mm: i64 = if rest.len() >= 4 { rest.get(2..4)?.parse().ok()? } else { 0 };
            sign * (hh * 3600 + mm * 60)
        }
        _ => return None,
    };
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || sec > 60 {
        return None;
    }
    let days = days_from_civil(y, mo as u32, d as u32);
    Some(Timestamp::new(days * 86400 + h * 3600 + mi * 60 + sec - offset, nanos))
}

/// Converts JSON to a P value of type `ty`.
///
/// Integers, floats, booleans and strings map directly; enums accept the element name or
/// value; tuples accept an array or an object keyed by field name (or index); sequences and
/// sets take arrays; maps take an object (keys parsed by the key type) or an array of
/// `[key, value]` pairs; machines take their id number or `Name(id)`; events their name.
pub fn value_from_json(prog: &Program, j: &Json, ty: &PType) -> Result<Value, String> {
    let bad = || format!("cannot read {} as {}", j, type_name(prog, ty));
    Ok(match ty {
        PType::TypeDef(t) => return value_from_json(prog, j, &prog.typedefs[t.idx()].ty),
        _ if j.is_null() && !matches!(ty, PType::Bool | PType::Int | PType::Float | PType::Enum(_)) => Value::Null,
        PType::Bool => Value::Bool(j.as_bool().ok_or_else(bad)?),
        PType::Int => Value::Int(json_int(j).ok_or_else(bad)?),
        PType::Float => Value::Float(j.as_f64().ok_or_else(bad)? as f32 as f64),
        PType::String => Value::str(j.as_str().ok_or_else(bad)?),
        PType::Enum(e) => {
            let en = &prog.enums[e.idx()];
            match j {
                Json::String(s) => Value::Int(en.elems.iter().find(|x| &x.name == s).ok_or_else(bad)?.value),
                _ => {
                    let v = json_int(j).ok_or_else(bad)?;
                    en.elems.iter().find(|x| x.value == v).ok_or_else(bad)?;
                    Value::Int(v)
                }
            }
        }
        PType::Machine | PType::Permission(_) => {
            let id = match j {
                Json::String(s) => {
                    let inner = s.rsplit_once('(').map(|(_, r)| r.trim_end_matches(')')).unwrap_or(s);
                    inner.trim().parse::<u64>().map_err(|_| bad())?
                }
                _ => j.as_u64().ok_or_else(bad)?,
            };
            let name = match j {
                Json::String(s) => s.clone(),
                _ => id.to_string(),
            };
            Value::Machine(Rc::new(MachineRef { id, name: name.into(), permissions: Rc::from(Vec::new()) }))
        }
        PType::Event => {
            let s = j.as_str().ok_or_else(bad)?;
            let s = s.rsplit('.').next().unwrap_or(s);
            let id = prog.events.iter().position(|e| e.name == s).ok_or_else(bad)?;
            Value::Event(crate::sema::EventId(id as u32))
        }
        PType::Tuple(ts) => {
            let items: Vec<&Json> = match j {
                Json::Array(a) => a.iter().collect(),
                Json::Object(o) => (0..ts.len()).map(|i| o.get(&i.to_string()).unwrap_or(&Json::Null)).collect(),
                _ => return Err(bad()),
            };
            if items.len() != ts.len() {
                return Err(bad());
            }
            Value::Tuple(Rc::new(
                items.iter().zip(ts).map(|(v, t)| value_from_json(prog, v, t)).collect::<Result<_, _>>()?,
            ))
        }
        PType::NamedTuple(fs) => {
            let mut values = Vec::new();
            for (i, (n, t)) in fs.iter().enumerate() {
                let v = match j {
                    Json::Object(o) => o.get(n).ok_or_else(|| format!("missing field '{}' in {}", n, j))?,
                    Json::Array(a) if a.len() == fs.len() => &a[i],
                    _ => return Err(bad()),
                };
                values.push(value_from_json(prog, v, t)?);
            }
            let names: Vec<String> = fs.iter().map(|(n, _)| n.clone()).collect();
            Value::NamedTuple(Rc::new(NamedTuple { names: names.into(), values }))
        }
        PType::Seq(t) => {
            let a = j.as_array().ok_or_else(bad)?;
            Value::Seq(Rc::new(a.iter().map(|v| value_from_json(prog, v, t)).collect::<Result<_, _>>()?))
        }
        PType::Set(t) => {
            let a = j.as_array().ok_or_else(bad)?;
            let mut s = SlotSet::new();
            for v in a {
                s.insert(value_from_json(prog, v, t)?);
            }
            Value::Set(Rc::new(s))
        }
        PType::Map(kt, vt) => {
            let mut m = SlotMap::new();
            match j {
                Json::Object(o) => {
                    for (k, v) in o {
                        m.set(map_key(prog, k, kt)?, value_from_json(prog, v, vt)?);
                    }
                }
                Json::Array(a) => {
                    for pair in a {
                        match pair.as_array().map(|p| p.as_slice()) {
                            Some([k, v]) => m.set(value_from_json(prog, k, kt)?, value_from_json(prog, v, vt)?),
                            _ => return Err(bad()),
                        }
                    }
                }
                _ => return Err(bad()),
            }
            Value::Map(Rc::new(m))
        }
        PType::Any | PType::Data | PType::Null | PType::Error | PType::Foreign(_) => infer(j),
    })
}

pub(crate) fn json_int(j: &Json) -> Option<i64> {
    j.as_i64().or_else(|| j.as_f64().filter(|f| f.fract() == 0.0).map(|f| f as i64))
}

/// A JSON object key read as a map key of type `ty`.
fn map_key(prog: &Program, k: &str, ty: &PType) -> Result<Value, String> {
    let mut t = ty;
    while let PType::TypeDef(id) = t {
        t = &prog.typedefs[id.idx()].ty;
    }
    if matches!(t, PType::String) {
        // P traces quote string keys
        if k.len() >= 2 && k.starts_with('"') && k.ends_with('"') {
            return Ok(Value::str(&k[1..k.len() - 1].replace("\\\"", "\"")));
        }
        return Ok(Value::str(k));
    }
    let j: Json = serde_json::from_str(k).unwrap_or_else(|_| Json::String(k.to_string()));
    value_from_json(prog, &j, ty)
}

fn infer(j: &Json) -> Value {
    match j {
        Json::Null => Value::Null,
        Json::Bool(b) => Value::Bool(*b),
        Json::Number(n) => match n.as_i64() {
            Some(i) => Value::Int(i),
            None => Value::Float(n.as_f64().unwrap_or(0.0) as f32 as f64),
        },
        Json::String(s) => Value::str(s),
        Json::Array(a) => Value::Seq(Rc::new(a.iter().map(infer).collect())),
        Json::Object(o) => {
            let names: Vec<String> = o.keys().cloned().collect();
            Value::NamedTuple(Rc::new(NamedTuple { names: names.into(), values: o.values().map(infer).collect() }))
        }
    }
}

fn type_name(prog: &Program, ty: &PType) -> String {
    match ty {
        PType::Enum(e) => prog.enums[e.idx()].name.clone(),
        PType::TypeDef(t) => prog.typedefs[t.idx()].name.clone(),
        other => format!("{:?}", other).to_lowercase(),
    }
}

/// Builds the event `name` with a JSON payload; `Ok(None)` if the program declares no such
/// event.
pub fn event_from_json(
    spec: &Spec,
    key: String,
    timestamp: Timestamp,
    name: &str,
    payload: Option<&Json>,
    log_line: Option<&str>,
) -> Result<Option<ObservedEvent>, String> {
    let Some(event) = spec.event_id(name) else { return Ok(None) };
    let ty = spec.payload_type(event).clone();
    let payload = match payload {
        None | Some(Json::Null) => Value::Null,
        Some(p) => value_from_json(spec.prog, p, &ty)?,
    };
    Ok(Some(ObservedEvent {
        key,
        timestamp,
        event,
        payload,
        log_line: log_line.map(String::from),
        custom_payload: None,
    }))
}

/// [`event_from_json`] as a [`ParsedEntry`] that names `name` when the program does not
/// declare it.
fn parsed_from_json(
    spec: &Spec,
    key: String,
    timestamp: Timestamp,
    name: &str,
    payload: Option<&Json>,
    log_line: Option<&str>,
) -> Result<ParsedEntry, String> {
    Ok(match event_from_json(spec, key, timestamp, name, payload, log_line)? {
        Some(e) => ParsedEntry { events: vec![e], unknown_events: Vec::new() },
        None => ParsedEntry { events: Vec::new(), unknown_events: vec![name.to_string()] },
    })
}

// ---- built-in parsers ----------------------------------------------------------------------

/// One JSON object per log line:
/// `{"key": "acct-1", "timestamp": 1706894854309, "event": "eWithDrawReq", "payload": {...}}`.
///
/// `timestamp` is epoch milliseconds or an ISO-8601 string; `key` and `timestamp` are
/// optional (missing ones read as `""` and `0`). `--parserConfiguration` remaps the fields
/// with a JSON object of dotted paths, e.g. `{"event": "msg.type", "key": "msg.account"}`.
/// Blank lines and events the program does not declare are skipped.
#[derive(Debug, Clone)]
pub struct JsonLinesParser {
    pub key: String,
    pub timestamp: String,
    pub event: String,
    pub payload: String,
}

impl Default for JsonLinesParser {
    fn default() -> Self {
        JsonLinesParser {
            key: "key".into(),
            timestamp: "timestamp".into(),
            event: "event".into(),
            payload: "payload".into(),
        }
    }
}

fn lookup<'j>(j: &'j Json, path: &str) -> Option<&'j Json> {
    if path.is_empty() {
        return Some(j);
    }
    path.split('.').try_fold(j, |cur, part| cur.get(part))
}

/// A log entry's key: a string as is, any other value as its JSON text, `""` when missing
/// or null.
pub(crate) fn key_from_json(j: Option<&Json>) -> String {
    match j {
        None | Some(Json::Null) => String::new(),
        Some(Json::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// A timestamp: epoch milliseconds (a whole number, or a string of one) or an ISO-8601
/// instant.
pub(crate) fn timestamp_from_json(j: &Json) -> Result<Timestamp, String> {
    match j {
        Json::String(s) => match s.parse::<i64>() {
            Ok(ms) => Ok(Timestamp::from_millis(ms)),
            Err(_) => parse_instant(s).ok_or_else(|| format!("invalid timestamp '{}'", s)),
        },
        n => json_int(n).map(Timestamp::from_millis).ok_or_else(|| format!("invalid timestamp {}", n)),
    }
}

impl Parser for JsonLinesParser {
    fn name(&self) -> &str {
        "JsonLines"
    }

    fn set_configuration(&mut self, configuration: &str) -> Result<(), String> {
        let j: Json = serde_json::from_str(configuration)
            .map_err(|e| format!("The specified parserConfiguration ({}) is invalid. ({})", configuration, e))?;
        let Some(o) = j.as_object() else {
            return Err(format!(
                "The specified parserConfiguration ({}) is invalid. (expected a JSON object)",
                configuration
            ));
        };
        for (k, v) in o {
            let v = v.as_str().ok_or_else(|| format!("parserConfiguration: '{}' must be a string", k))?.to_string();
            match k.as_str() {
                "key" => self.key = v,
                "timestamp" => self.timestamp = v,
                "event" => self.event = v,
                "payload" => self.payload = v,
                other => return Err(format!("parserConfiguration: unknown field '{}'", other)),
            }
        }
        Ok(())
    }

    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
        self.parse_entry(entry, spec).map(|p| p.events)
    }

    fn parse_entry(&self, entry: &str, spec: &Spec) -> Result<ParsedEntry, String> {
        if entry.is_empty() {
            return Ok(ParsedEntry::default());
        }
        let j: Json = serde_json::from_str(entry).map_err(|e| e.to_string())?;
        let name = lookup(&j, &self.event).and_then(|v| v.as_str()).ok_or("missing event name")?;
        let ts = match lookup(&j, &self.timestamp) {
            None | Some(Json::Null) => Timestamp::default(),
            Some(t) => timestamp_from_json(t)?,
        };
        let key = key_from_json(lookup(&j, &self.key));
        parsed_from_json(spec, key, ts, name, lookup(&j, &self.payload), Some(entry))
    }
}

/// The `SendEvent` and `Announce` entries of P's JSON traces (`p check` bug traces), for
/// `--inputkind JSON`. All events share one key and keep their trace order.
#[derive(Debug, Clone, Copy, Default)]
pub struct PTraceParser;

impl Parser for PTraceParser {
    fn name(&self) -> &str {
        "PTrace"
    }

    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
        self.parse_entry(entry, spec).map(|p| p.events)
    }

    fn parse_entry(&self, entry: &str, spec: &Spec) -> Result<ParsedEntry, String> {
        let j: Json = serde_json::from_str(entry).map_err(|e| e.to_string())?;
        let details = j.get("details").ok_or("missing details")?;
        let name = details.get("event").and_then(|v| v.as_str()).ok_or("missing event name")?;
        let name = name.rsplit('.').next().unwrap_or(name);
        let log = details.get("log").and_then(|v| v.as_str());
        parsed_from_json(spec, String::new(), Timestamp::default(), name, details.get("payload"), log)
    }
}

/// `PTraceJSONFileReader`: the `SendEvent`/`Announce` entries of a JSON trace, each as JSON
/// text.
pub fn ptrace_entries(text: &str) -> Result<Vec<String>, String> {
    let j: Json = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let arr = j.as_array().ok_or("expected a JSON array")?;
    let mut out = Vec::new();
    for e in arr {
        let ty = e.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if ty.eq_ignore_ascii_case("sendevent") || ty.eq_ignore_ascii_case("announce") {
            out.push(e.to_string());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instants() {
        assert_eq!(parse_instant("1970-01-01T00:00:00Z"), Some(Timestamp::new(0, 0)));
        assert_eq!(parse_instant("2024-02-02T17:27:34.309Z").map(|t| t.to_millis()), Some(1706894854309));
        assert_eq!(parse_instant("2024-02-02T18:27:34.309+01:00").map(|t| t.to_millis()), Some(1706894854309));
        assert_eq!(parse_instant("nope"), None);
    }
}
