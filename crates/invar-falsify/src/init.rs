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
    /// Machine-readable evidence keys, one per shape: "<Shape> <field>...".
    /// `invar lock` restricts a contract's bindings to these (directive
    /// 86428, pass 2); comments never reach the parser, these keys do.
    pub evidence_keys: Vec<String>,
    /// Manifest kind: "schema" (slice 1 and K2 pairs) or "egress" (slice 2 K1).
    pub kind: &'static str,
    /// Extra manifest keys written after the producer line (egress blocks:
    /// shape, boundary, eligibility). Empty for every slice 1 proposal.
    pub keys: Vec<(String, String)>,
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
    /// `[egress]` boundary markings read from an existing invar.toml; written
    /// back so a regenerated manifest keeps them (slice 2, R-K1-1).
    pub egress_marks: Vec<String>,
}

/// The status reported in the summary line.
pub fn status(d: &Discovery) -> &'static str {
    if d.repo_abstained && d.proposals.is_empty() {
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
        let evidence_keys: Vec<String> = pair
            .shapes
            .iter()
            .map(|s| {
                let mut k = s.shape.clone();
                for (f, _) in &s.fields {
                    k.push(' ');
                    k.push_str(f);
                }
                k
            })
            .collect();
        // The evidence travels as space-separated identifiers, so a shape or
        // field name with whitespace or a quote cannot be represented; such a
        // pair is abstained like any other unrepresentable one (M04 rule).
        let representable = [&id, &pair.producer, &pair.consumer].iter().all(|v| toml_safe(v))
            && pair.shapes.iter().all(|s| {
                let ok = |x: &str| toml_safe(x) && !x.chars().any(char::is_whitespace);
                ok(&s.shape) && s.fields.iter().all(|(f, _)| ok(f))
            });
        if !representable {
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
            evidence_keys,
            kind: "schema",
            keys: Vec::new(),
        });
    }

    // Slice 2: zod value producers (K2) and egress locks (K1). Appended after
    // every slice 1 proposal, so slice 1 output is a byte-identical prefix.
    // Slice 2 does not depend on workspace membership (an egress lock is a
    // single-repository self-contract), so slice 1's repository-level
    // abstention does not stop it.
    slice2(root, &rel_files, &rel_over, &mut d);
    // Slice 2b: string-coupling contracts (K3), appended after slice 2.
    slice2b(root, &rel_files, &rel_over, &mut d);
    d.abstentions.sort();
    d.abstentions.dedup();
    d
}

pub const EGRESS_STATUS: &str = "status: PROPOSED (heuristic; response-position reference found; runtime serialization not verified)";
pub const K2_STATUS: &str = "status: PROPOSED (heuristic; zod value producer, type-blind consumer; receiver of field reads not verified)";

fn unique_id(d: &Discovery, base: String) -> String {
    let mut id = base.clone();
    let mut n = 2;
    while d.proposals.iter().any(|p| p.id == id) {
        id = format!("{base}#{n}");
        n += 1;
    }
    id
}

