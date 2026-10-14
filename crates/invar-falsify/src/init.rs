//! `invar init`: zero-touch topology discovery.
//!
//! Scans a workspace and emits an `invar.toml`. HONEST SCOPE: discovery is
//! HEURISTIC. Since INIT-DISCOVERY slice 1 it proposes contracts only between
//! TypeScript code producers and consumers in different qualifying workspace
//! packages (see `codepairs`), each with its evidence and the status line
//! "PROPOSED (heuristic; receiver of field reads not verified)". Schema files
//! (JSON Schema, OpenAPI-JSON components) are INVENTORY ONLY (ruling H1): they
//! are listed, never proposed. Everything else it finds (YAML specs,
//! AsyncAPI, protobuf, framework routes, Pydantic models) is inventory too.
//! What init cannot resolve is an explicit ABSTAIN line, never a guess.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::invariant::payload_reads;

pub(crate) const SKIP_DIRS: &[&str] = &[
    ".git", "target", "node_modules", "dist", "build", ".venv", "venv", "__pycache__",
    "vendor", ".next", ".idea", ".vscode",
];
const MAX_FILE_BYTES: u64 = 1_000_000;
const MAX_FILES: usize = 20_000;
const MAX_PROPOSALS: usize = 200;
const SOURCE_EXTS: &[&str] = &[
    "js", "jsx", "ts", "tsx", "mjs", "cjs", "py", "rs", "go", "java", "kt",
];

#[derive(Debug, Clone)]
pub struct SchemaFound {
    pub path: String,
    pub pointer: Option<String>,
    pub label: String,
    pub fields: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct ConsumerFound {
    pub path: String,
    pub reads: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub category: &'static str,
    pub location: String,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: String,
    pub producer: String,
    pub pointer: Option<String>,
    pub consumer: String,
    pub overlap: BTreeSet<String>,
    /// Comment lines (without `# `) written above the block: E1 to E6,
    /// status, category, lock note.
    pub evidence: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub files_scanned: usize,
    pub truncated: bool,
    pub schemas: Vec<SchemaFound>,
    pub consumers: Vec<ConsumerFound>,
    pub inventory: Vec<InventoryItem>,
    pub proposals: Vec<Proposal>,
    /// Qualifying workspace packages (slice 1).
    pub packages: usize,
    /// Named imports examined that point at another qualifying package.
    pub candidates: usize,
    pub abstentions: Vec<crate::codepairs::Abstention>,
    /// A repository-level abstention (H2, SLICE-SCOPE) stopped discovery.
    pub repo_abstained: bool,
}

/// The status reported in the summary line.
pub fn status(d: &Discovery) -> &'static str {
    if d.repo_abstained {
        "ABSTAINED"
    } else if d.truncated {
        "TRUNCATED"
    } else {
        "COMPLETE"
    }
}

/// The summary line (without `# `), always the last line of the manifest.
pub fn summary(d: &Discovery) -> String {
    format!(
        "SUMMARY packages={} candidates={} proposals={} abstentions={} inventory={} status={}",
        d.packages,
        d.candidates,
        d.proposals.len(),
        d.abstentions.len(),
        d.inventory.len(),
        status(d)
    )
}

pub const PROPOSAL_STATUS: &str = "status: PROPOSED (heuristic; receiver of field reads not verified)";
pub const LOCK_NOTE: &str = "lock: run `invar lock` before `invar check`. Without a lock, check evaluates only payload.<field> reads and says nothing about this proposal (a HOLDS would be vacuous). `invar lock` binds every field of the producer file that the consumer mentions in any form, which is wider than the evidence above.";

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, truncated: &mut bool, oversized: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(_) => return,
    };
    entries.sort();
    for p in entries {
        if out.len() >= MAX_FILES {
            *truncated = true;
            return;
        }
        let md = match std::fs::symlink_metadata(&p) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if md.file_type().is_symlink() {
            continue;
        }
        if md.is_dir() {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if !SKIP_DIRS.contains(&name.as_str()) {
                walk(&p, out, truncated, oversized);
            }
        } else if md.is_file() && md.len() <= MAX_FILE_BYTES {
            out.push(p);
        } else if md.is_file() {
            oversized.push(p);
        }
    }
}

fn fields_of(v: &serde_json::Value) -> BTreeSet<String> {
    let mut f = BTreeSet::new();
    if let Some(props) = v.get("properties").and_then(|p| p.as_object()) {
        f.extend(props.keys().cloned());
    }
    if let Some(req) = v.get("required").and_then(|r| r.as_array()) {
        f.extend(req.iter().filter_map(|x| x.as_str().map(String::from)));
    }
    f
}

