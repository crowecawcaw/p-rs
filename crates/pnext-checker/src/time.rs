//! Time tracking: causal clocks over affine forms, and latency profiles.
//!
//! See `design/time/PROPOSAL.md`. A time value is an affine form
//! `c + Σ a_i·ε_i` with `ε_i ∈ [0, 1]`: every draw of a range gets a fresh
//! symbol, so differences of values that share history cancel exactly.
//! Times are microseconds.

use std::collections::BTreeMap;

/// Forms with more symbols than this collapse their smallest terms.
pub const MAX_TERMS: usize = 32;

/// An affine form over noise symbols in `[0, 1]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Time {
    c: i64,
    /// Sorted by symbol, no zero coefficients.
    terms: Vec<(u32, i64)>,
}

/// Source of fresh symbols (one per execution).
#[derive(Debug, Default, Clone)]
pub struct Symbols {
    next: u32,
}

impl Symbols {
    fn fresh(&mut self) -> u32 {
        let s = self.next;
        self.next = self.next.wrapping_add(1);
        s
    }
}

impl Time {
    pub fn point(c: i64) -> Time {
        Time { c, terms: Vec::new() }
    }

    /// A fresh draw of `lo..hi`.
    pub fn range(r: Range, syms: &mut Symbols) -> Time {
        let mut t = Time::point(r.lo);
        if r.hi > r.lo {
            t.terms.push((syms.fresh(), r.hi - r.lo));
        }
        t
    }

    pub fn is_point(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn best(&self) -> i64 {
        self.terms.iter().fold(self.c, |acc, &(_, a)| acc.saturating_add(a.min(0)))
    }

    pub fn worst(&self) -> i64 {
        self.terms.iter().fold(self.c, |acc, &(_, a)| acc.saturating_add(a.max(0)))
    }

    fn combine(&self, other: &Time, sign: i64) -> Time {
        let mut terms = Vec::with_capacity(self.terms.len() + other.terms.len());
        let (mut i, mut j) = (0, 0);
        while i < self.terms.len() || j < other.terms.len() {
            let a = self.terms.get(i);
            let b = other.terms.get(j);
            match (a, b) {
                (Some(&(sa, ca)), Some(&(sb, cb))) if sa == sb => {
                    let c = ca.saturating_add(sign.saturating_mul(cb));
                    if c != 0 {
                        terms.push((sa, c));
                    }
                    i += 1;
                    j += 1;
                }
                (Some(&(sa, ca)), Some(&(sb, _))) if sa < sb => {
                    terms.push((sa, ca));
                    i += 1;
                }
                (Some(&(sa, ca)), None) => {
                    terms.push((sa, ca));
                    i += 1;
                }
                (_, Some(&(sb, cb))) => {
                    terms.push((sb, sign.saturating_mul(cb)));
                    j += 1;
                }
                (None, None) => unreachable!(),
            }
        }
        Time { c: self.c.saturating_add(sign.saturating_mul(other.c)), terms }
    }

    pub fn add(&self, other: &Time) -> Time {
        self.combine(other, 1)
    }

    pub fn sub(&self, other: &Time) -> Time {
        self.combine(other, -1)
    }

    /// `max(a, b)`: exact when the sign of `a - b` is certain; otherwise a fresh
    /// form with the exact range but no correlation.
    pub fn max(&self, other: &Time, syms: &mut Symbols) -> Time {
        let d = self.sub(other);
        if d.best() >= 0 {
            return self.clone();
        }
        if d.worst() <= 0 {
            return other.clone();
        }
        let lo = self.best().max(other.best());
        let hi = self.worst().max(other.worst());
        Time::range(Range { lo, hi }, syms)
    }

    /// Keeps the form small: folds the smallest terms into one fresh symbol. The
    /// range is preserved; correlation with other values through those symbols is lost.
    pub fn collapse(&mut self, syms: &mut Symbols) {
        if self.terms.len() <= MAX_TERMS {
            return;
        }
        let mut by_size: Vec<(u32, i64)> = self.terms.clone();
        by_size.sort_by_key(|&(s, a)| (a.unsigned_abs(), s));
        let fold = by_size.len() - (MAX_TERMS - 1);
        let folded: Vec<u32> = by_size[..fold].iter().map(|&(s, _)| s).collect();
        let (mut lo, mut hi) = (0i64, 0i64);
        for &(_, a) in &by_size[..fold] {
            lo = lo.saturating_add(a.min(0));
            hi = hi.saturating_add(a.max(0));
        }
        self.terms.retain(|(s, _)| !folded.contains(s));
        self.c = self.c.saturating_add(lo);
        if hi > lo {
            // A fresh symbol is larger than every existing one, so order is kept.
            self.terms.push((syms.fresh(), hi - lo));
        }
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_point() {
            write!(f, "{}", self.c)
        } else {
            write!(f, "[{}, {}]", self.best(), self.worst())
        }
    }
}

// ---- runtime state -----------------------------------------------------------------

/// The stamp a sent event carries: when it is available to its receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub at: Time,
    /// Operation id of the sending machine.
    pub sender: u64,
}

