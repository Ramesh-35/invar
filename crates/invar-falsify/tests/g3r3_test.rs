//! G3 r3 repair fixtures (docs/falsification/G3-SPEC-r3.md), added under the
//! founder's ruling of 2026-10-23 after the r2 mutation run: M4 survived as
//! a non-equivalent false-claim path, M3 and M6 were killed without their
//! declared properties being isolated, and M8 was equivalent. These
//! fixtures are ADDED; tests/g3_test.rs (r2) is unchanged byte-for-byte.
//!
//! Each fixture isolates exactly one property and names the operator it
//! exists to kill. Every "must hold" fixture is built so the P5 set rule is
//! silent (each value present at base), so only the property under test
//! could produce a claim.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_g3r3_{tag}_{}_{nanos}", std::process::id()));
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

fn stated(out: &str) -> bool {
    out.contains("[not-evaluated:default-switch]")
}

fn run(tag: &str, base: &str, head: &str) -> (i32, String) {
    let d = ws(tag, &[("base.rs", base), ("head.rs", head), ("c.js", READS), ("invar.toml", TOML)]);
    invar(&d, &["check"])
}

// ---------------------------------------------------------- M4: selector type

#[test]
fn g3_f18_non_selector_type_at_a_default_site_holds() {
    // `Mode` contains neither `version` nor `format`, so under the frozen V5
    // grammar `Mode::V2` is NOT a selector value, wherever it appears. The
    // default site changes V1 -> V2 and the verdict must stay HOLDS with no
    // invented selector claim. (Kills M4: with the type requirement dropped,
    // `default mode` changed 1 -> 2 is manufactured.)
    let mk = |v: &str| format!("pub enum Mode {{ V1, V2 }}\nfn default() -> Mode {{\n    Mode::{v}\n}}\npub struct Lock {{ pub checksum: String }}\n");
    let (code, out) = run("f18", &mk("V1"), &mk("V2"));
    assert_eq!(code, 0, "a non-selector type is never a selector:\n{out}");
    assert!(!stated(&out), "{out}");
    assert!(!out.contains("'default mode'") && !out.contains("'mode'"), "no claim names the non-selector type:\n{out}");
}

// ---------------------------------------------------- M3: the default word

#[test]
fn g3_f19_a_line_without_default_creates_no_site() {
    // One changed value inside a NON-default function, on a line whose first
    // identifier (`pub`) is unique to it, so no two sites can merge: if any
    // trigger without the `default` word were accepted, this site alone
    // would fire. P5 is silent: V4 exists at base in `top`. (Kills M3 by
    // the declared property, independent of site merging.)
    let mk = |v: &str| format!(
        "pub enum ResolveVersion {{ V3, V4 }}\n\
         pub fn pick() -> ResolveVersion {{ ResolveVersion::{v} }}\n\
         fn top() -> ResolveVersion {{ ResolveVersion::V4 }}\n\
         struct Lock {{ checksum: String }}\n"
    );
    let (code, out) = run("f19", &mk("V3"), &mk("V4"));
    assert_eq!(code, 0, "only a default identifier opens a site:\n{out}");
    assert!(!stated(&out), "{out}");
}

// ------------------------------------------------ M6: type in the site key

