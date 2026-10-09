//! `invar mcp`: a read-only MCP server on stdio for coding agents.
//!
//! Two tools, no arguments, both about the `invar.toml` the server was started
//! with:
//!   * `check`: the same evaluation as `invar check` (with `invar.lock` and its
//!     waivers), as JSON. Skipped and errored contracts are stated, never
//!     reported as passing.
//!   * `contract`: what this repository depends on, from `invar.lock`: each
//!     locked contract's producer, hash, t_r, bound `Type.field`s with types
//!     and t_v, and the waivers with days left. No network.
//!
//! JSON-RPC 2.0, one message per line. The server never writes files.

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
        let skipped = outcomes.iter().filter(|o| o.skipped.is_some()).count();
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
                    "instructions": "Read-only. `check` evaluates this repository's cross-repository contracts; `contract` lists the producer fields it depends on (invar.lock). A violation is contract residue, not proof of a runtime failure; a skipped contract was not evaluated.",
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
            ]})),
            "tools/call" => {
                let name = msg
                    .pointer("/params/name")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let r = match name {
                    "check" => self.check(),
                    "contract" => self.contract(),
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