/// What a test run needs to track time (`CheckerOptions::time`).
#[derive(Debug, Clone, Default)]
pub struct TimeOptions {
    /// Delivery latencies and costs for this test, if any.
    pub profile: Option<Profile>,
    /// `--timed`: only schedule operations that are not certainly later than another.
    pub timed: bool,
}

impl TimeOptions {
    /// The `.schedule` header line identifying the mode and profile.
    pub fn schedule_header(&self) -> String {
        match &self.profile {
            Some(p) => format!("timed={};profile={};hash={:016x}", self.timed as u8, p.name, p.hash()),
            None => format!("timed={};profile=-;hash={:016x}", self.timed as u8, 0),
        }
    }
}

/// Per-execution time state of the runtime.
#[derive(Debug, Default)]
pub struct TimeState {
    pub opts: TimeOptions,
    pub syms: Symbols,
    /// `Mark()` handles.
    pub marks: Vec<Time>,
    /// The machine whose send or announce a monitor is currently observing.
    pub observer: Option<usize>,
}

impl TimeState {
    pub fn new(opts: TimeOptions) -> TimeState {
        TimeState { opts, ..Default::default() }
    }

    /// `max(a, b)`, collapsed.
    pub fn join(&mut self, a: &Time, b: &Time) -> Time {
        let mut t = a.max(b, &mut self.syms);
        t.collapse(&mut self.syms);
        t
    }

    /// `a + draw(r)`, collapsed.
    pub fn advance(&mut self, a: &Time, r: Range) -> Time {
        let mut t = a.add(&Time::range(r, &mut self.syms));
        t.collapse(&mut self.syms);
        t
    }
}

/// Names of the foreign functions the checker implements natively.
pub const TIME_FUNCTIONS: [&str; 7] = ["Now", "NowBest", "Elapse", "ElapseCost", "Mark", "Elapsed", "ElapsedBest"];

// ---- profiles ------------------------------------------------------------------------

/// A latency or cost range in microseconds, `lo <= hi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub lo: i64,
    pub hi: i64,
}

/// One resolved latency profile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub default: Option<Range>,
    pub events: BTreeMap<String, Range>,
    pub links: BTreeMap<(String, String), Range>,
    pub link_events: BTreeMap<(String, String, String), Range>,
    pub costs: BTreeMap<String, Range>,
}

impl Profile {
    /// Delivery latency of `event` sent from a `sender` machine to a `receiver`
    /// machine (declaration names). Self-sends only use the event entry.
    pub fn latency(&self, sender: &str, receiver: &str, event: &str, self_send: bool) -> Option<Range> {
        if self_send {
            return self.events.get(event).copied();
        }
        let link = (sender.to_string(), receiver.to_string());
        self.link_events
            .get(&(link.0.clone(), link.1.clone(), event.to_string()))
            .or_else(|| self.links.get(&link))
            .or_else(|| self.events.get(event))
            .or(self.default.as_ref())
            .copied()
    }

    pub fn cost(&self, name: &str) -> Option<Range> {
        self.costs.get(name).copied()
    }