#[test]
fn g3_f20_two_types_under_one_trigger_name_are_keyed_apart() {
    // Two `default` sites with the SAME trigger name for two DIFFERENT
    // types. The FormatVersion site holds 4 on both sides; the
    // ResolveVersion site moves 3 -> 4. Keyed by trigger and type, the
    // switch fires. Keyed by trigger alone (M6), the sets merge to
    // {3,4} -> {4} and the switch is HIDDEN: a verdict difference (exit 4
    // vs 0), so no P5 witness can mask the mutation: P5 is silent here
    // (ResolveVersion::V4 exists at base in `max`). No disclosure is due:
    // each trigger-and-type key has exactly one site.
    let mk = |v: &str| format!(
        "pub enum FormatVersion {{ V3, V4 }}\n\
         pub enum ResolveVersion {{ V3, V4 }}\n\
         impl A {{\n    fn default() -> FormatVersion {{\n        FormatVersion::V4\n    }}\n}}\n\
         impl B {{\n    fn default() -> ResolveVersion {{\n        ResolveVersion::{v}\n    }}\n}}\n\
         fn max() -> ResolveVersion {{ ResolveVersion::V4 }}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    );
    let (code, out) = run("f20", &mk("V3"), &mk("V4"));
    assert_eq!(code, 4, "the ResolveVersion default site switched:\n{out}");
    assert!(out.contains("'default resolveversion' changed 3 -> 4"), "{out}");
    assert!(!out.contains("'default formatversion'"), "the FormatVersion site is unchanged:\n{out}");
    assert!(!out.contains("same-named default sites"), "distinct types are distinguishable sites:\n{out}");
}

// ------------------------------- comment guards (replacing the equivalent M8)

#[test]
fn g3_f21_guard_a_commented_selector_lines_never_fire() {
    // Guard A: the per-line comment skip in default_selectors (shared with
    // F1). The selector assignment changes ONLY inside a comment; the live
    // assignment is unchanged. (Kills M8a. G3's own variant scanning has a
    // second, redundant guard, so guard A is observable through F1's
    // assignment pattern, the pattern it also protects.)
    let mk = |v: &str| format!("const defaultLockfileVersion = 2;\n// const defaultLockfileVersion = {v};\npub struct Lock {{ pub checksum: String }}\n");
    let (code, out) = run("f21", &mk("2"), &mk("3"));
    assert_eq!(code, 0, "a commented selector is not a selector:\n{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f22_guard_b_a_comment_is_never_a_trigger() {
    // Guard B: default_sites skips comment lines as triggers. The only
    // `default` word is in a comment ending with `:`, a continuation token,
    // so without the guard the comment would open a window over the next
    // line's changed value. P5 is silent (V4 at base). (Kills M8b.)
    let mk = |v: &str| format!(
        "pub enum ResolveVersion {{ V3, V4 }}\n\
         // default:\n\
         fn pick() -> ResolveVersion {{ ResolveVersion::{v} }}\n\
         fn max() -> ResolveVersion {{ ResolveVersion::V4 }}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    );
    let (code, out) = run("f22", &mk("V3"), &mk("V4"));
    assert_eq!(code, 0, "a comment opens no site:\n{out}");
    assert!(!stated(&out), "{out}");
}

#[test]
fn g3_f23_guard_c_a_comment_inside_the_window_is_transparent() {
    // Guard C: the P6 window steps over comment lines. A comment between
    // the default signature and its value must neither count toward the
    // two-line window nor end it, so the real switch is still stated. This
    // guard's regression cannot make a comment's content count (V5 scanning
    // rejects comment lines independently); its observable effect is that a
    // comment would wrongly END the window and hide a real switch. The
    // fixture asserts that direction, stated as the guard's only behavior.
    // P5 is silent (V4 at base). (Kills M8c.)
    let mk = |v: &str| format!(
        "pub enum ResolveVersion {{ V3, V4 }}\n\
         fn default() -> ResolveVersion {{\n    // the default lockfile version\n    ResolveVersion::{v}\n}}\n\
         fn max() -> ResolveVersion {{ ResolveVersion::V4 }}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    );
    let (code, out) = run("f23", &mk("V3"), &mk("V4"));
    assert_eq!(code, 4, "the comment is transparent; the site switched:\n{out}");
    assert!(out.contains("'default resolveversion' changed 3 -> 4"), "{out}");
}

// ------------------------------------------------- blank line closes window

#[test]
fn g3_f24_a_blank_line_closes_the_window() {
    // r3 clarification, made explicit: a blank line is a non-comment line
    // that does not end with a continuation token, so it closes the P6
    // window. The value two lines below a blank is not associated. P5 is
    // silent (V4 at base). (Kills M15: blank lines skipped like comments.)
    let mk = |v: &str| format!(
        "pub enum ResolveVersion {{ V3, V4 }}\n\
         fn default() -> ResolveVersion {{\n\n    ResolveVersion::{v}\n}}\n\
         fn max() -> ResolveVersion {{ ResolveVersion::V4 }}\n\
         pub struct Lock {{ pub checksum: String }}\n"
    );
    let (code, out) = run("f24", &mk("V3"), &mk("V4"));
    assert_eq!(code, 0, "no association across a blank line:\n{out}");
    assert!(!stated(&out), "{out}");
}