fn escape_pointer(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

/// Values the flat-TOML parser can round-trip (no quotes or line breaks).
fn toml_safe(s: &str) -> bool {
    !s.contains('"') && !s.contains('\n') && !s.contains('\r')
}

fn quoted_after(line: &str, marker: &str) -> Option<String> {
    let i = line.find(marker)? + marker.len();
    let rest = &line[i..];
    let (start, q) = rest.char_indices().find(|(_, c)| matches!(c, '"' | '\'' | '`'))?;
    let body = &rest[start + q.len_utf8()..];
    let end = body.find(q)?;
    Some(body[..end].to_string())
}

fn scan_routes(path: &str, ext: &str, text: &str, inv: &mut Vec<InventoryItem>) {
    const VERBS: &[&str] = &["get", "post", "put", "delete", "patch"];
    let mut push = |n: usize, fw: &str, what: String| {
        inv.push(InventoryItem {
            category: if fw == "pydantic" { "model" } else { "route" },
            location: format!("{path}:{}", n + 1),
            note: format!("{fw} {what}"),
        })
    };
    for (n, line) in text.lines().enumerate() {
        match ext {
            "rs" => {
                if let Some(r) = quoted_after(line, ".route(") {
                    push(n, "axum", format!("{r:?}"));
                    continue;
                }
                for v in VERBS {
                    if let Some(r) = quoted_after(line, &format!("#[{v}(")) {
                        push(n, "actix", format!("{} {r:?}", v.to_uppercase()));
                    }
                }
                if let Some(r) = quoted_after(line, "web::resource(") {
                    push(n, "actix", format!("{r:?}"));
                }
            }
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => {
                for v in VERBS {
                    for obj in ["app", "router"] {
                        if let Some(r) = quoted_after(line, &format!("{obj}.{v}(")) {
                            push(n, "express", format!("{} {r:?}", v.to_uppercase()));
                        }
                    }
                }
            }
            "py" => {
                for v in VERBS {
                    for obj in ["app", "router"] {
                        if let Some(r) = quoted_after(line, &format!("@{obj}.{v}(")) {
                            push(n, "fastapi", format!("{} {r:?}", v.to_uppercase()));
                        }
                    }
                }
                let t = line.trim_start();
                if t.starts_with("class ") && t.contains("(BaseModel)") {
                    let name = t["class ".len()..].split('(').next().unwrap_or("").trim();
                    push(n, "pydantic", name.to_string());
                }
            }
            _ => {}
        }
    }
}

fn file_name(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

/// Scan `root` and build the discovery inventory and contract proposals.
pub fn discover(root: &Path) -> Discovery {
    let mut files = Vec::new();
    let mut truncated = false;
    let mut oversized = Vec::new();
    walk(root, &mut files, &mut truncated, &mut oversized);
    let mut d = Discovery {
        files_scanned: files.len(),
        truncated,
        ..Default::default()
    };

    for p in &files {
        let r = rel(root, p);
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let text = match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(_) => continue, // binary or unreadable
        };

        match ext.as_str() {
            "json" => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if v.get("openapi").is_some() {
                        let mut any = false;
                        if let Some(schemas) =
                            v.pointer("/components/schemas").and_then(|s| s.as_object())
                        {
                            for (sname, sv) in schemas {
                                if sv.get("properties").is_some() {
                                    d.schemas.push(SchemaFound {
                                        path: r.clone(),
                                        pointer: Some(format!(
                                            "/components/schemas/{}",
                                            escape_pointer(sname)
                                        )),
                                        label: format!("openapi:{sname}"),
                                        fields: fields_of(sv),
                                    });
                                    any = true;
                                }
                            }
                        }
                        if !any {
                            d.inventory.push(InventoryItem {
                                category: "openapi",
                                location: r.clone(),
                                note: "OpenAPI JSON with no component schemas carrying properties".into(),
                            });
                        }
                    } else if v.get("asyncapi").is_some() {
                        d.inventory.push(InventoryItem {
                            category: "asyncapi",
                            location: r.clone(),
                            note: "AsyncAPI JSON; event-contract mapping not yet automated".into(),
                        });
                    } else if v.get("properties").and_then(|p| p.as_object()).is_some() {
                        d.schemas.push(SchemaFound {
                            path: r.clone(),
                            pointer: None,
                            label: "json-schema".into(),
                            fields: fields_of(&v),
                        });
                    }
                }
            }
            "yaml" | "yml" => {
                let spec_like = text
                    .lines()
                    .any(|l| l.starts_with("openapi:") || l.starts_with("asyncapi:"));
                if spec_like || name.starts_with("openapi") || name.starts_with("asyncapi") {
                    d.inventory.push(InventoryItem {
                        category: "yaml-spec",
                        location: r.clone(),
                        note: "OpenAPI/AsyncAPI YAML; not checkable (no YAML parser in the zero-dependency build)".into(),
                    });
                }
            }
            "proto" => d.inventory.push(InventoryItem {
                category: "proto",
                location: r.clone(),
                note: "protobuf; no proto contract checker yet".into(),
            }),
            _ => {}
        }

        if SOURCE_EXTS.contains(&ext.as_str()) {
            let reads = payload_reads(&text);
            if !reads.is_empty() {
                d.consumers.push(ConsumerFound { path: r.clone(), reads });
            }
            scan_routes(&r, &ext, &text, &mut d.inventory);
        }
    }

    // H1: schema files are inventory only; they are never paired.
    for sc in &d.schemas {
        d.inventory.push(InventoryItem {
            category: "schema",
            location: match &sc.pointer {
                Some(ptr) => format!("{}#{}", sc.path, ptr),
                None => sc.path.clone(),
            },
            note: "schema file; inventory only (H1), never proposed".into(),
        });
    }

    // Slice 1: TypeScript code pairs across qualifying workspace packages.
    let rel_files: Vec<String> = files.iter().map(|p| rel(root, p)).collect();
    let rel_over: Vec<String> = oversized.iter().map(|p| rel(root, p)).collect();
    let code = crate::codepairs::discover_code(root, &rel_files, &rel_over, SKIP_DIRS);
    d.packages = code.packages.len();
    d.candidates = code.candidates;
    d.repo_abstained = code.repo_abstained;
    d.truncated |= code.truncated;
    d.abstentions = code.abstentions;
    for pair in &code.pairs {
        let base_id = format!("code:{}->{}", file_name(&pair.producer), file_name(&pair.consumer));
        // alpha.29 uniqueness rule: deterministic `#2`, `#3` in discovery
        // order (byte order of producer, then consumer path).
        let mut id = base_id.clone();
        let mut n = 2;
        while d.proposals.iter().any(|p| p.id == id) {
            id = format!("{base_id}#{n}");
            n += 1;
        }
        if ![&id, &pair.producer, &pair.consumer].iter().all(|v| toml_safe(v)) {
            d.abstentions.push(crate::codepairs::Abstention {
                code: "UNREPRESENTABLE",
                location: format!("{} -> {}", pair.producer, pair.consumer).replace(['\n', '\r'], " "),
                reason: "path not representable in flat TOML; not proposed".into(),
            });
            continue;
        }
        if d.proposals.len() >= MAX_PROPOSALS {
            d.truncated = true;
            break;
        }
        let overlap: BTreeSet<String> = pair
            .shapes
            .iter()
            .flat_map(|s| s.fields.iter().map(|(f, _)| f.clone()))
            .collect();
        d.proposals.push(Proposal {
            id,
            producer: pair.producer.clone(),
            pointer: None,
            consumer: pair.consumer.clone(),
            overlap,
            evidence: evidence_lines(pair),
        });
    }
    d.abstentions.sort();
    d.abstentions.dedup();
    d
}

