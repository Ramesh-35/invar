//! `invar.lock`: the producer fields a consumer repository depends on (R2).
//!
//! One file, committed in the consumer repo, beside `invar.toml`:
//!
//! ```text
//! [lock]
//! format = "1"
//!
//! [[contract]]
//! id = "orders"
//! producer = "github:acme/api@main:openapi.yaml"
//! pointer = "/components/schemas/Order"
//! blake3 = "<hash of the producer bytes when locked>"
//! recorded_at = "2026-10-08T14:03:11Z"            # t_r
//! bind = "spec 2026-10-08 Order.amount_cents integer"
//! #       source t_v       Type.field          type
//!
//! [[waiver]]
//! contract = "orders"
//! field = "Order.amount_cents"
//! owner = "nikhil"
//! reason = "moving to amount_minor, tracked in #42"
//! expires = "2026-11-01"
//! ```
//!
//! `invar lock` writes it: fetch the producer (spec and/or code), read what the
//! consumer reads, keep the producer fields the consumer reads (the dependency
//! slice), each bound to its declaring type. `invar check` compares the live
//! producer with that slice: a bound field that no longer exists anywhere, or
//! whose type changed in the same type, is residue. `invar lock --update`
//! accepts a reviewed producer change. This replaces `producer_base`.
//!
//! Time: `recorded_at` (t_r) is when INVAR fetched and wrote a contract;
//! `t_v` on each binding is the day that binding was accepted. Unchanged
//! entries keep both, so re-locking an unchanged producer rewrites nothing.
//!
//! Waivers are written by people and kept across updates. A waived field is
//! reported WAIVED with the days left; from its `expires` date the waiver is
//! EXPIRED and `invar check` fails (exit 1) until it is removed or renewed.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, bail, Context, Result};

use crate::check::{artifact_name, fetch_text, getk, is_code, load_object, Sections, Witness};
use crate::resolver::{Fetched, Resolver};
use crate::surface::{broad_reads, code_fields, name_key, spec_fields, Fields};

pub const LOCK_FORMAT: &str = "1";

/// One bound field: the consumer reads it, the producer declared it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Binding {
    /// "spec" or "code": the producer source that declares it.
    pub source: String,
    /// Declaring type (Go/Rust/TS type, Python class, function for runtime
    /// payloads, spec schema name); "" when there is none.
    pub scope: String,
    pub field: String,
    pub ty: Option<String>,
    /// t_v: the day this binding was accepted.
    pub since: String,
}

impl Binding {
    /// `Type.field`, or `field` when unscoped.
    pub fn display(&self) -> String {
        qualified(&self.scope, &self.field)
    }
    fn key(&self) -> (String, String, String) {
        (self.source.clone(), self.scope.clone(), self.field.clone())
    }
}

