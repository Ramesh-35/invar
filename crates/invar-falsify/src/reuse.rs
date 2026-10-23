//! CHECK-INCR-1: reuse a HOLDS when every input of the verdict is unchanged
//! (founder rulings I-D1 to I-D6, 2026-10-23).
//!
//! A hold record says: under engine E, the contract whose invar.toml block
//! hashes to B and whose invar.lock entry hashes to L read these files, with
//! these BLAKE3 hashes, and HOLDS. It lives in invar.lock (`held`,
//! `held_input`, written by `invar lock`) and in the local cache
//! `.invar/cache/verdicts.json` (written by `invar check`). A record is
//! reused only when E, B, L and the bytes of every recorded input are all
//! unchanged; anything else is evaluated. The input list is not written by
//! hand per contract kind: it is every file the evaluation read, captured at
//! the one function all reads pass through (`check::fetch`).
//!
//! Only HOLDS is recorded. A contract named by a waiver or a locked sample is
//! never recorded or reused (ruling I-D2a): its outcome moves with the date.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::resolver::{Fetched, Resolver};

/// Engine identity (ruling I-D5): crate version, "-", and the BLAKE3 of the
/// engine's sources, Cargo.lock and the compiler version, computed by
/// build.rs. Two builds that can disagree on a verdict never share it.
pub const ENGINE: &str = env!("INVAR_ENGINE");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    /// When the HOLDS was evaluated (RFC 3339, UTC).
    pub at: String,
    pub engine: String,
    /// BLAKE3 of the contract's invar.toml block.
    pub block: String,
    /// BLAKE3 of the contract's invar.lock entry (`lock::render_entry`), or
    /// "-" when the contract has none.
    pub entry: String,
    /// (BLAKE3 of the bytes, reference as written), in the order read.
    pub inputs: Vec<(String, String)>,
}

impl Held {
    /// The `held` line: "<at> <engine> <block> <entry>".
    pub fn line(&self) -> String {
        format!("{} {} {} {}", self.at, self.engine, self.block, self.entry)
    }

    /// Read a lock record. A malformed one is no record (the contract is
    /// evaluated), never an error and never a reuse.
    pub fn parse<'a>(held: Option<&str>, inputs: impl Iterator<Item = &'a str>) -> Option<Held> {
        let toks: Vec<&str> = held?.split(' ').collect();
        let [at, engine, block, entry] = toks.as_slice() else {
            return None;
        };
        let inputs: Option<Vec<(String, String)>> = inputs
            .map(|l| l.split_once(' ').map(|(h, r)| (h.to_string(), r.to_string())))
            .collect();
        let inputs = inputs?;
        if inputs.is_empty() {
            return None;
        }
        Some(Held {
            at: at.to_string(),
            engine: engine.to_string(),
            block: block.to_string(),
            entry: entry.to_string(),
            inputs,
        })
    }

    /// Equal in everything but the time it was evaluated.
    pub fn same_inputs(&self, other: &Held) -> bool {
        self.engine == other.engine && self.block == other.block && self.entry == other.entry && self.inputs == other.inputs
    }
}

/// BLAKE3 of an `[[invariant]]` block's pairs, in order, unambiguously.
pub fn block_hash(block: &[(String, String)]) -> String {
    let mut h = blake3::Hasher::new();
    for (k, v) in block {
        h.update(k.as_bytes());
        h.update(&[0]);
        h.update(v.as_bytes());
        h.update(&[0]);
    }
    h.finalize().to_hex().to_string()
}

/// BLAKE3 of a lock entry without its record, or "-" without an entry.
pub fn entry_hash(entry: Option<&crate::lock::LockedContract>) -> String {
    match entry.map(crate::lock::render_entry) {
        Some(Ok(text)) => blake3::hash(text.as_bytes()).to_hex().to_string(),
        Some(Err(_)) => "unrenderable".to_string(),
        None => "-".to_string(),
    }
}

