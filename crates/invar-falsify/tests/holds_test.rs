//! INVAR-HOLDS-001 fixtures (docs/architecture/INVAR-HOLDS-001.md, rulings
//! H-D1 to H-D6 of 2026-10-18). GATE 1: written BEFORE the durability line
//! exists. Declared outcomes against the unchanged engine: H-F1, H-F2, H-F3,
//! H-F6, H-F7 and H-F9 fail (no durability line exists); H-F4, H-F5, H-N1,
//! H-N2 and H-M1 pass as invariance controls and stay.
//!
//! Interpretations carried to the Gate 5 ledger:
//! - H-INT-1: spec A3's unparsable-since arm is UNREACHABLE by construction:
//!   lock::parse_bind bails on any bind line whose t_v date does not parse,
//!   so a lock that parses carries only parseable since dates. The stronger
//!   guard is the lock grammar itself; mutation 10 of the spec is therefore
//!   replaced by the singular/plural rule mutant, killed by H-F9.
//! - H-INT-2: a future-dated since (clock skew) would yield a negative age;
//!   the line is omitted rather than claiming a negative duration.

use std::path::{Path, PathBuf};
use std::process::Command;

use invar_falsify::lock::{format_date, parse_date, today_utc};

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_hld_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (rel, body) in files {
        put(&dir, rel, body);
    }
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(bin()).args(no_reuse(args)).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

const ORDER_IN: &str = "import { z } from \"zod\";\nexport const OrderIn = z.object({\n  total_minor: z.number(),\n  currency_code: z.string(),\n  unread_flag: z.boolean(),\n});\n";

const CONSUME: &str = "import { OrderIn } from \"../lib/order.js\";\nexport function f(row) {\n  return [row.total_minor, row.currency_code];\n}\n";

const SCHEMA_TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"code:order\"\nproducer = \"lib/order.ts\"\nconsumer = \"src/consume.js\"\nevidence = \"OrderIn currency_code total_minor\"\n";

/// A locked schema-contract repo whose check HOLDS (exit 0).
fn held(tag: &str) -> PathBuf {
    let d = ws(tag, &[("lib/order.ts", ORDER_IN), ("src/consume.js", CONSUME), ("invar.toml", SCHEMA_TOML)]);
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "pre-condition: {out}");
    d
}

fn check(d: &Path) -> (i32, String) {
    let a = invar(d, &["check"]);
    let b = invar(d, &["check"]);
    assert_eq!(a, b, "check is not deterministic within a run pair");
    a
}

/// Rewrite every since (t_v) date in the lock's bind lines from `from` to
/// `to`. The space-delimited pattern never touches recorded_at (t_r), whose
/// date is followed by `T`, so CM-H-4 stays bound in: the line must follow
/// since, and a mutant reading recorded_at prints a different date.
fn redate(d: &Path, from: &str, to: &str) {
    let p = d.join("invar.lock");
    let text = std::fs::read_to_string(&p).unwrap().replace(&format!(" {from} "), &format!(" {to} "));
    std::fs::write(p, text).unwrap();
}

fn durability_line(date: &str, days: i64) -> String {
    format!("  durability: held since {date} ({days} day{})", if days == 1 { "" } else { "s" })
}

// ------------------------------------------------------------- fixtures

#[test]
fn h_f1_fresh_lock_holds_since_today_zero_days() {
    let d = held("hf1");
    let (c, out) = check(&d);
    assert_eq!(c, 0, "{out}");
    let today = format_date(today_utc());
    assert!(out.contains(&durability_line(&today, 0)), "exact frozen format (R-H-1): {out}");
    assert_eq!(out.matches("durability:").count(), 1, "one line per held contract: {out}");
}

#[test]
fn h_f2_edited_since_dates_drive_the_line_not_recorded_at() {
    let d = held("hf2");
    let today = format_date(today_utc());
    redate(&d, &today, "2026-01-01");
    // recorded_at stays today inside the lock, so CM-H-4 is bound in: the
    // line must follow since (t_v), never recorded_at (t_r).
    let (c, out) = check(&d);
    assert_eq!(c, 0, "{out}");
    let days = today_utc() - parse_date("2026-01-01").unwrap();
    assert!(days > 100, "sanity: {days}");
    assert!(out.contains(&durability_line("2026-01-01", days)), "{out}");
}

