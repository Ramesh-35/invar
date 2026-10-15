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
use crate::surface::{broad_reads, code_fields, mention_lines, name_key, spec_fields, Fields};

pub const LOCK_FORMAT: &str = "1";
/// Format 2 (alpha.16, read-only legacy) is format 1 plus single-sample
/// [[sample]] sections; format 3 carries presence-witness counters. This
/// engine reads all three, forever, and writes the lowest format that can
/// express the content: 1 without samples, 3 with them. An older engine
/// reading a newer format fails closed with a clear message, never a wrong
/// verdict.
pub const LOCK_FORMATS: [&str; 4] = ["1", "2", "3", "4"];

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

/// One key path's accumulated presence witnesses
/// (docs/architecture/MULTI-SAMPLE-WITNESSES.md).
/// W3 (docs/architecture/W3-VALUE-WITNESS.md, ratified stated-first):
/// whether a path's VALUE has been identical across every observation.
/// Values are never stored; `Constant` carries only a salted fingerprint
/// (16 hex of blake3(contract || path || canonical JSON value)). Once two
/// distinct values are seen the path is `Free` forever and never states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VState {
    /// Not a short scalar (or wildcard/array path): value never tracked.
    Untracked,
    Constant(String),
    Free,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessCount {
    pub path: String,
    /// Observed JSON types, sorted, pipe-joined.
    pub types: String,
    pub present: u64,
    pub absent: u64,
    pub null: u64,
    pub vstate: VState,
}

