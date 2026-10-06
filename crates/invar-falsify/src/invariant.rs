//! General invariant-authoring engine (cross-repository contract kinds).
//!
//! Three kinds share one shape: a PRODUCER provides a surface, a CONSUMER
//! depends on part of it, and the residue is what the consumer depends on that
//! the producer does not provide (or an ordering the trace violates). Each
//! checker is deterministic and pure.
//!
//! HONEST SCOPE: these checkers expand the product surface, not epistemic
//! adequacy. Adequacy is SYNTHETIC until a kind is run on a REAL incident. Only
//! `SchemaContract` has a real incident so far (REAL-INCIDENT-001).
//! `EventContract` and `LockOrderContract` are synthetic-tested only. An empty
//! residue means the contract holds; a non-empty residue names the violation.

use std::collections::BTreeSet;

use crate::reality::differential_residue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    SchemaContract,
    EventContract,
    LockOrderContract,
}

impl ContractKind {
    pub fn parse(s: &str) -> Option<ContractKind> {
        match s {
            "SchemaContract" => Some(ContractKind::SchemaContract),
            "EventContract" => Some(ContractKind::EventContract),
            "LockOrderContract" => Some(ContractKind::LockOrderContract),
            _ => None,
        }
    }
}

/// Select a sub-document by JSON pointer (e.g. `/components/schemas/Order`
/// inside an OpenAPI JSON file) and re-serialize it, so the schema checker can
/// run on one component of a larger spec.
pub fn select_pointer(json: &str, pointer: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("not valid JSON: {e}"))?;
    let sub = v
        .pointer(pointer)
        .ok_or_else(|| format!("JSON pointer {pointer:?} not found"))?;
    serde_json::to_string(sub).map_err(|e| e.to_string())
}

/// The fields a source reads off `payload`, e.g. `payload.amount_cents`.
/// Only this access pattern is recognized (not `payload["x"]` or `.get("x")`).
pub fn payload_reads(source: &str) -> BTreeSet<String> {
    let mut read = BTreeSet::new();
    let bytes = source.as_bytes();
    let needle = "payload.";
    let mut i = 0usize;
    while let Some(pos) = source[i..].find(needle) {
        let start = i + pos + needle.len();
        let mut end = start;
        while end < source.len() {
            let c = bytes[end];
            if c.is_ascii_alphanumeric() || c == b'_' {
                end += 1;
            } else {
                break;
            }
        }
        if end > start {
            read.insert(source[start..end].to_string());
        }
        i = start.max(i + pos + 1);
    }
    read
}

fn json_object_keys(json: &str) -> BTreeSet<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v.as_object().map(|o| o.keys().cloned().collect()))
        .unwrap_or_default()
}

/// SchemaContract: consumer-read payload fields the producer schema does not
/// DEFINE as properties. Reuses the schema differential probe.
pub fn schema_contract(producer_schema_json: &str, consumer_source: &str) -> Vec<String> {
    differential_residue(producer_schema_json, consumer_source)
}

/// EventContract: fields a consumer handler reads off the event payload that the
/// producer does not EMIT (emitted = top-level keys of the emitted JSON payload).
pub fn event_contract(producer_payload_json: &str, consumer_handler_source: &str) -> Vec<String> {
    let emitted = json_object_keys(producer_payload_json);
    payload_reads(consumer_handler_source)
        .difference(&emitted)
        .cloned()
        .collect()
}

/// One observed step in a cross-service trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub actor: String,
    pub action: String,
    pub resource: String,
}

impl Step {
    pub fn new(actor: &str, action: &str, resource: &str) -> Step {
        Step {
            actor: actor.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
        }
    }
}

/// LockOrderContract: assert `before` happens before `after` in the observed
/// trace. Residue is non-empty when `after` precedes `before` (inversion) or
/// when `after` occurs with no preceding `before` (missing precondition). If
/// the guarded `after` never occurs, there is nothing to violate.
pub fn lock_order_contract(before: &Step, after: &Step, observed: &[Step]) -> Vec<String> {
    let pos = |s: &Step| observed.iter().position(|o| o == s);
    match (pos(before), pos(after)) {
        (_, None) => Vec::new(),
        (Some(b), Some(a)) if b < a => Vec::new(),
        (Some(_), Some(_)) => vec![format!(
            "ORDER_INVERSION: {}:{} on {} occurred before {}:{} on {}",
            after.actor, after.action, after.resource, before.actor, before.action, before.resource
        )],
        (None, Some(_)) => vec![format!(
            "MISSING_PRECONDITION: {}:{} on {} occurred with no preceding {}:{} on {}",
            after.actor, after.action, after.resource, before.actor, before.action, before.resource
        )],
    }
}

/// Dispatch input for the generalized engine.
pub enum ContractInput<'a> {
    Schema {
        producer_schema: &'a str,
        consumer_source: &'a str,
    },
    Event {
        producer_payload: &'a str,
        consumer_handler: &'a str,
    },
    LockOrder {
        before: Step,
        after: Step,
        observed: Vec<Step>,
    },
}

/// Generalized differential engine: dispatch evidence to the checker for its
/// declared kind. Returns the residue (empty = contract holds).
pub fn check_contract(input: &ContractInput) -> Vec<String> {
    match input {
        ContractInput::Schema {
            producer_schema,
            consumer_source,
        } => schema_contract(producer_schema, consumer_source),
        ContractInput::Event {
            producer_payload,
            consumer_handler,
        } => event_contract(producer_payload, consumer_handler),
        ContractInput::LockOrder {
            before,
            after,
            observed,
        } => lock_order_contract(before, after, observed),
    }
}