    /// Rejects entries naming machines or events the program does not declare.
    pub fn check_names(&self, machines: &[&str], events: &[&str]) -> Result<(), String> {
        let bad = |kind: &str, n: &str| format!("latency profile '{}': unknown {} '{}'", self.name, kind, n);
        let machine = |n: &String| if machines.contains(&n.as_str()) { Ok(()) } else { Err(bad("machine", n)) };
        let event = |n: &String| if events.contains(&n.as_str()) { Ok(()) } else { Err(bad("event", n)) };
        for e in self.events.keys() {
            event(e)?;
        }
        for (a, b) in self.links.keys() {
            machine(a)?;
            machine(b)?;
        }
        for (a, b, e) in self.link_events.keys() {
            machine(a)?;
            machine(b)?;
            event(e)?;
        }
        Ok(())
    }

    /// Stable hash of the resolved entries (FNV-1a), recorded in schedule files.
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |s: &str| {
            for b in s.bytes().chain(std::iter::once(0)) {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let r = |r: &Range| format!("{}..{}", r.lo, r.hi);
        feed(&self.name);
        if let Some(d) = &self.default {
            feed("default");
            feed(&r(d));
        }
        for (k, v) in &self.events {
            feed("e");
            feed(k);
            feed(&r(v));
        }
        for ((a, b), v) in &self.links {
            feed("l");
            feed(a);
            feed(b);
            feed(&r(v));
        }
        for ((a, b, e), v) in &self.link_events {
            feed("le");
            feed(a);
            feed(b);
            feed(e);
            feed(&r(v));
        }
        for (k, v) in &self.costs {
            feed("c");
            feed(k);
            feed(&r(v));
        }
        h
    }
}

/// The contents of a `--latency` file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileSet {
    pub profiles: BTreeMap<String, Profile>,
    /// Test case name -> profile name.
    pub tests: BTreeMap<String, String>,
}

impl ProfileSet {
    /// Parses the TOML subset described in the proposal: comments, `[section]`
    /// headers and `key = "string"` pairs (keys bare or quoted).
    pub fn parse(text: &str) -> Result<ProfileSet, String> {
        enum Section {
            None,
            Profile(String),
            Tests,
        }
        let mut raw: BTreeMap<String, Vec<(String, String, usize)>> = BTreeMap::new();
        let mut tests = BTreeMap::new();
        let mut section = Section::None;
        for (n, line) in text.lines().enumerate() {
            let ln = n + 1;
            let line = strip_comment(line).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(h) = line.strip_prefix('[') {
                let h = h.strip_suffix(']').ok_or_else(|| format!("line {}: malformed section header", ln))?.trim();
                section = if h == "tests" {
                    Section::Tests
                } else if let Some(p) = h.strip_prefix("profile.") {
                    let p = unquote(p.trim());
                    if raw.contains_key(&p) {
                        return Err(format!("line {}: profile '{}' is defined twice", ln, p));
                    }
                    raw.insert(p.clone(), Vec::new());
                    Section::Profile(p)
                } else {
                    return Err(format!("line {}: unknown section '[{}]' (expected [profile.<name>] or [tests])", ln, h));
                };
                continue;
            }
            let (k, v) = split_pair(line).ok_or_else(|| format!("line {}: expected key = \"value\"", ln))?;
            match &section {
                Section::None => return Err(format!("line {}: entry outside a section", ln)),
                Section::Tests => {
                    tests.insert(k, v);
                }
                Section::Profile(p) => raw.get_mut(p).unwrap().push((k, v, ln)),
            }
        }
        let mut profiles = BTreeMap::new();
        for name in raw.keys() {
            let mut visiting = Vec::new();
            let p = resolve(name, &raw, &mut visiting)?;
            profiles.insert(name.clone(), p);
        }
        for (t, p) in &tests {
            if !profiles.contains_key(p) {
                return Err(format!("[tests]: test '{}' uses unknown profile '{}'", t, p));
            }
        }
        Ok(ProfileSet { profiles, tests })
    }