fn slice2(root: &Path, rel_files: &[String], rel_over: &[String], d: &mut Discovery) {
    use crate::codepairs::Abstention;
    d.egress_marks = std::fs::read_to_string(root.join("invar.toml"))
        .map(|t| crate::valuepairs::egress_marks(&t))
        .unwrap_or_default();
    let vd = crate::valuepairs::discover_values(root, rel_files, rel_over, &d.egress_marks);
    d.abstentions.extend(vd.abstentions);
    let ok = |x: &str| toml_safe(x) && !x.chars().any(char::is_whitespace);
    for e in &vd.egress {
        let id = unique_id(d, format!("egress:{}", e.shape));
        let first = &e.boundaries[0];
        let representable = ok(&id)
            && ok(&e.producer)
            && ok(&e.shape)
            && e.boundaries.iter().all(|b| ok(&b.file) && ok(&b.eligibility))
            && e.fields.iter().all(|(f, _)| ok(f));
        if !representable {
            d.abstentions.push(Abstention {
                code: "UNREPRESENTABLE",
                location: format!("{}:{}", e.producer, e.decl_line).replace(['\n', '\r'], " "),
                reason: "path or name not representable in flat TOML; not proposed".into(),
            });
            continue;
        }
        if d.proposals.len() >= MAX_PROPOSALS {
            d.truncated = true;
            break;
        }
        let elig: BTreeSet<String> = e
            .boundaries
            .iter()
            .flat_map(|b| b.eligibility.split(',').map(|x| if x.starts_with("versioned-path") { "versioned-path".to_string() } else { "marked".to_string() }))
            .collect();
        d.proposals.push(Proposal {
            id,
            producer: e.producer.clone(),
            pointer: None,
            consumer: String::new(),
            overlap: e.fields.iter().map(|(f, _)| f.clone()).collect(),
            evidence: egress_lines(e),
            evidence_keys: Vec::new(),
            kind: "egress",
            keys: vec![
                ("shape".into(), e.shape.clone()),
                ("boundary".into(), first.file.clone()),
                ("eligibility".into(), elig.into_iter().collect::<Vec<_>>().join(",")),
            ],
        });
    }
    for p in &vd.pairs {
        let id = unique_id(d, format!("code:{}->{}", file_name(&p.producer), file_name(&p.consumer)));
        let evidence_keys: Vec<String> = p
            .shapes
            .iter()
            .map(|s| {
                let mut k = s.shape.clone();
                for (f, _) in &s.fields {
                    k.push(' ');
                    k.push_str(f);
                }
                k
            })
            .collect();
        let representable = [&id, &p.producer, &p.consumer].iter().all(|v| toml_safe(v))
            && p.shapes.iter().all(|s| ok(&s.shape) && s.fields.iter().all(|(f, _)| ok(f)));
        if !representable {
            d.abstentions.push(Abstention {
                code: "UNREPRESENTABLE",
                location: format!("{} -> {}", p.producer, p.consumer).replace(['\n', '\r'], " "),
                reason: "path not representable in flat TOML; not proposed".into(),
            });
            continue;
        }
        if d.proposals.len() >= MAX_PROPOSALS {
            d.truncated = true;
            break;
        }
        d.proposals.push(Proposal {
            id,
            producer: p.producer.clone(),
            pointer: None,
            consumer: p.consumer.clone(),
            overlap: p.shapes.iter().flat_map(|s| s.fields.iter().map(|(f, _)| f.clone())).collect(),
            evidence: k2_lines(p),
            evidence_keys,
            kind: "schema",
            keys: Vec::new(),
        });
    }
}

pub const STRINGS_STATUS: &str = "status: PROPOSED (heuristic; string list in a query position overlaps one shape; runtime store binding not verified)";

fn slice2b(root: &Path, rel_files: &[String], rel_over: &[String], d: &mut Discovery) {
    use crate::codepairs::Abstention;
    let sd = crate::valuepairs::strings::discover_strings(root, rel_files, rel_over);
    d.abstentions.extend(sd.abstentions);
    let ok = |x: &str| toml_safe(x) && !x.chars().any(char::is_whitespace);
    for p in &sd.proposals {
        let id = unique_id(d, format!("strings:{}@{}", p.shape, p.list_at));
        let representable = ok(&id)
            && ok(&p.producer)
            && ok(&p.shape)
            && ok(&p.list_file)
            && ok(&p.list_at)
            && p.overlap.iter().all(|(f, _, _)| ok(f));
        if !representable {
            d.abstentions.push(Abstention {
                code: "UNREPRESENTABLE",
                location: format!("{}:{}", p.list_file, p.list_line).replace(['\n', '\r'], " "),
                reason: "path or name not representable in flat TOML; not proposed".into(),
            });
            continue;
        }
        if d.proposals.len() >= MAX_PROPOSALS {
            d.truncated = true;
            break;
        }
        let names: Vec<String> = p.overlap.iter().map(|(f, _, _)| f.clone()).collect();
        d.proposals.push(Proposal {
            id,
            producer: p.producer.clone(),
            pointer: None,
            consumer: String::new(),
            overlap: names.iter().cloned().collect(),
            evidence: strings_lines(p),
            evidence_keys: Vec::new(),
            kind: "strings",
            keys: vec![
                ("shape".into(), p.shape.clone()),
                ("list".into(), p.list_file.clone()),
                ("listat".into(), p.list_at.clone()),
                ("scope".into(), p.scope.to_string()),
                ("overlap".into(), names.join(" ")),
            ],
        });
    }
}