pub fn qualified(scope: &str, field: &str) -> String {
    if scope.is_empty() {
        field.to_string()
    } else {
        format!("{scope}.{field}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedContract {
    pub id: String,
    pub producer: String,
    pub pointer: Option<String>,
    pub blake3: String,
    pub producer_code: Option<String>,
    pub blake3_code: Option<String>,
    /// t_r: when INVAR fetched the producer and wrote this entry.
    pub recorded_at: String,
    pub bindings: Vec<Binding>,
    /// Every field of each bound type when locked: (source, scope, fields).
    /// Lets `invar fix --suggest` see what was added beside a removal.
    pub members: Vec<(String, String, Vec<String>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waiver {
    pub contract: String,
    pub field: String,
    pub owner: String,
    pub reason: String,
    /// YYYY-MM-DD; the waiver stops applying on this day.
    pub expires: String,
}

/// Longest a waiver may run: `expires` at most this many days after today.
/// A waiver is written or renewed on the day it is committed, so the cap
/// binds at creation and renewal without a stored creation date.
pub const WAIVER_MAX_DAYS: i64 = 90;

/// Reject waivers that run past the cap. Expired ones are not rejected here:
/// they fail the check as EXPIRED.
pub fn validate_waivers(waivers: &[Waiver], today: i64) -> Result<()> {
    for w in waivers {
        let left = w.days_left(today);
        if left > WAIVER_MAX_DAYS {
            bail!(
                "invar.lock: waiver for {:?} expires {} ({left} days away); the maximum is {WAIVER_MAX_DAYS} days",
                w.field,
                w.expires
            );
        }
    }
    Ok(())
}

impl Waiver {
    /// Days left before `expires` (0 or less means expired).
    pub fn days_left(&self, today: i64) -> i64 {
        parse_date(&self.expires).map(|d| d - today).unwrap_or(0)
    }
    /// Matches residue `Type.field` or bare `field` by exact name.
    pub fn covers(&self, residue: &str) -> bool {
        self.field == residue || residue.rsplit_once('.').map(|(_, f)| f) == Some(&self.field)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lock {
    pub contracts: Vec<LockedContract>,
    pub waivers: Vec<Waiver>,
}

// ---------------------------------------------------------------- dates

/// Days since 1970-01-01 for a proleptic Gregorian date (Hinnant).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (
        if m <= 2 {
            yoe + era * 400 + 1
        } else {
            yoe + era * 400
        },
        m,
        d,
    )
}

/// Strict `YYYY-MM-DD` to a day number; `None` for anything else.
pub fn parse_date(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| -> Option<i64> {
        let t = &s[r];
        if t.bytes().all(|c| c.is_ascii_digit()) {
            t.parse().ok()
        } else {
            None
        }
    };
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    if !(1..=12).contains(&m) || d < 1 {
        return None;
    }
    let n = days_from_civil(y, m, d);
    // Reject 2026-02-30 and the like: the date must round-trip.
    (civil_from_days(n) == (y, m, d)).then_some(n)
}

pub fn format_date(day: i64) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{y:04}-{m:02}-{d:02}")
}

fn unix_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Today (UTC) as a day number.
pub fn today_utc() -> i64 {
    unix_secs().div_euclid(86_400)
}

/// Now (UTC), RFC 3339 to the second.
pub fn now_rfc3339() -> String {
    let s = unix_secs();
    let (day, rem) = (s.div_euclid(86_400), s.rem_euclid(86_400));
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        format_date(day),
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

// ---------------------------------------------------------------- surfaces

/// Schema name a pointer selects, used as the scope of spec fields:
/// `/components/schemas/Order` -> `Order`, `/definitions/io.x.v1.PodGC` ->
/// `PodGC`. Structural segments (`schema`, `allOf`, indexes) give "".
fn pointer_scope(pointer: Option<&str>) -> String {
    let last = pointer
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("")
        .replace("~1", "/")
        .replace("~0", "~");
    let last = last.rsplit(['.', '/']).next().unwrap_or("").to_string();
    let structural = [
        "schema",
        "items",
        "properties",
        "allOf",
        "oneOf",
        "anyOf",
        "content",
    ];
    if last.is_empty()
        || !last.as_bytes()[0].is_ascii_alphabetic()
        || structural.contains(&last.as_str())
    {
        String::new()
    } else {
        last
    }
}

/// Spaces collapsed and `"` made `'`, so any type survives the lock file.
fn norm_ty(t: &Option<String>) -> Option<String> {
    t.as_ref()
        .map(|s| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .replace('"', "'")
        })
        .filter(|s| !s.is_empty())
}

/// The producer surface: (source, scope, field) -> type.
type Surface = BTreeMap<(String, String, String), Option<String>>;

struct Live {
    surface: Surface,
    blake3: String,
    blake3_code: Option<String>,
}

fn producer_surface(
    id: &str,
    producer: &str,
    pointer: Option<&str>,
    code: Option<&str>,
    resolver: &Resolver,
    prov: &mut Vec<Fetched>,
) -> Result<Live> {
    let mut s = Surface::new();
    let mut put = |source: &str, fields: Fields, scope_override: Option<&str>| {
        for ((scope, name), ty) in fields {
            let scope = scope_override.map(str::to_string).unwrap_or(scope);
            s.insert((source.into(), scope, name), norm_ty(&ty));
        }
    };
    let text = fetch_text(resolver, producer, prov)?;
    let blake3 = blake3::hash(text.as_bytes()).to_hex().to_string();
    if is_code(producer) {
        let f = code_fields(&artifact_name(producer), &text)
            .ok_or_else(|| anyhow!("{id}: cannot read fields from {}", artifact_name(producer)))?;
        put("code", f, None);
    } else {
        let obj = load_object(id, producer, &text, pointer)?;
        put("spec", spec_fields(&obj), Some(&pointer_scope(pointer)));
    }
    let mut blake3_code = None;
    if let Some(c) = code {
        let t = fetch_text(resolver, c, prov)?;
        blake3_code = Some(blake3::hash(t.as_bytes()).to_hex().to_string());
        let f = code_fields(&artifact_name(c), &t)
            .ok_or_else(|| anyhow!("{id}: cannot read fields from {}", artifact_name(c)))?;
        put("code", f, None);
    }
    Ok(Live {
        surface: s,
        blake3,
        blake3_code,
    })
}

/// Consumer reads, by naming-form key -> sites.
fn consumer_reads(consumer: &str, text: &str) -> BTreeMap<String, Vec<(String, usize)>> {
    let mut out: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    for (f, l) in broad_reads(text) {
        out.entry(name_key(&f)).or_default().push((f, l));
    }
    let _ = consumer;
    out
}

struct Declared {
    id: String,
    producer: String,
    pointer: Option<String>,
    producer_code: Option<String>,
    consumer: String,
}

fn declared(sections: &Sections) -> Result<Vec<Declared>> {
    let mut out = Vec::new();
    for b in crate::check::gather_blocks(sections)? {
        if getk(&b, "kind").as_deref() != Some("schema") {
            continue;
        }
        let id = getk(&b, "id").context("a locked schema invariant needs an id")?;
        out.push(Declared {
            producer: getk(&b, "producer").with_context(|| format!("{id}: needs producer"))?,
            consumer: getk(&b, "consumer").with_context(|| format!("{id}: needs consumer"))?,
            pointer: getk(&b, "pointer"),
            producer_code: getk(&b, "producer_code"),
            id,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------- lock

/// What `invar lock` would write now. `prev` supplies t_r/t_v for entries
/// that did not change, and the waivers, which are always kept.
pub fn compute(sections: &Sections, resolver: &Resolver, prev: &Lock, today: i64) -> Result<Lock> {
    let now = now_rfc3339();
    let mut contracts = Vec::new();
    for d in declared(sections)? {
        if let Some(lang) = crate::check::unsupported_language_of(&[
            d.producer.as_str(),
            d.producer_code.as_deref().unwrap_or(""),
            d.consumer.as_str(),
        ]) {
            bail!("{}: cannot lock, {lang}", d.id);
        }
        let live = producer_surface(
            &d.id,
            &d.producer,
            d.pointer.as_deref(),
            d.producer_code.as_deref(),
            resolver,
            &mut Vec::new(),
        )?;
        let reads = consumer_reads(
            &d.consumer,
            &fetch_text(resolver, &d.consumer, &mut Vec::new())?,
        );
        let old = prev.contracts.iter().find(|c| c.id == d.id);
        let mut bindings: Vec<Binding> = live
            .surface
            .iter()
            .filter(|((_, _, f), _)| reads.contains_key(&name_key(f)))
            .map(|((source, scope, field), ty)| {
                let mut b = Binding {
                    source: source.clone(),
                    scope: scope.clone(),
                    field: field.clone(),
                    ty: ty.clone(),
                    since: format_date(today),
                };
                if let Some(ob) = old.and_then(|o| o.bindings.iter().find(|x| x.key() == b.key())) {
                    if ob.ty == b.ty {
                        b.since = ob.since.clone();
                    }
                }
                b
            })
            .collect();
        bindings.sort();
        let members = members_of(&live.surface, &bindings);
        let mut c = LockedContract {
            id: d.id.clone(),
            producer: d.producer,
            pointer: d.pointer,
            blake3: live.blake3,
            producer_code: d.producer_code,
            blake3_code: live.blake3_code,
            recorded_at: now.clone(),
            bindings,
            members,
        };
        if let Some(o) = old {
            let mut same = o.clone();
            same.recorded_at = c.recorded_at.clone();
            if same == c {
                c.recorded_at = o.recorded_at.clone();
            }
        }
        contracts.push(c);
    }
    contracts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Lock {
        contracts,
        waivers: prev.waivers.clone(),
    })
}

/// All field names of each (source, scope) that has a binding.
fn members_of(surface: &Surface, bindings: &[Binding]) -> Vec<(String, String, Vec<String>)> {
    let scopes: BTreeSet<(String, String)> = bindings
        .iter()
        .map(|b| (b.source.clone(), b.scope.clone()))
        .collect();
    scopes
        .into_iter()
        .map(|(src, scope)| {
            let fields = surface
                .keys()
                .filter(|(s, sc, _)| s == &src && sc == &scope)
                .map(|(_, _, f)| f.clone())
                .collect();
            (src, scope, fields)
        })
        .collect()
}

/// Human-readable changes from `old` to `new`, one line each.
pub fn diff(old: &Lock, new: &Lock) -> Vec<String> {
    let mut out = Vec::new();
    for n in &new.contracts {
        let Some(o) = old.contracts.iter().find(|c| c.id == n.id) else {
            out.push(format!(
                "+ contract {} ({} bindings)",
                n.id,
                n.bindings.len()
            ));
            continue;
        };
        if o.producer != n.producer || o.pointer != n.pointer || o.producer_code != n.producer_code
        {
            out.push(format!("~ contract {}: producer reference changed", n.id));
        }
        let ok: BTreeMap<_, _> = o.bindings.iter().map(|b| (b.key(), b)).collect();
        let nk: BTreeMap<_, _> = n.bindings.iter().map(|b| (b.key(), b)).collect();
        for (k, b) in &ok {
            match nk.get(k) {
                None => out.push(format!("- {} {} ({})", n.id, b.display(), b.source)),
                Some(nb) if nb.ty != b.ty => out.push(format!(
                    "~ {} {} type {} -> {}",
                    n.id,
                    b.display(),
                    b.ty.as_deref().unwrap_or("?"),
                    nb.ty.as_deref().unwrap_or("?")
                )),
                _ => {}
            }
        }
        for (k, b) in &nk {
            if !ok.contains_key(k) {
                out.push(format!("+ {} {} ({})", n.id, b.display(), b.source));
            }
        }
    }
    for o in &old.contracts {
        if !new.contracts.iter().any(|c| c.id == o.id) {
            out.push(format!("- contract {}", o.id));
        }
    }
    out
}

// ---------------------------------------------------------------- file

fn esc_field(s: &str) -> String {
    s.replace('%', "%25").replace(' ', "%20")
}
fn unesc_field(s: &str) -> String {
    s.replace("%20", " ").replace("%25", "%")
}

fn bind_line(b: &Binding) -> String {
    // A leading "." marks an unscoped field whose own name contains a dot.
    let f = esc_field(&b.field);
    let name = if b.scope.is_empty() && b.field.contains('.') {
        format!(".{f}")
    } else {
        qualified(&b.scope, &f)
    };
    format!(
        "{} {} {} {}",
        b.source,
        b.since,
        name,
        b.ty.as_deref().unwrap_or("?")
    )
}

fn parse_bind(line: &str) -> Result<Binding> {
    let mut it = line.splitn(4, ' ');
    let (source, since, name, ty) = (
        it.next().unwrap_or(""),
        it.next().unwrap_or(""),
        it.next().unwrap_or(""),
        it.next().unwrap_or(""),
    );
    if !matches!(source, "spec" | "code")
        || parse_date(since).is_none()
        || name.is_empty()
        || ty.is_empty()
    {
        bail!("bad bind {line:?}: expected \"spec|code YYYY-MM-DD Type.field type\"");
    }
    let (scope, field) = match name.strip_prefix('.') {
        Some(f) => (String::new(), f.to_string()),
        None => match name.split_once('.') {
            Some((s, f)) => (s.to_string(), f.to_string()),
            None => (String::new(), name.to_string()),
        },
    };
    Ok(Binding {
        source: source.into(),
        scope,
        field: unesc_field(&field),
        ty: (ty != "?").then(|| ty.to_string()),
        since: since.into(),
    })
}

fn q(v: &str) -> Result<String> {
    if v.contains('"') || v.contains('\n') {
        bail!("value {v:?} cannot contain a double quote or a line break");
    }
    Ok(format!("\"{v}\""))
}

/// Deterministic text: contracts by id, bindings sorted, waivers as written.
pub fn render(lock: &Lock) -> Result<String> {
    let mut s = String::from(
        "# invar.lock: the producer fields this repository depends on.\n\
         # Written by `invar lock`. Accept a reviewed producer change with\n\
         # `invar lock --update`. Waivers are written by hand and kept.\n\n\
         [lock]\nformat = \"1\"\n",
    );
    for c in &lock.contracts {
        s.push_str(&format!(
            "\n[[contract]]\nid = {}\nproducer = {}\n",
            q(&c.id)?,
            q(&c.producer)?
        ));
        if let Some(p) = &c.pointer {
            s.push_str(&format!("pointer = {}\n", q(p)?));
        }
        s.push_str(&format!("blake3 = {}\n", q(&c.blake3)?));
        if let (Some(pc), Some(h)) = (&c.producer_code, &c.blake3_code) {
            s.push_str(&format!(
                "producer_code = {}\nblake3_code = {}\n",
                q(pc)?,
                q(h)?
            ));
        }
        s.push_str(&format!("recorded_at = {}\n", q(&c.recorded_at)?));
        for b in &c.bindings {
            s.push_str(&format!("bind = {}\n", q(&bind_line(b))?));
        }
        for (src, scope, fields) in &c.members {
            let scope = if scope.is_empty() {
                "-"
            } else {
                scope.as_str()
            };
            let fields: Vec<String> = fields.iter().map(|f| esc_field(f)).collect();
            s.push_str(&format!(
                "members = {}\n",
                q(&format!("{src} {scope} {}", fields.join(" ")))?
            ));
        }
    }
    for w in &lock.waivers {
        s.push_str(&format!(
            "\n[[waiver]]\ncontract = {}\nfield = {}\nowner = {}\nreason = {}\nexpires = {}\n",
            q(&w.contract)?,
            q(&w.field)?,
            q(&w.owner)?,
            q(&w.reason)?,
            q(&w.expires)?
        ));
    }
    Ok(s)
}

pub fn parse(path: &str, text: &str) -> Result<Lock> {
    let sections = crate::config::parse_sections(path, text).map_err(|e| anyhow!("{e}"))?;
    let mut lock = Lock::default();
    let mut format_seen = false;
    for (name, pairs) in &sections {
        let need = |k: &str| getk(pairs, k).ok_or_else(|| anyhow!("{path}: [{name}] needs {k}"));
        match name.as_str() {
            "lock" => {
                if getk(pairs, "format").as_deref() != Some(LOCK_FORMAT) {
                    bail!(
                        "{path}: unsupported lock format (this INVAR reads format {LOCK_FORMAT})"
                    );
                }
                format_seen = true;
            }
            "contract" => {
                let bindings = pairs
                    .iter()
                    .filter(|(k, _)| k == "bind")
                    .map(|(_, v)| parse_bind(v))
                    .collect::<Result<Vec<_>>>()
                    .with_context(|| format!("{path}: contract {:?}", getk(pairs, "id")))?;
                let members = pairs
                    .iter()
                    .filter(|(k, _)| k == "members")
                    .map(|(_, v)| {
                        let mut it = v.split(' ');
                        let src = it.next().unwrap_or("").to_string();
                        let scope = match it.next().unwrap_or("") {
                            "-" => String::new(),
                            x => x.to_string(),
                        };
                        if !matches!(src.as_str(), "spec" | "code") {
                            bail!("{path}: bad members {v:?}");
                        }
                        Ok((
                            src,
                            scope,
                            it.filter(|f| !f.is_empty()).map(unesc_field).collect(),
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                lock.contracts.push(LockedContract {
                    id: need("id")?,
                    producer: need("producer")?,
                    pointer: getk(pairs, "pointer"),
                    blake3: need("blake3")?,
                    producer_code: getk(pairs, "producer_code"),
                    blake3_code: getk(pairs, "blake3_code"),
                    recorded_at: need("recorded_at")?,
                    bindings,
                    members,
                });
            }
            "waiver" => {
                let w = Waiver {
                    contract: need("contract")?,
                    field: need("field")?,
                    owner: need("owner")?,
                    reason: need("reason")?,
                    expires: need("expires")?,
                };
                if [&w.contract, &w.field, &w.owner, &w.reason]
                    .iter()
                    .any(|v| v.trim().is_empty())
                {
                    bail!("{path}: a waiver needs a non-empty contract, field, owner and reason");
                }
                if parse_date(&w.expires).is_none() {
                    bail!(
                        "{path}: waiver for {:?}: expires must be YYYY-MM-DD",
                        w.field
                    );
                }
                lock.waivers.push(w);
            }
            other => bail!("{path}: unknown section [{other}]"),
        }
    }
    if !format_seen {
        bail!("{path}: missing [lock] format");
    }
    Ok(lock)
}

// ---------------------------------------------------------------- check

/// An artifact reference without its ref: `github:o/r@v2:api.yaml` ->
/// `github:o/r:api.yaml`. Local paths are returned unchanged.
fn same_artifact(spec: &str) -> String {
    match spec.strip_prefix("github:").and_then(|r| r.split_once('@')) {
        Some((repo, rest)) => match rest.split_once(':') {
            Some((_, path)) => format!("github:{repo}:{path}"),
            None => spec.to_string(),
        },
        None => spec.to_string(),
    }
}

/// Result of checking one contract against its lock entry.
pub struct LockedResult {
    /// Live fields of each type the lock binds, for `invar fix --suggest`.
    pub live_members: Vec<(String, String, Vec<String>)>,
    pub residue: Vec<String>,
    pub witnesses: Vec<Witness>,
    pub notes: Vec<String>,
}

/// Compare the live producer with the locked slice. Residue: a bound field the
/// consumer still reads that no longer exists in any scope of its source, or
/// whose type changed in the same scope. Reported as `Type.field`.
pub fn check_contract(
    entry: &LockedContract,
    producer: &str,
    pointer: Option<&str>,
    code: Option<&str>,
    consumer: &str,
    resolver: &Resolver,
    prov: &mut Vec<Fetched>,
) -> Result<LockedResult> {
    // Same artifact required; the ref may move (that is what gets checked).
    if same_artifact(&entry.producer) != same_artifact(producer)
        || entry.pointer.as_deref() != pointer
        || entry.producer_code.as_deref().map(same_artifact) != code.map(same_artifact)
    {
        bail!(
            "{}: invar.toml names a different producer than invar.lock; run `invar lock --update`",
            entry.id
        );
    }
    let live = producer_surface(&entry.id, producer, pointer, code, resolver, prov)?;
    let reads = consumer_reads(consumer, &fetch_text(resolver, consumer, prov)?);
    let cname = artifact_name(consumer);
    let mut notes = Vec::new();
    let unchanged = live.blake3 == entry.blake3 && live.blake3_code == entry.blake3_code;
    notes.push(if unchanged {
        format!("producer unchanged since lock ({})", entry.recorded_at)
    } else {
        format!("producer changed since lock ({})", entry.recorded_at)
    });

    let names_in = |source: &str| -> BTreeSet<String> {
        live.surface
            .keys()
            .filter(|(s, _, _)| s == source)
            .map(|(_, _, f)| f.clone())
            .collect()
    };
    let (spec_names, code_names) = (names_in("spec"), names_in("code"));
    let mut residue = Vec::new();
    let mut witnesses = Vec::new();
    let mut seen = BTreeSet::new();
    for b in &entry.bindings {
        let Some(sites) = reads.get(&name_key(&b.field)) else {
            continue; // the consumer no longer reads it: nothing depends on it
        };
        let where_ = if b.source == "spec" {
            format!(
                "{} {}",
                artifact_name(producer),
                pointer.unwrap_or_default()
            )
            .trim()
            .to_string()
        } else {
            artifact_name(code.unwrap_or(producer))
        };
        let names = if b.source == "spec" {
            &spec_names
        } else {
            &code_names
        };
        let (missing, type_change) =
            match live
                .surface
                .get(&(b.source.clone(), b.scope.clone(), b.field.clone()))
            {
                Some(t) => match (&b.ty, t) {
                    (Some(a), Some(n)) if a != n => {
                        (false, Some((a.clone(), n.clone(), where_.clone())))
                    }
                    _ => (false, None),
                },
                None => (!names.contains(&b.field), None),
            };
        if !missing && type_change.is_none() {
            continue;
        }
        let shown = b.display();
        if !seen.insert(shown.clone()) {
            // Same Type.field from the other source: one finding, both facts.
            if let Some(w) = witnesses
                .iter_mut()
                .find(|w: &&mut Witness| w.residue == shown)
            {
                if missing {
                    w.missing_from = Some(match w.missing_from.take() {
                        Some(m) => format!("{m}, {where_}"),
                        None => where_.clone(),
                    });
                }
                if w.type_change.is_none() {
                    w.type_change = type_change;
                }
            }
            continue;
        }
        let mut read_as: Vec<String> = sites
            .iter()
            .map(|(f, _)| f.clone())
            .filter(|f| f != &b.field)
            .collect();
        read_as.sort();
        read_as.dedup();
        let mut s: Vec<String> = sites.iter().map(|(_, l)| format!("{cname}:{l}")).collect();
        s.dedup();
        residue.push(shown.clone());
        witnesses.push(Witness {
            residue: shown,
            sites: s,
            missing_from: missing.then(|| where_.clone()),
            type_change,
            read_as,
        });
    }
    // Reads of producer fields the lock does not bind: the slice is stale.
    let bound: BTreeSet<String> = entry.bindings.iter().map(|b| name_key(&b.field)).collect();
    let unbound: BTreeSet<String> = live
        .surface
        .keys()
        .map(|(_, _, f)| f.clone())
        .filter(|f| reads.contains_key(&name_key(f)) && !bound.contains(&name_key(f)))
        .collect();
    if !unbound.is_empty() {
        notes.push(format!(
            "{} read field(s) not in invar.lock ({}); run `invar lock --update` to bind them",
            unbound.len(),
            unbound.into_iter().take(5).collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(LockedResult {
        live_members: members_of(&live.surface, &entry.bindings),
        residue,
        witnesses,
        notes,
    })
}