thread_local! {
    static CAPTURE: RefCell<Option<Vec<(String, String)>>> = const { RefCell::new(None) };
}

/// Called by `check::fetch` for every file an evaluation reads.
pub(crate) fn note_read(reference: &str, text: &str) {
    CAPTURE.with(|c| {
        if let Some(v) = c.borrow_mut().as_mut() {
            v.push((blake3::hash(text.as_bytes()).to_hex().to_string(), reference.to_string()));
        }
    });
}

/// Records reads while alive; dropping it (also on an early error return)
/// stops recording, so nothing leaks into a later evaluation.
pub(crate) struct Capture;

impl Capture {
    pub(crate) fn start() -> Capture {
        CAPTURE.with(|c| *c.borrow_mut() = Some(Vec::new()));
        Capture
    }
    pub(crate) fn take(self) -> Vec<(String, String)> {
        CAPTURE.with(|c| c.borrow_mut().take().unwrap_or_default())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        CAPTURE.with(|c| *c.borrow_mut() = None);
    }
}

/// Re-read every recorded input. Some(provenance, exactly as the evaluation
/// would have recorded it) when every one resolves to the recorded bytes;
/// None on any difference or any read failure (the evaluation then runs and
/// reports the failure as it always has).
pub(crate) fn inputs_unchanged(rec: &Held, resolver: &Resolver) -> Option<Vec<Fetched>> {
    let mut prov = Vec::new();
    let mut seen: BTreeMap<&str, Option<Fetched>> = BTreeMap::new();
    for (hash, reference) in &rec.inputs {
        let p = match seen.get(reference.as_str()) {
            Some(p) => p.clone(),
            None => {
                let r = resolver.resolve(reference).ok()?;
                if blake3::hash(r.text.as_bytes()).to_hex().as_str() != hash {
                    return None;
                }
                seen.insert(reference, r.provenance.clone());
                r.provenance
            }
        };
        if let Some(p) = p {
            prov.push(p);
        }
    }
    Some(prov)
}

/// Where hold records may come from in one run.
#[derive(Debug, Default)]
pub struct Plan {
    /// Lock records may be reused (not under `--ci` without a verified lock
    /// signature, ruling I-D4).
    pub from_lock: bool,
    /// Cache records (empty under `--ci`, ruling I-D4).
    pub cache: BTreeMap<String, Held>,
}

pub const CACHE_DIR: &str = ".invar/cache";
pub const CACHE_FILE: &str = ".invar/cache/verdicts.json";

#[derive(Serialize, Deserialize)]
struct CacheFile {
    format: String,
    records: BTreeMap<String, Held>,
}

/// The local cache under `base`. Missing, unreadable or malformed: empty,
/// never an error (it is not authoritative; the run evaluates instead).
pub fn load_cache(base: &Path) -> BTreeMap<String, Held> {
    std::fs::read_to_string(base.join(CACHE_FILE))
        .ok()
        .and_then(|t| serde_json::from_str::<CacheFile>(&t).ok())
        .filter(|c| c.format == "1")
        .map(|c| c.records)
        .unwrap_or_default()
}

/// Replace the cache with this run's records: a temp file renamed into
/// place, so a reader never sees half a file. The directory ignores itself
/// in git.
pub fn save_cache(base: &Path, records: &BTreeMap<String, Held>) -> std::io::Result<()> {
    let dir = base.join(CACHE_DIR);
    std::fs::create_dir_all(&dir)?;
    let ignore = dir.join(".gitignore");
    if std::fs::read_to_string(&ignore).ok().as_deref() != Some("*\n") {
        std::fs::write(&ignore, "*\n")?;
    }
    let body = serde_json::to_string_pretty(&CacheFile { format: "1".into(), records: records.clone() })
        .map_err(std::io::Error::other)?;
    let tmp = dir.join(format!("verdicts.json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, body + "\n")?;
    std::fs::rename(&tmp, base.join(CACHE_FILE))
}
