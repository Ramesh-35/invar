//! `invar mcp`: a read-only MCP server on stdio for coding agents.
//!
//! Four tools, all read-only, about the `invar.toml` the server was started
//! with:
//!   * `check`: the same evaluation as `invar check` (with `invar.lock` and its
//!     waivers), as JSON. Skipped and errored contracts are stated, never
//!     reported as passing.
//!   * `contract`: what this repository depends on, from `invar.lock`: each
//!     locked contract's producer, hash, t_r, bound `Type.field`s with types
//!     and t_v, and the waivers with days left. No network.
//!   * `dependents`: who depends on a producer field, from invar.lock files
//!     on local disk (the A4 primitive). Same rendering path as
//!     `invar dependents --format json`; zero network by construction.
//!   * `coverage`: per-contract evaluation disclosure, never a verdict. Same
//!     rendering path as `invar coverage --format json`.
//!
//! JSON-RPC 2.0, one message per line. The server never writes files; adding
//! a write-capable tool (e.g. init) would revoke that stated guarantee and
//! needs its own ratification first.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::check::{evaluate_with, ContractOutcome, Witness};
use crate::lock::{self, Lock};
use crate::resolver::{Resolver, Transport};

const PROTOCOLS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

pub struct Server<'a> {
    pub config: PathBuf,
    /// Live fetches for `github:` artifacts; `None` reads local files only.
    pub transport: Option<&'a dyn Transport>,
    pub token: Option<String>,
    pub today: i64,
    /// The binary's MEASURED_RECORD line, passed in so the coverage tool
    /// states the same record as `invar coverage` (one source, main.rs).
    pub measured_record: &'static str,
}

fn base_dir(config: &Path) -> PathBuf {
    config
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn load_lock(config: &Path) -> Result<Option<Lock>> {
    let p = base_dir(config).join("invar.lock");
    if !p.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&p)?;
    Ok(Some(lock::parse(&p.display().to_string(), &text)?))
}

// ------------------------------------------------------------- JSON-1
// docs/falsification/JSON-SPEC.md (AGENT-1 P1, ruling JSON-D1): the
// structured fields are derived from the outcome the engine already built;
// nothing here changes a verdict or a line of text output.

/// R-1: the rule a witness convicts under.
fn rule_of(w: &Witness, kind: &str) -> &'static str {
    if kind == "lock_order" {
        "lock-order"
    } else if w.missing_from.as_deref().is_some_and(|m| rs1_writes(m).is_some()) {
        "request-side"
    } else if w.missing_from.is_some() {
        "absent"
    } else {
        "retyped"
    }
}

/// The written lines of the RS-1 form (`... at head: no inbound read and no
/// literal of K remain (...); written at N:L1, L2`), as (N, [L1, L2]).
fn rs1_writes(m: &str) -> Option<(&str, Vec<u64>)> {
    if !m.contains(" at head: no inbound read and no literal of ") {
        return None;
    }
    let (_, at) = m.rsplit_once("; written at ")?;
    let (name, lines) = at.rsplit_once(':')?;
    let lines: Vec<u64> = lines.split(", ").map(|l| l.trim().parse().ok()).collect::<Option<_>>()?;
    Some((name, lines))
}

/// R-2: consumer sites as {file, line, access}.
fn consumer_sites(w: &Witness, kind: &str) -> Vec<Value> {
    let access = if kind == "lock_order" { "trace" } else { "read" };
    let mut out: Vec<Value> = w
        .sites
        .iter()
        .filter_map(|s| s.rsplit_once(':'))
        .filter_map(|(f, l)| l.parse::<u64>().ok().map(|l| json!({"file": f, "line": l, "access": access})))
        .collect();
    if let Some((name, lines)) = w.missing_from.as_deref().and_then(rs1_writes) {
        out.extend(lines.into_iter().map(|l| json!({"file": name, "line": l, "access": "write"})));
    }
    out
}

/// R-3: the bound producers the witness places the field in: for each place,
/// the longest bound name it equals or starts with, followed by a space or a
/// colon. Never a guess: no match contributes nothing.
fn producers_of(w: &Witness, bound: &[String]) -> Vec<String> {
    let mut places: Vec<&str> = Vec::new();
    if let Some(m) = &w.missing_from {
        let head = m.split(" · ").next().unwrap_or(m);
        places.extend(head.split(", "));
    }
    if let Some((_, _, at)) = &w.type_change {
        places.push(at);
    }
    let mut out: Vec<String> = Vec::new();
    for p in places {
        let hit = bound
            .iter()
            .filter(|n| p == n.as_str() || p.strip_prefix(n.as_str()).is_some_and(|r| r.starts_with(' ') || r.starts_with(':')))
            .max_by_key(|n| n.len());
        if let Some(n) = hit {
            if !out.contains(n) {
                out.push(n.clone());
            }
        }
    }
    out
}

