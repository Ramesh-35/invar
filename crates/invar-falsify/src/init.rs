//! `invar init`: zero-touch topology discovery.
//!
//! Scans a workspace and emits an `invar.toml`. HONEST SCOPE: discovery is
//! HEURISTIC. It recognizes files by name and content patterns; the only real
//! parsing is JSON (serde_json). It proposes runnable contracts only where it
//! can actually run them today: a JSON Schema, or an OpenAPI-JSON component
//! schema, paired with a source file that reads at least one of that schema's
//! field names via `payload.<field>`. Everything else it finds (YAML specs,
//! AsyncAPI, protobuf, framework routes, Pydantic models) is reported as
//! inventory, explicitly NOT checkable yet. Every generated block is a
//! proposal for a human to review, never an asserted contract.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::invariant::payload_reads;

const SKIP_DIRS: &[&str] = &[
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
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub files_scanned: usize,
    pub truncated: bool,
    pub schemas: Vec<SchemaFound>,
    pub consumers: Vec<ConsumerFound>,
    pub inventory: Vec<InventoryItem>,
    pub proposals: Vec<Proposal>,
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, truncated: &mut bool) {
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
                walk(&p, out, truncated);
            }
        } else if md.is_file() && md.len() <= MAX_FILE_BYTES {
            out.push(p);
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
    walk(root, &mut files, &mut truncated);
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

    for s in &d.schemas {
        for c in &d.consumers {
            let overlap: BTreeSet<String> = s.fields.intersection(&c.reads).cloned().collect();
            if overlap.is_empty() {
                continue;
            }
            let id = format!("schema:{}:{}->{}", file_name(&s.path), s.label, file_name(&c.path));
            if ![&id, &s.path, &c.path].iter().all(|v| toml_safe(v))
                || !s.pointer.as_deref().map_or(true, toml_safe)
            {
                d.inventory.push(InventoryItem {
                    category: "skipped",
                    location: format!("{} -> {}", s.path, c.path),
                    note: "path not representable in flat TOML".into(),
                });
                continue;
            }
            if d.proposals.len() >= MAX_PROPOSALS {
                d.truncated = true;
                break;
            }
            d.proposals.push(Proposal {
                id,
                producer: s.path.clone(),
                pointer: s.pointer.clone(),
                consumer: c.path.clone(),
                overlap,
            });
        }
    }
    d
}

/// Render the discovery as an `invar.toml` the flat parser can read back.
pub fn render_manifest(d: &Discovery) -> String {
    let mut s = String::new();
    s.push_str("# invar.toml generated by `invar init`.\n#\n");
    s.push_str("# HONEST SCOPE: discovery is HEURISTIC (file names and text patterns; only\n");
    s.push_str("# JSON is actually parsed). Each [[invariant]] block below is a PROPOSAL,\n");
    s.push_str("# paired because the consumer reads at least one field the schema names.\n");
    s.push_str("# Review every block before trusting a verdict. Paths are relative to this\n");
    s.push_str("# file. A consumer in another repo can be referenced remotely under --ci as\n");
    s.push_str("# github:OWNER/REPO@REF:PATH (REF may be {base}).\n\n");

    s.push_str("[discovery]\n");
    s.push_str(&format!("files_scanned = \"{}\"\n", d.files_scanned));
    s.push_str(&format!("truncated = \"{}\"\n", d.truncated));
    s.push_str(&format!("schemas_found = \"{}\"\n", d.schemas.len()));
    s.push_str(&format!("consumers_found = \"{}\"\n", d.consumers.len()));
    s.push_str(&format!("contracts_proposed = \"{}\"\n\n", d.proposals.len()));

    for p in &d.proposals {
        s.push_str("[[invariant]]\n");
        s.push_str("kind = \"schema\"\n");
        s.push_str(&format!("id = \"{}\"\n", p.id));
        s.push_str(&format!("producer = \"{}\"\n", p.producer));
        if let Some(ptr) = &p.pointer {
            s.push_str(&format!("pointer = \"{ptr}\"\n"));
        }
        s.push_str(&format!("consumer = \"{}\"\n", p.consumer));
        let ov: Vec<&str> = p.overlap.iter().map(String::as_str).collect();
        s.push_str(&format!("# proposed because both mention: {}\n\n", ov.join(", ")));
    }

    if d.proposals.is_empty() {
        s.push_str("# No runnable contract could be proposed. Template:\n");
        s.push_str("# [[invariant]]\n# kind = \"schema\"          # schema | event | lock_order\n");
        s.push_str("# id = \"my-contract\"\n# producer = \"path/to/schema.json\"\n");
        s.push_str("# consumer = \"path/to/client.js\"\n\n");
    }

    s.push_str("# ---- Found but NOT yet checkable (inventory only) ----\n");
    if d.inventory.is_empty() {
        s.push_str("# (none)\n");
    }
    for i in &d.inventory {
        s.push_str(&format!("# {:<9} {}  {}\n", i.category, i.location, i.note));
    }
    s
}