fn evidence_lines(p: &crate::codepairs::CodePair) -> Vec<String> {
    let pkg = |k: &crate::codepairs::Package| format!("{} at {}/package.json ({})", k.name, k.dir, k.marker);
    let mut v = vec![PROPOSAL_STATUS.to_string()];
    v.push(format!("E1 producer package {}", pkg(&p.producer_pkg)));
    for s in &p.shapes {
        v.push(format!("E2 {} declared at {}:{}", s.shape, p.producer, s.decl_line));
    }
    v.push(format!("E3 consumer package {}", pkg(&p.consumer_pkg)));
    for s in &p.shapes {
        let via = match &s.barrel {
            Some((b, l)) => format!(", re-exported by barrel {b}:{l} from {}", p.producer),
            None => String::new(),
        };
        v.push(format!(
            "E4 {} imported at {}:{} from {:?}, resolved to {}{via}",
            s.shape, p.consumer, s.import_line, s.specifier, s.resolved
        ));
    }
    for s in &p.shapes {
        let f: Vec<String> = s.fields.iter().map(|(n, l)| format!("{n} ({}:{l})", p.consumer)).collect();
        v.push(format!("E5 {} fields read: {}", s.shape, f.join(", ")));
        v.push(format!(
            "E5 {} stoplisted reads: {}",
            s.shape,
            if s.stoplisted.is_empty() { "(none)".to_string() } else { s.stoplisted.join(", ") }
        ));
    }
    v.push(format!("E6 pair {} -> {}", p.producer, p.consumer));
    v.push(format!("category: {}", p.category));
    v.push(LOCK_NOTE.to_string());
    v
}