/// A-1, S-1: the stable reason code of a not-evaluated statement or a skip.
/// The text's own `[not-evaluated:CODE]` tag first; then the engine's fixed
/// untagged sentences, enumerated in JSON-SPEC B; else `unclassified`.
pub fn reason_code(text: &str) -> String {
    if let Some(code) = text.rsplit_once("[not-evaluated:").and_then(|(_, t)| t.strip_suffix(']')) {
        return code.to_string();
    }
    let code = if text.ends_with(", not evaluated (pattern not read)") {
        "pattern-not-read"
    } else if text.contains(" moved to ") && text.contains("; consumer read at ") && text.ends_with(" not evaluated") {
        "field-moved"
    } else if text.starts_with("the lock binds no field of this contract (0 bindings)") {
        "no-bindings"
    } else if text.starts_with("language not supported (") {
        "unsupported-language"
    } else if text.starts_with("not evaluated: no invar.lock entry") {
        "no-lock-entry"
    } else if text.starts_with("not evaluated: inline list ") {
        "list-unrecognizable"
    } else if text.starts_with("not evaluated: list ") && text.contains(" is no longer statically extractable") {
        "list-not-extractable"
    } else if text.starts_with("not evaluated: list ") && text.contains(" no longer found in ") {
        "list-not-found"
    } else if text.starts_with("not evaluated: no shape ") {
        "shape-not-found"
    } else if text.starts_with("not evaluated: ") && text.contains(" is no longer statically extractable") {
        "shape-not-extractable"
    } else {
        "unclassified"
    };
    code.to_string()
}

/// The artifact names a contract block binds as producers.
fn bound_producers(block: &[(String, String)]) -> Vec<String> {
    ["producer", "producer_base", "producer_code", "producer_code_base"]
        .iter()
        .filter_map(|k| crate::check::getk(block, k))
        .map(|v| crate::check::artifact_name(&v))
        .collect()
}

pub fn outcome_json(o: &ContractOutcome) -> Value {
    outcome_json_bound(o, &[])
}

/// `outcome_json` with the contract's bound producer names (R-3).
pub fn outcome_json_bound(o: &ContractOutcome, bound: &[String]) -> Value {
    let mut v = outcome_json_base(o);
    if let Some(items) = v["residue"].as_array_mut() {
        for (item, w) in items.iter_mut().zip(&o.witnesses) {
            item["rule"] = json!(rule_of(w, &o.kind));
            item["consumer_sites"] = json!(consumer_sites(w, &o.kind));
            item["producers"] = json!(producers_of(w, bound));
            item["candidate_rename"] = match w.missing_from.as_deref().and_then(crate::check::rename_candidate) {
                Some((_, to, ty)) => json!({"key": to, "type": ty}),
                None => Value::Null,
            };
        }
    }
    v["not_evaluated_items"] = json!(o.unevaluated.iter().map(|u| json!({"reason": reason_code(u), "text": u})).collect::<Vec<_>>());
    v["skip"] = match &o.skipped {
        Some(s) => json!({"reason": reason_code(s), "text": s}),
        None => Value::Null,
    };
    v
}

fn outcome_json_base(o: &ContractOutcome) -> Value {
    let status = if o.skipped.is_some() {
        "skipped"
    } else if o.violated() {
        "violated"
    } else if !o.unevaluated.is_empty() {
        "not_evaluated"
    } else if !o.waived.is_empty() {
        "holds_waived"
    } else {
        "holds"
    };
    json!({
        "id": o.id,
        "kind": o.kind,
        "status": status,
        "skipped_reason": o.skipped,
        "residue": o.witnesses.iter().map(|w| json!({
            "field": w.residue,
            "read_at": w.sites,
            "missing_from": w.missing_from,
            "type_change": w.type_change.as_ref().map(|(a, b, at)| json!({"from": a, "to": b, "in": at})),
            "read_as": w.read_as,
        })).collect::<Vec<_>>(),
        "waived": o.waived.iter().map(|(f, owner, reason, left)| json!({
            "field": f, "owner": owner, "reason": reason, "days_remaining": left
        })).collect::<Vec<_>>(),
        "expired_waivers": o.expired.iter().map(|(f, owner)| json!({"field": f, "owner": owner})).collect::<Vec<_>>(),
        "not_evaluated": o.unevaluated,
        "notes": o.notes,
        "fetched": o.provenance.iter().map(|p| json!({"ref": p.reference, "bytes": p.bytes, "blake3": p.blake3_hex})).collect::<Vec<_>>(),
    })
}