fn strings_lines(p: &crate::valuepairs::strings::StringsProposal) -> Vec<String> {
    let names: Vec<&str> = p.overlap.iter().map(|(f, _, _)| f.as_str()).collect();
    let mut v = vec![STRINGS_STATUS.to_string()];
    v.push("S1 STRING-COUPLING contract: a string list that names fields of one shape; no consumer code is read and no runtime store binding is claimed".to_string());
    v.push(format!(
        "S2 {} declared at {}:{} with {} direct keys",
        p.shape,
        p.producer,
        p.shape_line,
        p.shape_keys.len()
    ));
    v.push(format!("S3 list {} at {}:{} holds {} string entries ({} distinctive after the stoplist)", p.list_at, p.list_file, p.list_line, p.list_len, p.distinctive));
    v.push(format!("S4 scope: {}", p.scope));
    let o: Vec<String> = p.overlap.iter().map(|(f, _, l)| format!("{f} ({}:{l})", p.list_file)).collect();
    v.push(format!("S5 distinctive overlap ({} of {} list entries): {}", names.len(), p.distinctive, o.join(", ")));
    v.push(format!(
        "S6 DISCLAIMER: that this list reaches the store the shape describes is not verified; a name overlap is not proof (display column lists and unrelated lists can overlap)"
    ));
    if !p.also.is_empty() {
        let sites: Vec<String> = p.also.iter().map(|(at, line, n)| format!("{at}@{line} |O|={n}")).collect();
        v.push(format!("S7 also: {} (other sites are not locked; add a block by hand to lock them)", sites.join("; ")));
    }
    v.push(format!("category: {}", p.category));
    v.push(format!(
        "lock: run `invar lock` before `invar check`. Without a lock a strings contract is SKIPPED (exit 4), never HOLDS. The lock binds exactly the S5 names; at check a locked name still in the list but no longer a key of {} is VIOLATED (stale reference); removal from the list, narrowing and additions HOLD.",
        p.shape
    ));
    v
}

fn egress_lines(e: &crate::valuepairs::EgressProposal) -> Vec<String> {
    let names: Vec<&str> = e.fields.iter().map(|(f, _)| f.as_str()).collect();
    let mut v = vec![EGRESS_STATUS.to_string()];
    v.push("X1 EMITTED-SHAPE contract, not a producer/consumer pair: a promise about what this repository emits at a boundary; no consumer evidence is used or claimed".to_string());
    for b in &e.boundaries {
        let how: Vec<String> = b
            .eligibility
            .split(',')
            .map(|x| match x.split_once(':') {
                Some(("versioned-path", seg)) => format!("versioned path segment {seg:?}"),
                Some((_, mark)) => format!("marked under [egress] in invar.toml ({mark:?})"),
                None => x.to_string(),
            })
            .collect();
        v.push(format!("X2 boundary file {} is eligible by {}", b.file, how.join(" and ")));
    }
    v.push(format!(
        "X3 {} declared at {}:{} as a z.object literal with {} direct fields: {}",
        e.shape,
        e.producer,
        e.decl_line,
        names.len(),
        names.join(", ")
    ));
    for b in &e.boundaries {
        let via = match &b.barrel {
            Some((bf, l)) => format!(", re-exported by barrel {bf}:{l}"),
            None => String::new(),
        };
        v.push(format!(
            "X4 {} imported at {}:{} from {:?}, resolved to {}{via}",
            e.shape, b.file, b.import_line, b.specifier, b.resolved
        ));
        let refs: Vec<String> = b.response_refs.iter().map(|l| format!("{}:{l}", b.file)).collect();
        v.push(format!("X5 response-position reference(s): {}", refs.join(", ")));
    }
    v.push(format!(
        "X6 DISCLAIMER: runtime serialization of {} is not verified; a response-position reference does not prove the route emits this schema",
        e.shape
    ));
    v.push(format!("category: {}", e.category));
    v.push(format!(
        "lock: run `invar lock` before `invar check`. Without a lock an egress contract is SKIPPED (exit 4), never HOLDS. The lock binds every direct field of {}; at check a removed or renamed field is VIOLATED and an added field HOLDS.",
        e.shape
    ));
    v
}