#[test]
fn h_f3_a_reaccepted_binding_resets_durability_to_today() {
    // A consistent rename of a BOUND field on both sides: the new binding is
    // accepted fresh today, so max(since) returns to today. (Gate 1 taught
    // the converse too: editing an UNBOUND field preserves the age, because
    // nothing was re-accepted; the engine was right and the first version of
    // this fixture was wrong, recorded in the ledger.)
    let d = held("hf3");
    let today = format_date(today_utc());
    redate(&d, &today, "2026-01-01");
    put(&d, "lib/order.ts", &ORDER_IN.replace("currency_code", "currency_kode"));
    put(&d, "src/consume.js", &CONSUME.replace("currency_code", "currency_kode"));
    // The lock is evidence-bounded (slice 1 L01), so the human re-declares
    // the renamed field in the manifest; only then is it re-accepted.
    put(&d, "invar.toml", &SCHEMA_TOML.replace("currency_code", "currency_kode"));
    let (c, out) = invar(&d, &["lock", "--update"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = check(&d);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains(&format!("held since {today}")), "re-acceptance resets the clock: {out}");
    assert!(!out.contains("held since 2026-01-01"), "{out}");
}

#[test]
fn h_f4_a_violated_contract_carries_no_age() {
    let d = held("hf4");
    put(&d, "lib/order.ts", &ORDER_IN.replace("currency_code", "currency_kode"));
    let (c, out) = check(&d);
    assert_eq!(c, 1, "{out}");
    assert!(!out.contains("durability:"), "a broken thing with a proud age is worse than no line: {out}");
}

#[test]
fn h_f5_a_skipped_contract_carries_no_age() {
    let d = ws("hf5", &[("lib/order.ts", ORDER_IN), ("src/consume.js", CONSUME), ("invar.toml", SCHEMA_TOML)]);
    // No lock: the contract is SKIPPED (exit 4), and a skip never has an age.
    let (c, out) = check(&d);
    assert_eq!(c, 4, "{out}");
    assert!(!out.contains("durability:"), "{out}");
    // A LOCKED contract that becomes unevaluable (a strings list turned
    // dynamic after lock, the k3_u2 pattern): the lock entry exists with
    // since dates, the outcome is not evaluated, and still no age may print.
    // Without this half, an age-on-skipped mutant is unreachable (Gate 3
    // caught that as a surviving mutant, recorded in the ledger).
    let token = "import { z } from \"zod\";\nexport const ApiToken = z.object({\n  accessKey: z.string(),\n  lastUsed: z.string(),\n  permissions: z.array(z.string()),\n  expiresAt: z.string(),\n});\n";
    let service = "const SELECT_FIELDS = ['id', 'lastUsed', 'accessKey', 'permissions', 'expiresAt'];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n";
    let toml = "[[invariant]]\nkind = \"strings\"\nid = \"strings:ApiToken@const:SELECT_FIELDS\"\nproducer = \"src/token.ts\"\nshape = \"ApiToken\"\nlist = \"src/service.ts\"\nlistat = \"const:SELECT_FIELDS\"\nscope = \"same-directory\"\noverlap = \"accessKey expiresAt lastUsed permissions\"\n";
    let d2 = ws("hf5b", &[("src/token.ts", token), ("src/service.ts", service), ("invar.toml", toml)]);
    assert_eq!(invar(&d2, &["lock"]).0, 0);
    put(&d2, "src/service.ts", &service.replace("['id',", "[...extra, 'id',"));
    let (c, out) = check(&d2);
    assert_eq!(c, 4, "{out}");
    assert!(!out.contains("durability:"), "a lock entry exists, the outcome is skipped, and no age may print: {out}");
}

#[test]
fn h_f6_the_youngest_acceptance_wins() {
    let d = held("hf6");
    let today = format_date(today_utc());
    // CM-H-1: exactly one BIND line aged, the other left at today; max(since)
    // must win. (A naive first-occurrence replace would age recorded_at,
    // which precedes the bind lines; Gate 3 caught that as a surviving
    // mutant, recorded in the ledger.)
    let p = d.join("invar.lock");
    let text = std::fs::read_to_string(&p).unwrap();
    let mut aged_one = false;
    let aged: String = text
        .lines()
        .map(|l| {
            if !aged_one && l.starts_with("bind = ") && l.contains(&today) {
                aged_one = true;
                format!("{}\n", l.replace(&today, "2026-01-01"))
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    assert!(aged_one, "one bind line must have been aged:\n{text}");
    assert!(aged.lines().any(|l| l.starts_with("bind = ") && l.contains(&today)), "one bind line must remain at today");
    std::fs::write(&p, aged).unwrap();
    let (c, out) = check(&d);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains(&durability_line(&today, 0)), "max(since), never min: {out}");
    assert!(!out.contains("held since 2026-01-01"), "{out}");
}

#[test]
fn h_f7_every_held_kind_gets_its_own_line() {
    // One repo, three kinds: schema, egress, strings; all locked and held.
    let toml = format!(
        "{SCHEMA_TOML}\n[[invariant]]\nkind = \"egress\"\nid = \"egress:OrderResponse\"\nproducer = \"lib/schemas.ts\"\nshape = \"OrderResponse\"\nboundary = \"app/api/v1/orders/route.ts\"\neligibility = \"versioned-path\"\nanchor = \"GET /v1/orders\"\n\n[[invariant]]\nkind = \"strings\"\nid = \"strings:ApiToken@const:SELECT_FIELDS\"\nproducer = \"src/token.ts\"\nshape = \"ApiToken\"\nlist = \"src/service.ts\"\nlistat = \"const:SELECT_FIELDS\"\nscope = \"same-directory\"\noverlap = \"accessKey expiresAt lastUsed permissions\"\n"
    );
    let d = ws(
        "hf7",
        &[
            ("lib/order.ts", ORDER_IN),
            ("src/consume.js", CONSUME),
            ("lib/schemas.ts", "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_code: z.string(),\n});\n"),
            ("src/token.ts", "import { z } from \"zod\";\nexport const ApiToken = z.object({\n  accessKey: z.string(),\n  lastUsed: z.string(),\n  permissions: z.array(z.string()),\n  expiresAt: z.string(),\n});\n"),
            ("src/service.ts", "const SELECT_FIELDS = ['id', 'lastUsed', 'accessKey', 'permissions', 'expiresAt'];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n"),
            ("invar.toml", &toml),
        ],
    );
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = check(&d);
    assert_eq!(c, 0, "{out}");
    assert_eq!(out.matches("[HOLDS]").count(), 3, "{out}");
    assert_eq!(out.matches("durability: held since").count(), 3, "one line per held contract, every kind: {out}");
}

#[test]
fn h_f9_one_day_prints_singular() {
    let d = held("hf9");
    let today = format_date(today_utc());
    let yesterday = format_date(today_utc() - 1);
    redate(&d, &today, &yesterday);
    let (c, out) = check(&d);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains(&durability_line(&yesterday, 1)), "exactly `1 day`, never `1 days`: {out}");
    assert!(!out.contains("1 days"), "{out}");
}

#[test]
fn h_n1_machine_surfaces_stay_silent() {
    let d = held("hn1");
    let (c, out) = invar(&d, &["check", "--receipt", "r.txt", "--sarif", "s.json", "--report-md", "m.md"]);
    assert_eq!(c, 0, "{out}");
    for f in ["r.txt", "s.json", "m.md"] {
        let text = std::fs::read_to_string(d.join(f)).unwrap();
        assert!(!text.contains("durability") && !text.contains("held since"), "{f} must stay byte-stable (R-H-4): {text}");
    }
}

#[test]
fn h_n2_proof_imports_keep_their_own_time_axis() {
    // A held proof import under check --proof: epochs, never durability.
    let d = ws(
        "hn2p",
        &[
            ("lib/schemas.ts", "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_code: z.string(),\n});\n"),
            ("invar.toml", "[[invariant]]\nkind = \"egress\"\nid = \"egress:OrderResponse\"\nproducer = \"lib/schemas.ts\"\nshape = \"OrderResponse\"\nboundary = \"app/api/v1/orders/route.ts\"\neligibility = \"versioned-path\"\nanchor = \"GET /v1/orders\"\n"),
        ],
    );
    let git = |args: &[&str]| {
        let o = Command::new("git")
            .args(["-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false"])
            .args(args)
            .current_dir(&d)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert_eq!(invar(&d, &["lock", "--emit-proof", "egress:OrderResponse"]).0, 0);
    let p = std::fs::read_to_string(d.join("invar.proof")).unwrap();
    let dg = p.lines().find_map(|l| l.strip_prefix("digest = \"")).unwrap().trim_end_matches('"');
    let cons = ws(
        "hn2c",
        &[
            ("vendor/orders.proof", &p),
            ("invar.toml", &format!("[[proof.import]]\nproof = \"vendor/orders.proof\"\ncontract_id = \"egress:OrderResponse\"\nanchor = \"GET /v1/orders\"\nfields = \"id total_minor\"\naccepted_epoch = \"1\"\naccepted_digest = \"{dg}\"\n")),
        ],
    );
    let (c, out) = invar(&cons, &["check", "--proof"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS] proof"), "{out}");
    assert!(!out.contains("durability:"), "two time semantics are never blended (B3): {out}");
}

#[test]
fn h_m1_byte_identical_within_a_day() {
    let d = held("hm1");
    let a = invar(&d, &["check"]);
    let b = invar(&d, &["check"]);
    assert_eq!(a, b);
}

/// CHECK-INCR-1 (K-2): these tests exercise evaluation, so `check` runs with
/// `--no-reuse`; reuse is tested in check_incr1_test.rs.
fn no_reuse<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut v = args.to_vec();
    if v.first() == Some(&"check") && !v.contains(&"--no-reuse") {
        v.push("--no-reuse");
    }
    v
}
