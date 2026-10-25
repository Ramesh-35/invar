//! G3 fixtures (RECALL-ATTACK-1 phase 1, docs/falsification/G3-SPEC.md),
//! frozen with the spec BEFORE any implementation. G3 widens F1's selector
//! grammar with variant values (V5: `TypeIdent::V4` where TypeIdent contains
//! version/format) and default-site occurrences (P6: a default-word line
//! whose two-line window holds a V5 literal). Everything F1 ruled stands:
//! a switch is stated (exit 4), never VIOLATED, and adds nothing when
//! residue exists.
//!
//! On the aed8988 baseline: g3_f01, g3_f02, g3_f08, g3_f12, g3_f13, g3_f14
//! and g3_f15 FAIL (the new behavior); the other ten pass and are the
//! frozen boundary guards. r2 adds f13 to f17 under the founder's A-1 and
//! A-2 rulings of 2026-10-23.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_g3_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

const TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"g3\"\nproducer = \"head.rs\"\nproducer_base = \"base.rs\"\nconsumer = \"c.js\"\n";
const READS: &str = "const v = payload.checksum;\n";

/// An r1-113-shaped producer: the type's variants all exist on both sides
/// (V4 is in max_stable at base too); only the default site's value moves.
fn cargo_like(default_variant: &str) -> String {
    format!(
        "pub enum ResolveVersion {{ V1, V2, V3, V4 }}\n\
         impl ResolveVersion {{\n\
             fn default() -> ResolveVersion {{\n\
                 ResolveVersion::{default_variant}\n\
             }}\n\
             fn max_stable() -> ResolveVersion {{\n\
                 ResolveVersion::V4\n\
             }}\n\
         }}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    )
}

fn stated(out: &str) -> bool {
    out.contains("[not-evaluated:default-switch]")
}

// ------------------------------------------------------- the new behavior

#[test]
fn g3_f01_default_site_variant_switch_is_stated() {
    let d = ws("f01", &[("base.rs", &cargo_like("V3")), ("head.rs", &cargo_like("V4")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "a default-site switch is stated, never HOLDS:\n{out}");
    assert!(stated(&out) && out.contains("[NOT EVALUATED]"), "{out}");
}

#[test]
fn g3_f02_variant_set_gain_is_stated() {
    let base = "pub enum FormatVersion { V1 }\nfn pick(v: FormatVersion) -> FormatVersion { FormatVersion::V1 }\npub struct Out { pub checksum: String }\n";
    let head = "pub enum FormatVersion { V1, V2 }\nfn pick(v: FormatVersion) -> FormatVersion { FormatVersion::V2 }\npub struct Out { pub checksum: String }\n";
    let d = ws("f02", &[("base.rs", base), ("head.rs", head), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "a variant value gained at head is stated:\n{out}");
    assert!(stated(&out), "{out}");
}

#[test]
fn g3_f08_two_types_keyed_separately() {
    let two = |fmt: &str, res: &str| {
        format!(
            "pub enum FormatVersion {{ V1, V2 }}\npub enum ResolveVersion {{ V3, V4 }}\n\
             fn default_format() -> FormatVersion {{\n    FormatVersion::{fmt}\n}}\n\
             fn default_resolve() -> ResolveVersion {{\n    ResolveVersion::{res}\n}}\n\
             pub struct Lock {{ pub checksum: String }}\n"
        )
    };
    let d = ws("f08", &[("base.rs", &two("V1", "V3")), ("head.rs", &two("V1", "V4")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("resolveversion'") && out.contains("3 -> 4"), "the witness names the changed type:\n{out}");
    assert!(!out.contains("formatversion'"), "the unchanged type stays silent:\n{out}");
}

#[test]
fn g3_f12_witness_text_is_exact() {
    let d = ws("f12", &[("base.rs", &cargo_like("V3")), ("head.rs", &cargo_like("V4")), ("c.js", READS), ("invar.toml", TOML)]);
    let (_, out) = invar(&d, &["check"]);
    assert!(
        out.contains("default selector 'default resolveversion' changed 3 -> 4 in head.rs; emission of the fields read in c.js not evaluated [not-evaluated:default-switch]"),
        "{out}"
    );
}

// --------------------------------------------------- frozen boundary guards

#[test]
fn g3_f03_nonnumeric_variant_is_unsupported() {
    let mk = |v: &str| format!("pub enum ResolveVersion {{ Legacy, Latest }}\nfn default() -> ResolveVersion {{\n    ResolveVersion::{v}\n}}\npub struct Lock {{ pub checksum: String }}\n");
    let d = ws("f03", &[("base.rs", &mk("Legacy")), ("head.rs", &mk("Latest")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "a non-numeric variant is outside the grammar, no claim is manufactured:\n{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f04_unchanged_default_site_holds() {
    let head = cargo_like("V3").replace("pub struct Lock", "// a moved comment\npub struct Lock");
    let d = ws("f04", &[("base.rs", &cargo_like("V3")), ("head.rs", &head), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f05_default_words_without_v5_in_window_hold() {
    let mk = |extra: &str| format!("#[derive(Default)]\npub struct Cfg {{ pub default_mode: bool }}\n{extra}pub enum ResolveVersion {{ V3, V4 }}\nfn pick() -> ResolveVersion {{\n    ResolveVersion::V3\n}}\npub struct Lock {{ pub checksum: String }}\n");
    let d = ws("f05", &[("base.rs", &mk("")), ("head.rs", &mk("// default: see docs\n")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "default words with no variant value in the window claim nothing:\n{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f06_commented_variants_do_not_fire() {
    let mk = |v: &str| format!("pub enum ResolveVersion {{ V3, V4 }}\nfn default() -> ResolveVersion {{\n    // was ResolveVersion::{v}\n    pick()\n}}\npub struct Lock {{ pub checksum: String }}\n");
    let d = ws("f06", &[("base.rs", &mk("V3")), ("head.rs", &mk("V4")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f07_a_value_removed_never_fires() {
    let base = "pub enum FormatVersion { V1, V2 }\nfn pick(v: FormatVersion) -> FormatVersion { FormatVersion::V2 }\nfn old(v: FormatVersion) -> FormatVersion { FormatVersion::V1 }\npub struct Out { pub checksum: String }\n";
    let head = "pub enum FormatVersion { V1, V2 }\nfn pick(v: FormatVersion) -> FormatVersion { FormatVersion::V2 }\npub struct Out { pub checksum: String }\n";
    let d = ws("f07", &[("base.rs", base), ("head.rs", head), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "a disappearing value with nothing new never fires (F1 rule, unchanged):\n{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f09_residue_wins_and_g3_adds_nothing() {
    // The selector patterns are textual, so a variant-valued default site in
    // a JS producer exercises G3 and the field extraction together: head
    // drops a read field (residue) AND moves the default site V3 -> V4.
    // F1's adds-nothing rule must hold for G3 occurrences too.
    let mk = |v: &str, digest: &str| format!("const Format = {{ V3: 3, V4: 4 }};\nfunction defaultVersion() {{\n  return FormatVersion::{v};\n}}\nfunction write(pkg) {{\n  return {{ checksum: pkg.checksum{digest} }};\n}}\n");
    let toml = TOML.replace("head.rs", "head.js").replace("base.rs", "base.js");
    let d = ws("f09", &[("base.js", &mk("V3", ", digest: pkg.digest")), ("head.js", &mk("V4", "")), ("c.js", "const v = payload.checksum;\nconst d = payload.digest;\n"), ("invar.toml", &toml)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "real residue is VIOLATED:\n{out}");
    assert!(out.contains("[VIOLATED]") && !stated(&out), "with residue, a switch adds nothing (F1 rule, unchanged by G3):\n{out}");
}

#[test]
fn g3_f10_existing_f1_patterns_unchanged() {
    let mk = |v: &str| format!("const defaultLockfileVersion = {v};\nfunction write(pkg) {{\n  return {{ checksum: pkg.checksum }};\n}}\n");
    let toml = TOML.replace("head.rs", "head.js").replace("base.rs", "base.js");
    let d = ws("f10", &[("base.js", &mk("2")), ("head.js", &mk("3")), ("c.js", READS), ("invar.toml", &toml)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("default selector 'defaultlockfileversion' changed 2 -> 3 in head.js"), "{out}");
}

#[test]
fn g3_f11_strict_mode_is_untouched() {
    // Strict mode (no producer_base) runs no switch analysis at all. On a
    // .rs producer the baseline extracts no spec fields and convicts the
    // read; that behavior, right or wrong, is frozen here byte-for-byte so
    // G3 cannot alter strict mode in passing.
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"g3\"\nproducer = \"head.rs\"\nconsumer = \"c.js\"\n";
    let d = ws("f11", &[("head.rs", &cargo_like("V4")), ("c.js", READS), ("invar.toml", toml)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] g3 (schema): [\"checksum\"]") && out.contains("checksum read at c.js:1"), "{out}");
    assert!(!stated(&out), "{out}");
}

// ------------------------------------------------- r2: A-1 site identity

#[test]
fn g3_f13_two_sites_distinct_identifiers_keyed_apart() {
    // Type-only keying would merge the two sites' values ({3,4} base,
    // {4} head union {4}) and hide the switch; keying by the trigger
    // identifier fires on the changed site only (G3-M11's kill).
    let mk = |d: &str| format!(
        "pub enum ResolveVersion {{ V1, V2, V3, V4 }}\n\
         fn default() -> ResolveVersion {{\n    ResolveVersion::{d}\n}}\n\
         fn default_for_rust_version() -> ResolveVersion {{\n    ResolveVersion::V4\n}}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    );
    let d = ws("f13", &[("base.rs", &mk("V3")), ("head.rs", &mk("V4")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("'default resolveversion' changed 3 -> 4"), "the changed site is named:\n{out}");
    assert!(!out.contains("default_for_rust_version"), "the unchanged site stays silent:\n{out}");
}

#[test]
fn g3_f14_indistinguishable_sites_merge_with_disclosure() {
    let mk = |b: &str| format!(
        "pub enum FormatVersion {{ V1, V2 }}\n\
         impl A {{\n    fn default() -> FormatVersion {{\n        FormatVersion::V1\n    }}\n}}\n\
         impl B {{\n    fn default() -> FormatVersion {{\n        FormatVersion::{b}\n    }}\n}}\n\
         pub struct Out {{ pub checksum: String }}\n"
    );
    let d = ws("f14", &[("base.rs", &mk("V1")), ("head.rs", &mk("V2")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "{out}");
    assert!(stated(&out) && out.contains("'default formatversion' changed 1 -> 2"), "{out}");
    assert!(out.contains("(2 same-named default sites; site attribution not established)"), "the merge is disclosed, never silent:\n{out}");
}

// ------------------------------------------------- r2: A-2 dot-path syntax

#[test]
fn g3_f15_dot_path_default_site_is_stated() {
    let mk = |v: &str| format!(
        "class FormatVersion:\n    V1 = 1\n    V2 = 2\n\n\
         def default_format():\n    return FormatVersion.{v}\n\n\
         def write(pkg):\n    return {{\"checksum\": pkg[\"checksum\"]}}\n"
    );
    let toml = TOML.replace("head.rs", "head.py").replace("base.rs", "base.py");
    let d = ws("f15", &[("base.py", &mk("V1")), ("head.py", &mk("V2")), ("c.js", READS), ("invar.toml", &toml)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "a dot-path variant at a default site is stated:\n{out}");
    assert!(out.contains("'default_format formatversion' changed 1 -> 2"), "{out}");
}

#[test]
fn g3_f16_dot_path_lexical_negatives_hold() {
    let mk = |m: &str, v: &str| format!(
        "class FormatVersion:\n    V1 = 1\n\n\
         def default_format():\n    return FormatVersion.{m}(formatVersion.{v})\n\n\
         def write(pkg):\n    return {{\"checksum\": pkg[\"checksum\"]}}\n"
    );
    let toml = TOML.replace("head.rs", "head.py").replace("base.rs", "base.py");
    let d = ws("f16", &[("base.py", &mk("parse", "v2x")), ("head.py", &mk("encode", "v3y")), ("c.js", READS), ("invar.toml", &toml)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "non-variant members and suffixed identifiers are not values:\n{out}");
    assert!(!stated(&out), "{out}");
}

// ----------------------------------- r2: window closed by a completed statement

#[test]
fn g3_f17_a_closed_declaration_lends_no_value() {
    // The default-word line ends its statement with `;`, so the variant
    // change two lines below belongs to another declaration and must not be
    // associated. Every variant value exists at base (V4 in max()), so P5's
    // set rule stays silent too: any firing here is a manufactured site.
    let mk = |v: &str| format!(
        "pub enum ResolveVersion {{ V3, V4 }}\n\
         const DEFAULT_TIMEOUT: u32 = 5;\n\
         fn pick() -> ResolveVersion {{\n    ResolveVersion::{v}\n}}\n\
         fn max() -> ResolveVersion {{\n    ResolveVersion::V4\n}}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    );
    let d = ws("f17", &[("base.rs", &mk("V3")), ("head.rs", &mk("V4")), ("c.js", READS), ("invar.toml", TOML)]);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(!stated(&out), "{out}");
}