fn k2_lines(p: &crate::valuepairs::K2Pair) -> Vec<String> {
    let mut v = vec![K2_STATUS.to_string()];
    for s in &p.shapes {
        v.push(format!("Z1 {} is a z.object literal declared at {}:{} (a value surface, not a type)", s.shape, p.producer, s.decl_line));
    }
    v.push(format!("Z2 consumer {} is a {}: the type system does not constrain these reads", p.consumer, p.consumer_kind));
    for s in &p.shapes {
        let via = match &s.barrel {
            Some((b, l)) => format!(", re-exported by barrel {b}:{l} from {}", p.producer),
            None => String::new(),
        };
        v.push(format!("Z3 {} imported at {}:{} from {:?}, resolved to {}{via}", s.shape, p.consumer, s.import_line, s.specifier, s.resolved));
    }
    for s in &p.shapes {
        let f: Vec<String> = s.fields.iter().map(|(n, l)| format!("{n} ({}:{l})", p.consumer)).collect();
        v.push(format!("Z4 {} fields read: {}", s.shape, f.join(", ")));
        v.push(format!(
            "Z4 {} stoplisted reads: {}",
            s.shape,
            if s.stoplisted.is_empty() { "(none)".to_string() } else { s.stoplisted.join(", ") }
        ));
    }
    v.push(format!("Z5 pair {} -> {}", p.producer, p.consumer));
    v.push(format!("category: {}", p.category));
    v.push("lock: run `invar lock` before `invar check`. The lock binds exactly the Z4 fields above, scoped to their shape. A proposal never claims the consumer is correct.".to_string());
    v
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

    if d.proposals.iter().any(|p| p.kind == "egress") {
        s.push_str("# Slice 2: blocks with kind = \"egress\" are EMITTED-SHAPE contracts: a promise\n# about the direct keys of a zod schema this repository emits at a boundary\n# (versioned path or [egress] marking, plus a response-position reference).\n# They carry no consumer evidence, and runtime serialization is NOT verified.\n\n");
    }
    if d.proposals.iter().any(|p| p.kind == "strings") {
        s.push_str("# Slice 2b: blocks with kind = \"strings\" are STRING-COUPLING contracts: a string\n# list in a query position that names fields of one shape. A locked name that stays\n# in the list after it leaves the shape is a stale reference. They carry no consumer\n# evidence, and that the list reaches the store is NOT verified.\n\n");
    }
    if !d.egress_marks.is_empty() {
        s.push_str("# Egress boundary markings (read from the previous invar.toml; kept so a\n# regenerated manifest does not lose them). Marking makes a file ELIGIBLE for\n# discovery only; it never establishes an emitted contract by itself.\n[egress]\n");
        for m in &d.egress_marks {
            s.push_str(&format!("boundary = \"{m}\"\n"));
        }
        s.push('\n');
    }

    for p in &d.proposals {
        s.push_str(&format!("# PROPOSED {}\n", p.id));
        for e in &p.evidence {
            s.push_str(&format!("# {e}\n"));
        }
        s.push_str("[[invariant]]\n");
        s.push_str(&format!("kind = \"{}\"\n", p.kind));
        s.push_str(&format!("id = \"{}\"\n", p.id));
        s.push_str(&format!("producer = \"{}\"\n", p.producer));
        if let Some(ptr) = &p.pointer {
            s.push_str(&format!("pointer = \"{ptr}\"\n"));
        }
        for (k, v) in &p.keys {
            s.push_str(&format!("{k} = \"{v}\"\n"));
        }
        if p.kind != "egress" && p.kind != "strings" {
            s.push_str(&format!("consumer = \"{}\"\n", p.consumer));
        }
        for e in &p.evidence_keys {
            s.push_str(&format!("evidence = \"{e}\"\n"));
        }
        s.push('\n');
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
    if !d.abstentions.is_empty() {
        // INVAR-GATE-001 R-L-1 and R-L-2 as amended (Option B, ruling of
        // 2026-10-18): one pointer, only when an ABSTAIN line was printed,
        // placed immediately BEFORE the SUMMARY line so the summary stays
        // the machine-readable terminator; `# ` keeps a written file valid.
        s.push_str("# codes: run `invar explain <CODE>` for any ABSTAIN code above\n");
    }
    s.push_str(&format!("# {}\n", summary(d)));
    s
}
