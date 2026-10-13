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

use crate::check::{evaluate_with, ContractOutcome};
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

pub fn outcome_json(o: &ContractOutcome) -> Value {
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
        Ok(json!({
            "result": result,
            "contracts": outcomes.iter().map(outcome_json).collect::<Vec<_>>(),
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