/// Render the discovery as an `invar.toml` the flat parser can read back.
/// Inventory lines shown by default; the full list needs --full-inventory.
/// A 1,600-comment-line config is a dump, not a disclosure (10-repo cold
/// audit, 2026-10-12: fastapi produced 1,614 inventory lines).
pub const INVENTORY_CAP: usize = 40;

pub fn render_manifest(d: &Discovery) -> String {
    render_manifest_opts(d, false)
}

pub fn render_manifest_opts(d: &Discovery, full_inventory: bool) -> String {
    let mut s = String::new();
    s.push_str("# invar.toml generated by `invar init`.\n#\n");
    s.push_str("# HONEST SCOPE: discovery is HEURISTIC (text patterns, not a parser). Each\n");
    s.push_str("# [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one\n");
    s.push_str("# workspace package, imported by name in another, with at least 2 of its\n");
    s.push_str("# direct fields read there. The receiver of those reads is NOT verified.\n");
    s.push_str("# Review every block before trusting a verdict. Schema files are inventory\n");
    s.push_str("# only. ABSTAIN lines are refusals to propose, not negative findings. Paths\n");
    s.push_str("# are relative to this file.\n\n");

    s.push_str("[discovery]\n");
    s.push_str(&format!("files_scanned = \"{}\"\n", d.files_scanned));
    s.push_str(&format!("truncated = \"{}\"\n", d.truncated));
    s.push_str(&format!("packages = \"{}\"\n", d.packages));
    s.push_str(&format!("candidates = \"{}\"\n", d.candidates));
    s.push_str(&format!("contracts_proposed = \"{}\"\n", d.proposals.len()));
    s.push_str(&format!("abstentions = \"{}\"\n", d.abstentions.len()));
    s.push_str(&format!("inventory = \"{}\"\n\n", d.inventory.len()));

    for p in &d.proposals {
        s.push_str(&format!("# PROPOSED {}\n", p.id));
        for e in &p.evidence {
            s.push_str(&format!("# {e}\n"));
        }
        s.push_str("[[invariant]]\n");
        s.push_str("kind = \"schema\"\n");
        s.push_str(&format!("id = \"{}\"\n", p.id));
        s.push_str(&format!("producer = \"{}\"\n", p.producer));
        if let Some(ptr) = &p.pointer {
            s.push_str(&format!("pointer = \"{ptr}\"\n"));
        }
        s.push_str(&format!("consumer = \"{}\"\n\n", p.consumer));
    }

    if d.proposals.is_empty() {
        s.push_str("# No contracts proposed. The ABSTAIN lines below say where init declined.\n");
        s.push_str("# To name a pair by hand (local path or remote ref github:OWNER/REPO@REF:PATH):\n");
        s.push_str("# [[invariant]]\n# kind = \"schema\"          # schema | event | lock_order\n");
        s.push_str("# id = \"my-contract\"\n# producer = \"path/to/schema.json\"\n");
        s.push_str("# consumer = \"path/to/client.js\"\n\n");
    }

    s.push_str("# ---- Abstentions (init declined to propose; not negative findings) ----\n");
    if d.abstentions.is_empty() {
        s.push_str("# (none)\n");
    }
    for a in &d.abstentions {
        s.push_str(&format!("# {}\n", a.line().replace(['\n', '\r'], " ")));
    }

    s.push_str("# ---- Found but NOT proposed (inventory only) ----\n");
    let mut inv: Vec<String> = d
        .inventory
        .iter()
        .map(|i| format!("INVENTORY {} {}  {}", i.category, i.location, i.note).replace(['\n', '\r'], " "))
        .collect();
    inv.sort();
    if inv.is_empty() {
        s.push_str("# (none)\n");
    }
    let shown = if full_inventory { inv.len() } else { inv.len().min(INVENTORY_CAP) };
    for i in &inv[..shown] {
        s.push_str(&format!("# {i}\n"));
    }
    if shown < inv.len() {
        s.push_str(&format!(
            "# ... {} more item(s) not shown; run `invar init --stdout --full-inventory` for the complete list\n",
            inv.len() - shown
        ));
    }
    s.push_str(&format!("# {}\n", summary(d)));
    s
}