/// Observed-reality sample record: the SHAPE of real payloads the consumer
/// accepted, as presence witnesses across N observations; never values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub contract: String,
    /// BLAKE3 of the most recent sample file (earlier files stay the
    /// consumer's own archive).
    pub blake3: String,
    /// The day of the most recent observation (YYYY-MM-DD); staleness keys
    /// off this.
    pub taken: String,
    /// The day of the first observation.
    pub window_start: String,
    pub observations: u64,
    pub witnesses: Vec<WitnessCount>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lock {
    pub contracts: Vec<LockedContract>,
    pub waivers: Vec<Waiver>,
    pub samples: Vec<Sample>,
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
    zod_shapes: &[String],
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
    let mut zod_extra: Vec<(String, String)> = Vec::new();
    if is_code(producer) {
        let f = code_fields(&artifact_name(producer), &text)
            .ok_or_else(|| anyhow!("{id}: cannot read fields from {}", artifact_name(producer)))?;
        put("code", f, None);
        // Slice 2 (INIT-DISCOVERY-SLICE2, K2): shape-attributed zod entries
        // for the named shapes only, never replacing a typed entry. With no
        // zod shape named (every slice 1 contract) this adds nothing.
        zod_extra = crate::valuepairs::zod_surface(&text, zod_shapes);
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
    for (scope, field) in zod_extra {
        s.entry(("code".into(), scope, field)).or_insert(None);
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
    /// `evidence = "<Shape> <field>..."` keys written by `invar init`
    /// (directive 86428, pass 2). When present, the lock binds ONLY fields
    /// of these shapes that the evidence names; a hand-written contract
    /// without evidence keys keeps the full surface-times-reads behavior.
    evidence: Vec<(String, Vec<String>)>,
    /// "schema" or "egress" (slice 2).
    kind: String,
    /// Egress only: the exported zod const whose direct keys are locked.
    shape: Option<String>,
}

fn declared(sections: &Sections) -> Result<Vec<Declared>> {
    let mut out = Vec::new();
    for b in crate::check::gather_blocks(sections)? {
        let kind = match getk(&b, "kind").as_deref() {
            Some("schema") => "schema",
            Some("egress") => "egress",
            Some("strings") => "strings",
            _ => continue,
        };
        let id = getk(&b, "id").context("a locked schema invariant needs an id")?;
        let strings_shape = getk(&b, "shape");
        let mut evidence: Vec<(String, Vec<String>)> = b
            .iter()
            .filter(|(k, _)| k == "evidence")
            .filter_map(|(_, v)| {
                let mut it = v.split_whitespace().map(str::to_string);
                let shape = it.next()?;
                let fields: Vec<String> = it.collect();
                (!fields.is_empty()).then_some((shape, fields))
            })
            .collect();
        if kind == "strings" {
            // Slice 2b: the lock binds exactly the `overlap` names, scoped to the shape.
            if let (Some(sh), Some(ov)) = (&strings_shape, getk(&b, "overlap")) {
                let fields: Vec<String> = ov.split_whitespace().map(str::to_string).collect();
                if !fields.is_empty() {
                    evidence.push((sh.clone(), fields));
                }
            }
        }
        out.push(Declared {
            producer: getk(&b, "producer").with_context(|| format!("{id}: needs producer"))?,
            consumer: if kind == "egress" || kind == "strings" {
                String::new()
            } else {
                getk(&b, "consumer").with_context(|| format!("{id}: needs consumer"))?
            },
            pointer: getk(&b, "pointer"),
            producer_code: getk(&b, "producer_code"),
            evidence,
            kind: kind.to_string(),
            shape: getk(&b, "shape"),
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
        if d.kind == "egress" {
            contracts.push(egress_entry(&d, resolver, prev, today, &now)?);
            continue;
        }
        if d.kind == "strings" {
            contracts.push(strings_entry(&d, &declared_list(sections, &d.id), resolver, prev, today, &now)?);
            continue;
        }
        let zod_shapes: Vec<String> = d.evidence.iter().map(|(sh, _)| sh.clone()).collect();
        let live = producer_surface(
            &d.id,
            &d.producer,
            d.pointer.as_deref(),
            d.producer_code.as_deref(),
            resolver,
            &mut Vec::new(),
            &zod_shapes,
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
        // Directive 86428 pass 2: a contract carrying evidence keys binds
        // nothing outside them. LockedBindings(p) is a subset of the
        // evidence-supported fields, scoped to their shapes.
        if !d.evidence.is_empty() {
            bindings.retain(|b| {
                d.evidence.iter().any(|(shape, fields)| {
                    b.scope == *shape && fields.iter().any(|f| name_key(f) == name_key(&b.field))
                })
            });
        }
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
        samples: prev.samples.clone(),
    })
}

/// The lock entry of an egress contract (INIT-DISCOVERY-SLICE2, K1): every
/// direct key of the named zod const, scoped to it. No consumer is read.
fn egress_entry(d: &Declared, resolver: &Resolver, prev: &Lock, today: i64, now: &str) -> Result<LockedContract> {
    let shape = d
        .shape
        .clone()
        .with_context(|| format!("{}: an egress invariant needs shape", d.id))?;
    let text = fetch_text(resolver, &d.producer, &mut Vec::new())?;
    let fields = match crate::valuepairs::read_shape(&text, &shape) {
        crate::valuepairs::ShapeRead::Found(f) => f,
        crate::valuepairs::ShapeRead::Dynamic(why) => bail!(
            "{}: cannot lock, {shape} in {} is not statically extractable ({why})",
            d.id,
            artifact_name(&d.producer)
        ),
        crate::valuepairs::ShapeRead::NotFound => bail!(
            "{}: cannot lock, no `export const {shape} = z.object(...)` in {}",
            d.id,
            artifact_name(&d.producer)
        ),
    };
    let old = prev.contracts.iter().find(|c| c.id == d.id);
    let mut bindings: Vec<Binding> = fields
        .iter()
        .map(|f| {
            let mut b = Binding {
                source: "code".into(),
                scope: shape.clone(),
                field: f.clone(),
                ty: None,
                since: format_date(today),
            };
            if let Some(ob) = old.and_then(|o| o.bindings.iter().find(|x| x.key() == b.key())) {
                b.since = ob.since.clone();
            }
            b
        })
        .collect();
    bindings.sort();
    let mut sorted = fields.clone();
    sorted.sort();
    let mut c = LockedContract {
        id: d.id.clone(),
        producer: d.producer.clone(),
        pointer: None,
        blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
        producer_code: None,
        blake3_code: None,
        recorded_at: now.to_string(),
        bindings,
        members: vec![("code".into(), shape, sorted)],
    };
    if let Some(o) = old {
        let mut same = o.clone();
        same.recorded_at = c.recorded_at.clone();
        if same == c {
            c.recorded_at = o.recorded_at.clone();
        }
    }
    Ok(c)
}

/// The `list` and `listat` keys of the declared strings contract `id`.
fn declared_list(sections: &Sections, id: &str) -> (Option<String>, Option<String>) {
    for b in crate::check::gather_blocks(sections).unwrap_or_default() {
        if getk(&b, "id").as_deref() == Some(id) {
            return (getk(&b, "list"), getk(&b, "listat"));
        }
    }
    (None, None)
}

/// The lock entry of a strings contract (INIT-DISCOVERY-SLICE2B, K3): the
/// `overlap` names, each scoped to the shape. The shape and the list must
/// both be statically extractable and every bound name must be a current key
/// of the shape; nothing is locked on a guess.
fn strings_entry(
    d: &Declared,
    list: &(Option<String>, Option<String>),
    resolver: &Resolver,
    prev: &Lock,
    today: i64,
    now: &str,
) -> Result<LockedContract> {
    use crate::valuepairs::strings::{read_any_shape, read_list, ListRead};
    use crate::valuepairs::ShapeRead;
    let shape = d.shape.clone().with_context(|| format!("{}: a strings invariant needs shape", d.id))?;
    let list_path = list.0.clone().with_context(|| format!("{}: a strings invariant needs list", d.id))?;
    let list_at = list.1.clone().with_context(|| format!("{}: a strings invariant needs listat", d.id))?;
    let text = fetch_text(resolver, &d.producer, &mut Vec::new())?;
    let keys = match read_any_shape(&text, &shape) {
        ShapeRead::Found(f) => f,
        ShapeRead::Dynamic(why) => bail!(
            "{}: cannot lock, {shape} in {} is not statically extractable ({why})",
            d.id,
            artifact_name(&d.producer)
        ),
        ShapeRead::NotFound => bail!("{}: cannot lock, no shape {shape:?} in {}", d.id, artifact_name(&d.producer)),
    };
    let list_text = fetch_text(resolver, &list_path, &mut Vec::new())?;
    match read_list(&list_text, &list_at) {
        ListRead::Found(_) => {}
        ListRead::Dynamic(why) => bail!("{}: cannot lock, list {list_at} in {} is not statically extractable ({why})", d.id, artifact_name(&list_path)),
        ListRead::NotFound => bail!("{}: cannot lock, no list {list_at} in {}", d.id, artifact_name(&list_path)),
    }
    let wanted: Vec<String> = d.evidence.iter().filter(|(s, _)| *s == shape).flat_map(|(_, f)| f.clone()).collect();
    if wanted.is_empty() {
        bail!("{}: a strings invariant needs an overlap key naming the locked fields", d.id);
    }
    let old = prev.contracts.iter().find(|c| c.id == d.id);
    let mut bindings = Vec::new();
    for w in &wanted {
        let Some(k) = keys.iter().find(|k| name_key(k) == name_key(w)) else {
            bail!("{}: cannot lock, {w:?} is not a key of {shape} in {}", d.id, artifact_name(&d.producer));
        };
        let mut b = Binding { source: "code".into(), scope: shape.clone(), field: k.clone(), ty: None, since: format_date(today) };
        if let Some(ob) = old.and_then(|o| o.bindings.iter().find(|x| x.key() == b.key())) {
            b.since = ob.since.clone();
        }
        bindings.push(b);
    }
    bindings.sort();
    bindings.dedup();
    let mut sorted = keys.clone();
    sorted.sort();
    let mut c = LockedContract {
        id: d.id.clone(),
        producer: d.producer.clone(),
        pointer: None,
        blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
        producer_code: None,
        blake3_code: None,
        recorded_at: now.to_string(),
        bindings,
        members: vec![("code".into(), shape, sorted)],
    };
    if let Some(o) = old {
        let mut same = o.clone();
        same.recorded_at = c.recorded_at.clone();
        if same == c {
            c.recorded_at = o.recorded_at.clone();
        }
    }
    Ok(c)
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
    // Lowest format that expresses the content: value-witness states need
    // format 4 (older engines fail closed on it, by design).
    let has_vstate = lock
        .samples
        .iter()
        .any(|sm| sm.witnesses.iter().any(|w| w.vstate != VState::Untracked));
    let format = if lock.samples.is_empty() {
        "1"
    } else if has_vstate {
        "4"
    } else {
        "3"
    };
    let mut s = format!(
        "# invar.lock: the producer fields this repository depends on.\n\
         # Written by `invar lock`. Accept a reviewed producer change with\n\
         # `invar lock --update`. Waivers are written by hand and kept.\n\n\
         [lock]\nformat = \"{format}\"\n",
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
    for sm in &lock.samples {
        s.push_str(&format!(
            "\n[[sample]]\ncontract = {}\nblake3 = {}\ntaken = {}\nwindow_start = {}\nobservations = {}\n",
            q(&sm.contract)?,
            q(&sm.blake3)?,
            q(&sm.taken)?,
            q(&sm.window_start)?,
            q(&sm.observations.to_string())?
        ));
        for w in &sm.witnesses {
            s.push_str(&format!(
                "field = {}\n",
                q(&{
                    let base = format!(
                        "{} {} {}/{}/{}",
                        esc_field(&w.path),
                        w.types,
                        w.present,
                        w.absent,
                        w.null
                    );
                    match &w.vstate {
                        VState::Untracked => base,
                        VState::Free => format!("{base} v=free"),
                        VState::Constant(fp) => format!("{base} v={fp}"),
                    }
                })?
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
                let f = getk(pairs, "format");
                if !LOCK_FORMATS.contains(&f.as_deref().unwrap_or("")) {
                    bail!(
                        "{path}: unsupported lock format (this INVAR reads formats 1 and 2)"
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
            "sample" => {
                let taken = need("taken")?;
                let observations: u64 = match getk(pairs, "observations") {
                    Some(n) => n
                        .parse()
                        .map_err(|_| anyhow!("{path}: bad sample observations {n:?}"))?,
                    None => 1, // format 2: one sample is one observation
                };
                let witnesses = pairs
                    .iter()
                    .filter(|(k, _)| k == "field")
                    .map(|(_, v)| {
                        let mut it = v.split(' ');
                        let (Some(p2), Some(t)) = (it.next(), it.next()) else {
                            bail!("{path}: bad sample field {v:?}");
                        };
                        let counts = it.next();
                        let (present, absent, null) = match counts {
                            Some(c) => {
                                let mut n = c.split('/');
                                let g = |x: Option<&str>| -> Result<u64> {
                                    x.and_then(|v| v.parse().ok())
                                        .ok_or_else(|| anyhow!("{path}: bad sample counts {v:?}"))
                                };
                                (g(n.next())?, g(n.next())?, g(n.next())?)
                            }
                            // format 2: present once; null once when the
                            // observed type includes null.
                            None => (1, 0, u64::from(t.split('|').any(|x| x == "null"))),
                        };
                        let vstate = match it.next() {
                            None => VState::Untracked,
                            Some("v=free") => VState::Free,
                            Some(v) => match v.strip_prefix("v=") {
                                Some(fp) if fp.len() == 16 => VState::Constant(fp.to_string()),
                                _ => bail!("{path}: bad sample value token {v:?}"),
                            },
                        };
                        Ok(WitnessCount {
                            path: unesc_field(p2),
                            types: t.to_string(),
                            present,
                            absent,
                            null,
                            vstate,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let sm = Sample {
                    contract: need("contract")?,
                    blake3: need("blake3")?,
                    window_start: getk(pairs, "window_start").unwrap_or_else(|| taken.clone()),
                    taken,
                    observations,
                    witnesses,
                };
                if parse_date(&sm.taken).is_none() || parse_date(&sm.window_start).is_none() {
                    bail!("{path}: sample for {:?}: dates must be YYYY-MM-DD", sm.contract);
                }
                lock.samples.push(sm);
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
    /// A1: changed or moved bindings INVAR saw but did not evaluate.
    pub unevaluated: Vec<String>,
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
    let zod_shapes: Vec<String> = entry
        .bindings
        .iter()
        .map(|b| b.scope.clone())
        .filter(|sc| !sc.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let live = producer_surface(&entry.id, producer, pointer, code, resolver, prov, &zod_shapes)?;
    let consumer_text = fetch_text(resolver, consumer, prov)?;
    let reads = consumer_reads(consumer, &consumer_text);
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
    let mut unevaluated: Vec<String> = Vec::new();
    let mut witnesses = Vec::new();
    let mut seen = BTreeSet::new();
    for b in &entry.bindings {
        let live_ty = live
            .surface
            .get(&(b.source.clone(), b.scope.clone(), b.field.clone()));
        let in_source = live
            .surface
            .keys()
            .any(|(s, _, f)| s == &b.source && f == &b.field);
        let Some(sites) = reads.get(&name_key(&b.field)) else {
            // A1 rule 1: no recognised read, but a changed binding the
            // consumer still mentions is stated, not dropped.
            let changed = match live_ty {
                Some(t) => matches!((&b.ty, t), (Some(a), Some(n)) if a != n),
                None => !in_source,
            };
            if let (true, Some(l)) = (changed, mention_lines(&consumer_text, &b.field).first()) {
                unevaluated.push(format!(
                    "{} changed; mentioned at {cname}:{l}, not evaluated (pattern not read)",
                    b.display()
                ));
            }
            continue;
        };
        // A1 rule 2: left its type but exists in another type: moved.
        if live_ty.is_none() && in_source && !b.scope.is_empty() {
            let to: Vec<String> = live
                .surface
                .keys()
                .filter(|(s, sc, f)| s == &b.source && f == &b.field && sc != &b.scope)
                .map(|(_, sc, _)| if sc.is_empty() { "(unscoped)".to_string() } else { sc.clone() })
                .collect();
            unevaluated.push(format!(
                "{} moved to {}; consumer read at {cname}:{} not evaluated",
                b.display(),
                to.join(", "),
                sites[0].1
            ));
            continue;
        }
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
    unevaluated.sort();
    unevaluated.dedup();
    Ok(LockedResult {
        unevaluated,
        live_members: members_of(&live.surface, &entry.bindings),
        residue,
        witnesses,
        notes,
    })
}

// ------------------------------------------------- observed-reality samples

/// Derive the SHAPE of a payload value: (key path, type) pairs, deterministic.
/// Types: string, integer, number, boolean, null, object, array; a path seen
/// with several types gets them joined sorted with `|` (so an optional null
/// shows as e.g. `string|null`). Array elements contribute under `path[]`.
pub fn payload_shape(v: &serde_json::Value) -> Vec<(String, String)> {
    use std::collections::{BTreeMap, BTreeSet};
    fn ty(v: &serde_json::Value) -> &'static str {
        match v {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
    }
    // W2 mechanism 1 (docs/architecture/W2-MAP-KEY-ABSTRACTION.md): a map's
    // keys are data, not schema, so its children derive under `*`.
    fn data_like(k: &str) -> bool {
        k.len() > 40
            || (!k.is_empty() && k.bytes().all(|c| c.is_ascii_digit()))
            || k.bytes()
                .any(|c| !(c.is_ascii_alphanumeric() || c == b'_' || c == b'-'))
    }
    fn is_map(m: &serde_json::Map<String, serde_json::Value>) -> bool {
        m.len() >= 16
            || (m.len() >= 2 && m.keys().filter(|k| data_like(k)).count() * 2 > m.len())
    }
    fn walk(v: &serde_json::Value, path: &str, out: &mut BTreeMap<String, BTreeSet<&'static str>>) {
        if !path.is_empty() {
            out.entry(path.to_string()).or_default().insert(ty(v));
        }
        match v {
            serde_json::Value::Object(m) => {
                let collapse = is_map(m);
                for (k, x) in m {
                    let seg = if collapse { "*" } else { k.as_str() };
                    let p = if path.is_empty() { seg.to_string() } else { format!("{path}.{seg}") };
                    walk(x, &p, out);
                }
            }
            serde_json::Value::Array(a) => {
                for x in a {
                    walk(x, &format!("{path}[]"), out);
                }
            }
            _ => {}
        }
    }
    let mut m = BTreeMap::new();
    walk(v, "", &mut m);
    // W2.1 amendment (disclosed in the spec): map detection on the DERIVED
    // tree too. A parent whose merged children exceed the same thresholds
    // (16+ distinct named segments, or a data-like majority of 2+) is a
    // map one level deeper than any single object showed; collapse to `*`
    // and repeat to fixpoint. Catches records nested under map entries
    // whose keys vary across entries (packages.*.bin.<name>).
    loop {
        use std::collections::BTreeMap as M;
        let mut children: M<String, BTreeSet<String>> = M::new();
        for path in m.keys() {
            if let Some(i) = path.rfind('.') {
                let seg = path[i + 1..].trim_end_matches("[]");
                if seg != "*" {
                    children
                        .entry(path[..i].to_string())
                        .or_default()
                        .insert(seg.to_string());
                }
            }
        }
        let marked: Vec<String> = children
            .into_iter()
            .filter(|(_, segs)| {
                segs.len() >= 16
                    || (segs.len() >= 2
                        && segs.iter().filter(|k| data_like(k)).count() * 2 > segs.len())
            })
            .map(|(p, _)| p)
            .collect();
        if marked.is_empty() {
            break;
        }
        let mut next: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
        'outer: for (path, ts) in m {
            for parent in &marked {
                if path.len() > parent.len()
                    && path.starts_with(parent.as_str())
                    && path.as_bytes()[parent.len()] == b'.'
                {
                    let rest = &path[parent.len() + 1..];
                    let (seg, tail) = match rest.find('.') {
                        Some(j) => (&rest[..j], &rest[j..]),
                        None => (rest, ""),
                    };
                    if seg.trim_end_matches("[]") != "*" {
                        let arr = &seg[seg.trim_end_matches("[]").len()..];
                        let new = format!("{parent}.*{arr}{tail}");
                        next.entry(new).or_default().extend(ts.iter());
                        continue 'outer;
                    }
                }
            }
            next.entry(path).or_default().extend(ts.iter());
        }
        m = next;
    }
    m.into_iter()
        .map(|(p, ts)| (p, ts.into_iter().collect::<Vec<_>>().join("|")))
        .collect()
}

/// Merge one observed shape into a sample's witnesses (one observation).
/// Salted value fingerprint (W3): nothing about the value is recoverable
/// from the lock; identical values share a fingerprint.
pub fn value_fp(contract: &str, path: &str, canon: &str) -> String {
    let mut h = blake3::Hasher::new();
    h.update(contract.as_bytes());
    h.update(b"\0");
    h.update(path.as_bytes());
    h.update(b"\0");
    h.update(canon.as_bytes());
    h.finalize().to_hex().to_string()[..16].to_string()
}

/// Canonical short-scalar values per EXACT path, for W3. Eligible: string
/// (at most 32 characters), integer/number, boolean; never null. Paths
/// that W2 collapsed (wildcards) or array elements aggregate many values,
/// so only paths present verbatim in the final shape are kept.
pub fn payload_scalars(
    v: &serde_json::Value,
    shape: &[(String, String)],
) -> std::collections::BTreeMap<String, String> {
    use serde_json::Value;
    let keep: std::collections::BTreeSet<&str> = shape
        .iter()
        .map(|(p, _)| p.as_str())
        .filter(|p| !p.contains('*') && !p.contains("[]"))
        .collect();
    let mut out = std::collections::BTreeMap::new();
    fn walk(v: &Value, path: &str, out: &mut std::collections::BTreeMap<String, String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                    walk(x, &p, out);
                }
            }
            Value::Array(_) | Value::Null => {}
            Value::String(s) if s.chars().count() > 32 => {}
            scalar => {
                out.insert(path.to_string(), scalar.to_string());
            }
        }
    }
    walk(v, "", &mut out);
    out.retain(|k, _| keep.contains(k.as_str()));
    out
}

pub fn merge_observation(sm: &mut Sample, shape: &[(String, String)], taken: &str, blake3: &str) {
    merge_observation_scalars(sm, shape, &std::collections::BTreeMap::new(), taken, blake3)
}

/// Merge one observation: presence/type witnesses plus W3 value constancy.
pub fn merge_observation_scalars(
    sm: &mut Sample,
    shape: &[(String, String)],
    scalars: &std::collections::BTreeMap<String, String>,
    taken: &str,
    blake3: &str,
) {
    use std::collections::BTreeSet;
    let prior = sm.observations;
    sm.observations += 1;
    sm.taken = taken.to_string();
    sm.blake3 = blake3.to_string();
    if sm.window_start.is_empty() {
        sm.window_start = taken.to_string();
    }
    let seen: std::collections::BTreeMap<&str, &str> =
        shape.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    for w in &mut sm.witnesses {
        match seen.get(w.path.as_str()) {
            Some(t) => {
                w.present += 1;
                if t.split('|').any(|x| x == "null") {
                    w.null += 1;
                }
                let mut ts: BTreeSet<&str> = w.types.split('|').collect();
                ts.extend(t.split('|'));
                w.types = ts.into_iter().collect::<Vec<_>>().join("|");
                // W3: constancy only survives identical scalar values; any
                // divergence (or a present non-scalar) frees the path forever.
                if let VState::Constant(fp0) = &w.vstate {
                    let same = scalars
                        .get(&w.path)
                        .map(|c| &value_fp(&sm.contract, &w.path, c) == fp0)
                        .unwrap_or(false);
                    if !same {
                        w.vstate = VState::Free;
                    }
                }
            }
            None => w.absent += 1,
        }
    }
    for (p2, t) in shape {
        if !sm.witnesses.iter().any(|w| &w.path == p2) {
            sm.witnesses.push(WitnessCount {
                path: p2.clone(),
                types: t.clone(),
                present: 1,
                // It did not appear in any earlier observation's shape.
                absent: prior,
                null: u64::from(t.split('|').any(|x| x == "null")),
                vstate: match scalars.get(p2) {
                    Some(c) => VState::Constant(value_fp(&sm.contract, p2, c)),
                    None => VState::Untracked,
                },
            });
        }
    }
    sm.witnesses.sort_by(|a, b| a.path.cmp(&b.path));
}

/// Verdicts for a fresh payload against accumulated witnesses
/// (MULTI-SAMPLE-WITNESSES.md): residue (path, detail) and stated
/// not-evaluated lines. Additions are never drift.
pub fn witness_drift(
    sm: &Sample,
    fresh: &[(String, String)],
    fresh_name: &str,
) -> (Vec<(String, String)>, Vec<String>) {
    witness_drift_scalars(sm, fresh, &std::collections::BTreeMap::new(), fresh_name)
}

/// Drift plus W3 value statements (ratified STATED-FIRST: a changed
/// constant is stated with [not-evaluated:value-switch], never accused;
/// promotion to residue requires its own ratification after a clean
/// false-accusation pass).
pub fn witness_drift_scalars(
    sm: &Sample,
    fresh: &[(String, String)],
    fresh_scalars: &std::collections::BTreeMap<String, String>,
    fresh_name: &str,
) -> (Vec<(String, String)>, Vec<String>) {
    let now: std::collections::BTreeMap<&str, &str> =
        fresh.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    // W2 mechanism 2: when an ancestor path is absent, every descendant is
    // trivially absent; report only the shallowest missing path.
    let absent: Vec<&str> = sm
        .witnesses
        .iter()
        .filter(|w| !now.contains_key(w.path.as_str()))
        .map(|w| w.path.as_str())
        .collect();
    let dominated = |p: &str| {
        absent.iter().any(|a| {
            a.len() < p.len()
                && p.starts_with(a)
                && matches!(p.as_bytes()[a.len()], b'.' | b'[')
        })
    };
    let mut residue = Vec::new();
    let mut stated = Vec::new();
    for w in &sm.witnesses {
        if dominated(&w.path) {
            continue;
        }
        let seen = now.get(w.path.as_str());
        let mismatch = match seen {
            None => Some("absent"),
            Some(t) if !t.split('|').all(|x| w.types.split('|').any(|k| k == x)) => Some("retyped"),
            _ => None,
        };
        let Some(kind) = mismatch else {
            // Present and type-consistent: the W3 value check applies here.
            if let VState::Constant(fp0) = &w.vstate {
                if sm.observations >= 3 && w.present == sm.observations {
                    if let Some(canon) = fresh_scalars.get(&w.path) {
                        if &value_fp(&sm.contract, &w.path, canon) != fp0 {
                            stated.push(format!(
                                "{} held one identical value across all {} observation(s); {fresh_name} shows {canon}; a changed constant often selects a format or version; stated, not a violation [not-evaluated:value-switch]",
                                w.path, sm.observations
                            ));
                        }
                    }
                }
            }
            continue;
        };
        let evidence = format!(
            "witnessed present {}/{} (window {}..{})",
            w.present, sm.observations, sm.window_start, sm.taken
        );
        if w.absent > 0 {
            stated.push(format!(
                "{} is {kind} in {fresh_name}, but it is optional ({} of {} observations lacked it); stated, not a violation [not-evaluated:optional-field-absent]",
                w.path, w.absent, sm.observations
            ));
        } else if sm.observations < 3 {
            stated.push(format!(
                "{} is {kind} in {fresh_name}, but only {} observation(s) witnessed it; too few to accuse [not-evaluated:witness-count-low]",
                w.path, sm.observations
            ));
        } else {
            let detail = match seen {
                None => format!("the fresh payload {fresh_name}; {evidence}"),
                Some(t) => format!(
                    "its witnessed types: {fresh_name} shows {t}, {evidence} as {}",
                    w.types
                ),
            };
            residue.push((w.path.clone(), detail));
        }
    }
    (residue, stated)
}

/// Ratified PII heuristic: key-name tokens that suggest a sample holds
/// personal data or credentials. A warning, never a block.
pub fn pii_suspects(fields: &[(String, String)]) -> Vec<String> {
    let needles = ["email", "password", "ssn", "token", "secret"];
    let mut out: Vec<String> = fields
        .iter()
        .map(|(p, _)| p)
        .filter(|p| {
            let l = p.to_ascii_lowercase();
            needles.iter().any(|n| l.contains(n))
        })
        .cloned()
        .collect();
    out.dedup();
    out
}

/// Days after which a locked sample is stale (the waiver cap, ratified).
pub const SAMPLE_STALE_DAYS: i64 = 90;

// ------------------------------------------------- TOML payload ingestion

/// Parse a TOML-SUBSET payload into a JSON value for shape derivation
/// (docs/architecture/OBSERVED-REALITY.md, TOML ingestion addendum). The
/// subset is what lockfile-style payloads use: `[table]`, `[[array of
/// tables]]`, `key = "string" | integer | true | false`, and arrays of
/// strings (single line or spread over lines). Anything outside the
/// subset is an ERROR naming the line, never a silent partial parse:
/// a payload half-read would witness a shape nobody shipped.
pub fn toml_payload_to_json(text: &str) -> Result<serde_json::Value> {
    use serde_json::{Map, Value};
    let mut root = Map::new();
    // Path of the table currently open; None = root.
    let mut current: Option<(Vec<String>, bool)> = None; // (path, is_array_of_tables)
    let mut pending: Vec<(String, String, usize)> = Vec::new(); // key, joined value, line
    let mut buf: Option<(String, String, usize)> = None; // multiline array accumulation

    fn insert<'a>(root: &'a mut Map<String, Value>, path: &[String], arr: bool) -> Result<&'a mut Map<String, Value>> {
        let mut cur = root;
        for (i, seg) in path.iter().enumerate() {
            let last = i + 1 == path.len();
            if last && arr {
                let e = cur
                    .entry(seg.clone())
                    .or_insert_with(|| Value::Array(Vec::new()));
                let Value::Array(a) = e else {
                    bail!("TOML: {seg:?} is both a table and an array of tables");
                };
                a.push(Value::Object(Map::new()));
                let Some(Value::Object(m)) = a.last_mut() else { unreachable!() };
                cur = m;
            } else {
                let e = cur
                    .entry(seg.clone())
                    .or_insert_with(|| Value::Object(Map::new()));
                cur = match e {
                    Value::Object(m) => m,
                    Value::Array(a) => match a.last_mut() {
                        Some(Value::Object(m)) => m,
                        _ => bail!("TOML: cannot descend into {seg:?}"),
                    },
                    _ => bail!("TOML: {seg:?} is not a table"),
                };
            }
        }
        Ok(cur)
    }

    fn scalar(v: &str, line: usize) -> Result<Value> {
        let v = v.trim();
        if let Some(s) = v.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            if s.contains('\\') {
                bail!("TOML subset: escape sequences not supported (line {line})");
            }
            return Ok(Value::String(s.to_string()));
        }
        if v == "true" || v == "false" {
            return Ok(Value::Bool(v == "true"));
        }
        if let Ok(n) = v.parse::<i64>() {
            return Ok(Value::Number(n.into()));
        }
        bail!("TOML subset: unsupported value {v:?} (line {line})")
    }

    fn parse_value(raw: &str, line: usize) -> Result<Value> {
        let raw = raw.trim();
        if let Some(inner) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let mut out = Vec::new();
            for part in inner.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                out.push(scalar(part, line)?);
            }
            return Ok(serde_json::Value::Array(out));
        }
        scalar(raw, line)
    }

    let flush = |root: &mut Map<String, Value>,
                     current: &Option<(Vec<String>, bool)>,
                     pending: &mut Vec<(String, String, usize)>|
     -> Result<()> {
        if pending.is_empty() {
            return Ok(());
        }
        let taken = std::mem::take(pending);
        let target = match current {
            None => root,
            Some((path, arr)) => insert(root, path, *arr)?,
        };
        for (k, v, line) in taken {
            if target.contains_key(&k) {
                bail!("TOML: duplicate key {k:?} (line {line})");
            }
            target.insert(k, parse_value(&v, line)?);
        }
        Ok(())
    };

    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw.trim();
        if let Some((k, acc, start)) = &mut buf {
            let add = line.split('#').next().unwrap_or("").trim();
            acc.push_str(add);
            if add.ends_with(']') {
                pending.push((k.clone(), acc.clone(), *start));
                buf = None;
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(inner) = line.strip_prefix("[[").and_then(|s| s.strip_suffix("]]")) {
            flush(&mut root, &current, &mut pending)?;
            current = Some((inner.trim().split('.').map(|s| s.trim().to_string()).collect(), true));
            // array-of-tables entry is created eagerly so empty tables exist
            insert(&mut root, &current.as_ref().unwrap().0, true)?;
            current.as_mut().unwrap().1 = false; // further keys go into the entry just pushed
            continue;
        }
        if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            flush(&mut root, &current, &mut pending)?;
            current = Some((inner.trim().split('.').map(|s| s.trim().to_string()).collect(), false));
            insert(&mut root, &current.as_ref().unwrap().0, false)?;
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            bail!("TOML subset: expected `key = value` (line {line_no}): {line:?}");
        };
        let k = k.trim().to_string();
        let v_full = v.split(" #").next().unwrap_or(v).trim().to_string();
        if v_full.starts_with('[') && !v_full.ends_with(']') {
            buf = Some((k, v_full, line_no));
        } else {
            pending.push((k, v_full, line_no));
        }
    }
    if let Some((k, _, start)) = buf {
        bail!("TOML subset: array for {k:?} never closed (line {start})");
    }
    flush(&mut root, &current, &mut pending)?;
    Ok(serde_json::Value::Object(root))
}

/// Parse a sample payload by file extension: `.toml` through the TOML
/// subset above, everything else as JSON. The extension decides; a JSON
/// file named .toml errors loudly rather than guessing.
pub fn parse_sample_payload(path: &str, bytes: &[u8]) -> Result<serde_json::Value> {
    let text = std::str::from_utf8(bytes).with_context(|| format!("{path} is not UTF-8"))?;
    if path.rsplit('.').next().map(|e| e.eq_ignore_ascii_case("toml")) == Some(true) {
        toml_payload_to_json(text).with_context(|| format!("{path} (TOML subset)"))
    } else {
        serde_json::from_str(text).with_context(|| format!("{path} is not JSON"))
    }
}