    /// The profile to use for `test`: `explicit` (from `--latency-profile`) if given,
    /// else the `[tests]` entry.
    pub fn for_test(&self, test: &str, explicit: Option<&str>) -> Result<Option<&Profile>, String> {
        match explicit {
            Some(p) => self.profiles.get(p).map(Some).ok_or_else(|| format!("unknown latency profile '{}'", p)),
            None => Ok(self.tests.get(test).and_then(|p| self.profiles.get(p))),
        }
    }
}

fn resolve(
    name: &str,
    raw: &BTreeMap<String, Vec<(String, String, usize)>>,
    visiting: &mut Vec<String>,
) -> Result<Profile, String> {
    if visiting.iter().any(|v| v == name) {
        return Err(format!("profile '{}' extends itself", name));
    }
    let entries = raw.get(name).ok_or_else(|| format!("unknown profile '{}'", name))?;
    visiting.push(name.to_string());
    let mut p = match entries.iter().find(|(k, _, _)| k == "extends") {
        Some((_, parent, _)) => resolve(parent, raw, visiting)?,
        None => Profile::default(),
    };
    visiting.pop();
    p.name = name.to_string();
    for (k, v, ln) in entries {
        if k == "extends" {
            continue;
        }
        let err = |m: String| format!("line {}: {}", ln, m);
        if k == "default" {
            p.default = Some(parse_range(v).map_err(err)?);
        } else if let Some(c) = k.strip_prefix("cost.") {
            p.costs.insert(c.to_string(), parse_range(v).map_err(err)?);
        } else if k.starts_with('(') {
            let close = k.find(')').ok_or_else(|| err(format!("malformed link '{}'", k)))?;
            let inner = &k[1..close];
            let (a, b) = inner.split_once(',').ok_or_else(|| err(format!("link '{}' needs two machine names", k)))?;
            let (a, b) = (a.trim().to_string(), b.trim().to_string());
            let rest = k[close + 1..].trim();
            let r = parse_range(v).map_err(err)?;
            if rest.is_empty() {
                p.links.insert((a, b), r);
            } else if let Some(e) = rest.strip_prefix('.') {
                p.link_events.insert((a, b, e.trim().to_string()), r);
            } else {
                return Err(err(format!("malformed link entry '{}'", k)));
            }
        } else if is_ident(k) {
            p.events.insert(k.clone(), parse_range(v).map_err(err)?);
        } else {
            return Err(err(format!("unknown key '{}'", k)));
        }
    }
    Ok(p)
}

fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

fn unquote(s: &str) -> String {
    s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s).to_string()
}

fn split_pair(line: &str) -> Option<(String, String)> {
    // The key may be quoted and contain '='-free text such as "(A, B).e".
    let (k, v) = if line.starts_with('"') {
        let end = line[1..].find('"')? + 1;
        let rest = line[end + 1..].trim_start().strip_prefix('=')?;
        (line[1..end].to_string(), rest)
    } else {
        let (k, v) = line.split_once('=')?;
        (k.trim().to_string(), v)
    };
    let v = v.trim();
    let v = v.strip_prefix('"')?.strip_suffix('"')?;
    Some((k, v.to_string()))
}

fn is_ident(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_alphabetic() || c == '_') && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `"5ms"`, `"1ms..2ms"`, `"250us"`, `"2s"`, `"0"`.
pub fn parse_range(s: &str) -> Result<Range, String> {
    let (lo, hi) = match s.split_once("..") {
        Some((a, b)) => (parse_duration(a)?, parse_duration(b)?),
        None => {
            let d = parse_duration(s)?;
            (d, d)
        }
    };
    if lo > hi {
        return Err(format!("range '{}' has lo > hi", s));
    }
    Ok(Range { lo, hi })
}