impl Server<'_> {
    fn resolver(&self) -> Resolver<'_> {
        Resolver {
            base_dir: base_dir(&self.config),
            allow_remote: self.transport.is_some(),
            token: self.token.clone(),
            base_ref: None,
            transport: self.transport,
            record_local: false,
        }
    }

    pub fn check(&self) -> Result<Value> {
        let raw = std::fs::read_to_string(&self.config)
            .map_err(|e| anyhow!("reading {}: {e}", self.config.display()))?;
        let sections = crate::config::parse_sections(&self.config.display().to_string(), &raw)
            .map_err(|e| anyhow!("{e}"))?;
        let lock = load_lock(&self.config)?;
        let outcomes = evaluate_with(&sections, &self.resolver(), lock.as_ref(), self.today)?;
        let violated = outcomes.iter().filter(|o| o.violated()).count();
        let skipped = outcomes
            .iter()
            .filter(|o| o.skipped.is_some() || (!o.violated() && !o.unevaluated.is_empty()))
            .count();
        let result = if violated > 0 {
            "violated"
        } else if skipped > 0 {
            "not_fully_evaluated"
        } else {
            "all_hold"
        };
        let blocks = crate::check::gather_blocks(&sections).unwrap_or_default();
        let bound = |id: &str| -> Vec<String> {
            blocks
                .iter()
                .find(|b| crate::check::getk(b, "id").as_deref() == Some(id))
                .map(|b| bound_producers(b))
                .unwrap_or_default()
        };
        Ok(json!({
            "result": result,
            "exit_code": crate::receipt::contracts_exit(&outcomes),
            "contracts": outcomes.iter().map(|o| outcome_json_bound(o, &bound(&o.id))).collect::<Vec<_>>(),
        }))
    }

    pub fn contract(&self) -> Result<Value> {
        let lock = load_lock(&self.config)?.ok_or_else(|| {
            anyhow!(
                "no invar.lock beside {}; run `invar lock`",
                self.config.display()
            )
        })?;
        Ok(json!({
            "contracts": lock.contracts.iter().map(|c| json!({
                "id": c.id,
                "producer": c.producer,
                "pointer": c.pointer,
                "producer_code": c.producer_code,
                "blake3": c.blake3,
                "recorded_at": c.recorded_at,
                "bindings": c.bindings.iter().map(|b| json!({
                    "field": b.display(), "source": b.source, "type": b.ty, "since": b.since
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "waivers": lock.waivers.iter().map(|w| json!({
                "contract": w.contract, "field": w.field, "owner": w.owner,
                "reason": w.reason, "expires": w.expires,
                "days_remaining": w.days_left(self.today),
            })).collect::<Vec<_>>(),
        }))
    }

    /// A4 over MCP: who depends on this field, from invar.lock files under
    /// `locks`. One rendering path with the CLI (`dependents::render_json`),
    /// so the two outputs are the same JSON document for the same inputs.
    pub fn dependents(&self, args: &Value) -> Result<Value> {
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing required argument \"query\""))?;
        let locks = args
            .get("locks")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing required argument \"locks\""))?;
        let producer = args.get("producer").and_then(Value::as_str);
        let locks_path = {
            let p = Path::new(locks);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                base_dir(&self.config).join(p)
            }
        };
        // SECURITY-SPEC S2-5: only a path inside the configuration's
        // directory, symbolic links resolved.
        let inside = std::fs::canonicalize(&locks_path)
            .ok()
            .zip(std::fs::canonicalize(base_dir(&self.config)).ok())
            .is_some_and(|(l, b)| l.starts_with(b));
        if !inside {
            return Err(anyhow!("locks path is outside the configuration directory: {locks}"));
        }
        let a = crate::dependents::find(&locks_path, query, producer)?;
        Ok(serde_json::from_str(&crate::dependents::render_json(
            query, producer, &a,
        ))?)
    }

    /// Coverage disclosure over MCP: what INVAR evaluates here and what it
    /// does not, with reason codes; never a verdict. One rendering path with
    /// the CLI (`demo::run_coverage`).
    pub fn coverage(&self, args: &Value) -> Result<Value> {
        let contract = args.get("contract").and_then(Value::as_str);
        let mut out = String::new();
        crate::demo::run_coverage(
            &self.config.display().to_string(),
            contract,
            "json",
            self.measured_record,
            &mut out,
        )?;
        Ok(serde_json::from_str(&out)?)
    }

    /// One JSON-RPC message in, at most one out (notifications get none).
    pub fn handle(&self, msg: &Value) -> Option<Value> {
        let id = msg.get("id")?.clone();
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let ok = |result: Value| Some(json!({"jsonrpc": "2.0", "id": id, "result": result}));
        let err = |code: i64, m: &str| {
            Some(json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": m}}))
        };
        match method {
            "initialize" => {
                let asked = msg
                    .pointer("/params/protocolVersion")
                    .and_then(Value::as_str);
                let version = asked
                    .filter(|v| PROTOCOLS.contains(v))
                    .unwrap_or(PROTOCOLS[0]);
                ok(json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "invar", "version": env!("CARGO_PKG_VERSION")},
                    "instructions": "Read-only. `check` evaluates this repository's cross-repository contracts; `contract` lists the producer fields it depends on (invar.lock); `dependents` answers who depends on a producer field from local invar.lock files; `coverage` discloses what is and is not evaluated here. A violation is contract residue, not proof of a runtime failure; a skipped contract was not evaluated.",
                }))
            }
            "ping" => ok(json!({})),
            "tools/list" => ok(json!({"tools": [
                {
                    "name": "check",
                    "description": "Evaluate the contracts in invar.toml against invar.lock and its waivers. Returns per-contract status (holds, holds_waived, violated, skipped) with Type.field residue, read sites and reasons. Read-only.",
                    "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false},
                    "annotations": {"readOnlyHint": true},
                },
                {
                    "name": "contract",
                    "description": "List the producer fields this repository depends on, from invar.lock: bound Type.field with type and acceptance date, producer hash, and waivers with days remaining. Read-only, no network.",
                    "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false},
                    "annotations": {"readOnlyHint": true},
                },
                {
                    "name": "dependents",
                    "description": "Who depends on a producer field, from invar.lock files on local disk (any depth under `locks`). Query is `Type.field` or a bare `field`, matched by naming form (case and `_`/`-` insignificant). Zero matches is an answer, not an error; unreadable lockfiles are stated in parse_errors, never dropped. Read-only, zero network by construction.",
                    "inputSchema": {"type": "object", "properties": {
                        "query": {"type": "string", "description": "`Type.field` or a bare `field`"},
                        "locks": {"type": "string", "description": "Directory to scan for invar.lock files; a relative path resolves against the invar.toml directory"},
                        "producer": {"type": "string", "description": "Only contracts whose producer ref contains this substring"}
                    }, "required": ["query", "locks"], "additionalProperties": false},
                    "annotations": {"readOnlyHint": true},
                },
                {
                    "name": "coverage",
                    "description": "Per-contract evaluation disclosure for this repository: what INVAR evaluates here and what it does not, with reason codes and the measured record. Never a verdict. Read-only.",
                    "inputSchema": {"type": "object", "properties": {
                        "contract": {"type": "string", "description": "Only this contract id"}
                    }, "additionalProperties": false},
                    "annotations": {"readOnlyHint": true},
                },
            ]})),
            "tools/call" => {
                let name = msg
                    .pointer("/params/name")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let args = msg
                    .pointer("/params/arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let r = match name {
                    "check" => self.check(),
                    "contract" => self.contract(),
                    "dependents" => self.dependents(&args),
                    "coverage" => self.coverage(&args),
                    _ => return err(-32602, &format!("unknown tool {name:?}")),
                };
                ok(match r {
                    Ok(v) => json!({
                        "content": [{"type": "text", "text": v.to_string()}],
                        "structuredContent": v,
                        "isError": false,
                    }),
                    Err(e) => json!({
                        "content": [{"type": "text", "text": format!("ERROR (not a verdict): {e:#}")}],
                        "isError": true,
                    }),
                })
            }
            _ => err(-32601, &format!("method not found: {method}")),
        }
    }

    pub fn serve(&self, input: impl BufRead, mut output: impl Write) -> Result<()> {
        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let reply = match serde_json::from_str::<Value>(&line) {
                Ok(msg) => self.handle(&msg),
                Err(_) => Some(
                    json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}}),
                ),
            };
            if let Some(r) = reply {
                writeln!(output, "{r}")?;
                output.flush()?;
            }
        }
        Ok(())
    }
}