fn parse_duration(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if s == "0" {
        return Ok(0);
    }
    let (num, mult) = if let Some(n) = s.strip_suffix("us") {
        (n, 1)
    } else if let Some(n) = s.strip_suffix("ms") {
        (n, 1_000)
    } else if let Some(n) = s.strip_suffix('s') {
        (n, 1_000_000)
    } else {
        return Err(format!("duration '{}' needs a unit (us, ms or s)", s));
    };
    let v: i64 = num.trim().parse().map_err(|_| format!("duration '{}' must be a whole number with a unit", s))?;
    if v < 0 {
        return Err(format!("duration '{}' is negative", s));
    }
    v.checked_mul(mult).ok_or_else(|| format!("duration '{}' is too large", s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(lo: i64, hi: i64) -> Range {
        Range { lo, hi }
    }

    #[test]
    fn shared_history_cancels() {
        let mut s = Symbols::default();
        let t0 = Time::range(r(6, 22), &mut s);
        let t1 = t0.add(&Time::range(r(6, 22), &mut s));
        let d = t1.sub(&t0);
        assert_eq!((d.best(), d.worst()), (6, 22));
        // Plain interval subtraction would give [-10, 38].
        assert_eq!((t1.best() - t0.worst(), t1.worst() - t0.best()), (-10, 38));
    }

    #[test]
    fn max_is_exact_when_ordered() {
        let mut s = Symbols::default();
        let a = Time::range(r(0, 10), &mut s);
        let b = a.add(&Time::range(r(1, 5), &mut s));
        assert_eq!(a.max(&b, &mut s), b);
        assert_eq!(b.max(&a, &mut s), b);
        let c = Time::range(r(3, 20), &mut s);
        let m = a.max(&c, &mut s);
        assert_eq!((m.best(), m.worst()), (3, 20));
    }

    #[test]
    fn collapse_keeps_range() {
        let mut s = Symbols::default();
        let mut t = Time::point(0);
        for i in 0..100 {
            t = t.add(&Time::range(r(i, i + 1 + i % 7), &mut s));
        }
        let (b, w) = (t.best(), t.worst());
        t.collapse(&mut s);
        assert!(t.terms.len() <= MAX_TERMS);
        assert_eq!((t.best(), t.worst()), (b, w));
        assert!(t.terms.windows(2).all(|p| p[0].0 < p[1].0));
    }

    #[test]
    fn parse_profiles() {
        let text = r#"
            # comment
            [profile.base]
            default = "500us..2ms"
            eReq = "1ms..2ms"   # trailing comment
            "(Client, LB)" = "1ms..5ms"
            "(LB, Db).eGet" = "2ms..4ms"
            "cost.DbQuery" = "2ms..8ms"

            [profile."far"]
            extends = "base"
            "(LB, Db)" = "60ms..90ms"

            [tests]
            tcA = "far"
        "#;
        let ps = ProfileSet::parse(text).unwrap();
        let far = ps.for_test("tcA", None).unwrap().unwrap();
        assert_eq!(far.latency("LB", "Db", "eGet", false), Some(r(2_000, 4_000)));
        assert_eq!(far.latency("LB", "Db", "eOther", false), Some(r(60_000, 90_000)));
        assert_eq!(far.latency("Client", "LB", "eReq", false), Some(r(1_000, 5_000)));
        assert_eq!(far.latency("X", "Y", "eReq", false), Some(r(1_000, 2_000)));
        assert_eq!(far.latency("X", "Y", "eZ", false), Some(r(500, 2_000)));
        assert_eq!(far.latency("X", "X", "eZ", true), None);
        assert_eq!(far.latency("X", "X", "eReq", true), Some(r(1_000, 2_000)));
        assert_eq!(far.cost("DbQuery"), Some(r(2_000, 8_000)));
        assert_ne!(far.hash(), ps.profiles["base"].hash());
        assert!(ps.for_test("tcB", None).unwrap().is_none());
    }

    #[test]
    fn parse_errors() {
        assert!(ProfileSet::parse("[profile.a]\nx = \"5\"").is_err());
        assert!(ProfileSet::parse("[profile.a]\nx = \"5ms..1ms\"").is_err());
        assert!(ProfileSet::parse("[profile.a]\nextends = \"a\"").is_err());
        assert!(ProfileSet::parse("[tests]\nt = \"nope\"").is_err());
        assert!(ProfileSet::parse("x = \"1ms\"").is_err());
        assert!(ProfileSet::parse("[profile.a]\nx = \"1.5ms\"").is_err());
    }
}
