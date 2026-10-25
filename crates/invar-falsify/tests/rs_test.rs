//! REQUEST-SIDE fixtures (RECALL-ATTACK-1 phase 2,
//! docs/falsification/REQUEST-SIDE-SPEC.md r1), frozen with the spec
//! BEFORE any implementation. Change-scoped workspaces (a producer_base is
//! declared), as the rules apply only there.
//!
//! RS-1 convicts (VIOLATED) when a base producer source reads K through the
//! frozen B2 grammar, no head producer source reads K or even holds K as a
//! string literal, and the consumer writes K through B1. RS-2 states (exit
//! 4) a serializer field that became read_only. RS-0 states (exit 4) a
//! removed inbound read when the consumer's payload is opaque. RS-0's
//! fixtures are SYNTHETIC: no corpus case exercises it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_rs_{tag}_{}_{nanos}", std::process::id()));
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

fn toml(producer: &str, base: &str, consumer: &str) -> String {
    format!("[[invariant]]\nkind = \"schema\"\nid = \"rs\"\nproducer = \"{producer}\"\nproducer_base = \"{base}\"\nconsumer = \"{consumer}\"\n")
}

fn check(tag: &str, files: &[(&str, &str)]) -> (i32, String) {
    let d = ws(tag, files);
    invar(&d, &["check", "--no-reuse"])
}

/// The r1-118 shape: a builder-style form reader of "pdfFormat" at base.
/// The read is on line 6.
const GO_BASE: &str = "package chromium\n\nfunc FormDataChromiumPdfFormats(ctx *api.Context) gotenberg.PdfFormats {\n\tvar pdfFormat string\n\tctx.FormData().\n\t\tString(\"pdfFormat\", &pdfFormat, \"\").\n\t\tBool(\"pdfua\", &pdfua, false)\n\treturn gotenberg.PdfFormats{}\n}\n";
/// Head: the read is gone; only identifiers containing the name remain.
const GO_HEAD: &str = "package chromium\n\nfunc FormDataChromiumPdfFormats(ctx *api.Context) gotenberg.PdfFormats {\n\tctx.FormData().\n\t\tBool(\"pdfua\", &pdfua, false)\n\tpdfFormats := gotenberg.PdfFormats{}\n\treturn pdfFormats\n}\n";
/// The consumer writes pdfFormat on line 3 (subscript assignment).
const PY_WRITES: &str = "data = {}\nif mode == \"a2b\":\n    data[\"pdfFormat\"] = \"PDF/A-2b\"\ndata[\"pdfua\"] = \"true\"\n";

fn rs1_files(head: &str, base: &str, consumer: &str) -> Vec<(&'static str, String)> {
    vec![
        ("head.go", head.to_string()),
        ("base.go", base.to_string()),
        ("c.py", consumer.to_string()),
        ("invar.toml", toml("head.go", "base.go", "c.py")),
    ]
}

fn run(tag: &str, files: Vec<(&'static str, String)>) -> (i32, String) {
    let refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (*a, b.as_str())).collect();
    check(tag, &refs)
}

fn no_rs(out: &str) -> bool {
    !out.contains("no inbound read and no literal")
        && !out.contains("[not-evaluated:opaque-payload]")
        && !out.contains("[not-evaluated:read-only-field]")
}

// ------------------------------------------------------------------- RS-1

#[test]
fn rs_f01_removed_inbound_read_with_a_consumer_write_convicts() {
    let (code, out) = run("f01", rs1_files(GO_HEAD, GO_BASE, PY_WRITES));
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] rs (schema): [\"pdfFormat\"]"), "{out}");
    assert!(
        out.contains("pdfFormat · not in head.go at head: no inbound read and no literal of pdfFormat remain (inbound read at base.go:6 at base); written at c.py:3"),
        "the witness names the consumer write line, the base read line and the absence at head:\n{out}"
    );
    assert!(!out.contains("pdfua"), "pdfua is still read at head; no claim about it:\n{out}");
}

#[test]
fn rs_f02_a_surviving_inbound_accessor_prevents_conviction() {
    let head = GO_HEAD.replace("\treturn pdfFormats\n", "\t_ = r.FormValue(\"pdfFormat\")\n\treturn pdfFormats\n");
    let (code, out) = run("f02", rs1_files(&head, GO_BASE, PY_WRITES));
    assert_eq!(code, 0, "the key is still read at head:\n{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f03_a_consumer_read_is_not_a_write() {
    let consumer = "resp = client.get(url)\nx = resp[\"pdfFormat\"]\nprint(x)\n";
    let (code, out) = run("f03", rs1_files(GO_HEAD, GO_BASE, consumer));
    assert_eq!(code, 0, "a read of the key is not a B1 write:\n{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f04_a_commented_out_base_read_is_no_binding() {
    let base = GO_BASE.replace("\t\tString(\"pdfFormat\", &pdfFormat, \"\").\n", "\t\t// String(\"pdfFormat\", &pdfFormat, \"\").\n");
    let (code, out) = run("f04", rs1_files(GO_HEAD, &base, PY_WRITES));
    assert_eq!(code, 0, "{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f05_a_commented_out_consumer_write_is_no_write() {
    let consumer = "data = {}\n# data[\"pdfFormat\"] = \"PDF/A-2b\"\ndata[\"pdfua\"] = \"true\"\n";
    let (code, out) = run("f05", rs1_files(GO_HEAD, GO_BASE, consumer));
    assert_eq!(code, 0, "{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f09a_an_unrelated_base_literal_is_no_inbound_binding() {
    // "pdfFormat" appears at base only inside a log call, which B2 does not
    // read; there is no inbound binding to have been removed.
    let base = GO_BASE.replace("\t\tString(\"pdfFormat\", &pdfFormat, \"\").\n", "").replace("\tvar pdfFormat string\n", "\tlog.Info(\"pdfFormat\")\n");
    let (code, out) = run("f09a", rs1_files(GO_HEAD, &base, PY_WRITES));
    assert_eq!(code, 0, "{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f09b_a_surviving_head_literal_blocks_conviction() {
    // The B2 read is gone, but "pdfFormat" survives at head in a form B2
    // does not read: an extraction boundary, never proof of absence, so
    // RS-1 makes no claim at all.
    let head = GO_HEAD.replace("\treturn pdfFormats\n", "\tdeprecated := []string{\"pdfFormat\"}\n\t_ = deprecated\n\treturn pdfFormats\n");
    let (code, out) = run("f09b", rs1_files(&head, GO_BASE, PY_WRITES));
    assert_eq!(code, 0, "{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f10_no_pairing_outside_the_contract() {
    // Contract a: the producer drops the read, but its consumer never writes
    // the key. Contract b: its consumer writes the key, but its producer
    // never read it. Neither contract is paired with the other's files.
    let other = "package other\n\nfunc Handle() {\n\tctx.FormData().\n\t\tBool(\"pdfua\", &pdfua, false)\n}\n";
    let t = format!(
        "{}\n{}",
        toml("head.go", "base.go", "a.py").replace("id = \"rs\"", "id = \"a\""),
        toml("other.go", "other_base.go", "b.py").replace("id = \"rs\"", "id = \"b\"")
    );
    let files = vec![
        ("head.go", GO_HEAD.to_string()),
        ("base.go", GO_BASE.to_string()),
        ("other.go", other.to_string()),
        ("other_base.go", other.to_string()),
        ("a.py", "data = {}\ndata[\"pdfua\"] = \"true\"\n".to_string()),
        ("b.py", PY_WRITES.to_string()),
        ("invar.toml", t),
    ];
    let (code, out) = run("f10", files);
    assert_eq!(code, 0, "no conviction without the contract binding both sides:\n{out}");
    assert!(no_rs(&out), "{out}");
}

#[test]
fn rs_f15_express_member_read_and_object_literal_write() {
    let base = "app.post(\"/convert\", (req, res) => {\n  const fmt = req.body.pdfFormat;\n  const ua = req.body.pdfua;\n  res.json({ ok: true });\n});\n";
    let head = "app.post(\"/convert\", (req, res) => {\n  const ua = req.body.pdfua;\n  res.json({ ok: true });\n});\n";
    let consumer = "await fetch(url, {\n  method: \"POST\",\n  body: JSON.stringify({ \"pdfFormat\": \"PDF/A-2b\", \"pdfua\": true }),\n});\n";
    let files = vec![
        ("head.js", head.to_string()),
        ("base.js", base.to_string()),
        ("c.ts", consumer.to_string()),
        ("invar.toml", toml("head.js", "base.js", "c.ts")),
    ];
    let (code, out) = run("f15", files);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("inbound read at base.js:2 at base); written at c.ts:3"), "{out}");
}

#[test]
fn rs_f16_builder_setter_write_convicts() {
    let consumer = "package client\n\nfunc send(w *multipart.Writer) {\n\tw.WriteField(\"pdfFormat\", \"PDF/A-2b\")\n\tw.WriteField(\"pdfua\", \"true\")\n}\n";
    let files = vec![
        ("head.go", GO_HEAD.to_string()),
        ("base.go", GO_BASE.to_string()),
        ("c.go", consumer.to_string()),
        ("invar.toml", toml("head.go", "base.go", "c.go")),
    ];
    let (code, out) = run("f16", files);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("written at c.go:4"), "{out}");
}

#[test]
fn rs_f17_a_read_moved_to_the_code_companion_is_still_a_read() {
    let comp_head = "package chromium\n\nfunc formats(r *http.Request) string {\n\treturn r.FormValue(\"pdfFormat\")\n}\n";
    let comp_base = "package chromium\n\nfunc formats(r *http.Request) string {\n\treturn \"\"\n}\n";
    let t = toml("head.go", "base.go", "c.py") + "producer_code = \"comp.go\"\nproducer_code_base = \"comp_base.go\"\n";
    let files = vec![
        ("head.go", GO_HEAD.to_string()),
        ("base.go", GO_BASE.to_string()),
        ("comp.go", comp_head.to_string()),
        ("comp_base.go", comp_base.to_string()),
        ("c.py", PY_WRITES.to_string()),
        ("invar.toml", t),
    ];
    let (code, out) = run("f17", files);
    assert!(!out.contains("no inbound read and no literal"), "the companion still reads the key at head:\n{out}");
    assert_ne!(code, 1, "{out}");
}

#[test]
fn rs_f20_rs1_adds_to_existing_residue_without_duplicates() {
    // Response side: base emits `digest` in a Go struct the consumer reads;
    // head drops it (existing rule, residue digest). Request side: RS-1.
    let base = GO_BASE.replace("package chromium\n", "package chromium\n\ntype Out struct {\n\tDigest string `json:\"digest\"`\n}\n");
    let head = GO_HEAD.replace("package chromium\n", "package chromium\n\ntype Out struct {\n}\n");
    let consumer = format!("{PY_WRITES}d = resp[\"digest\"]\n");
    let (code, out) = run("f20", rs1_files(&head, &base, &consumer));
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("\"digest\"") && out.contains("\"pdfFormat\""), "both convictions stand:\n{out}");
    assert_eq!(out.matches("no inbound read and no literal of pdfFormat").count(), 1, "{out}");
}

// ------------------------------------------------------------------- RS-2

/// DRF producer: device_role writable at base, read_only at head (line 4).
const DRF_BASE: &str = "class DeviceSerializer(NetBoxModelSerializer):\n    url = serializers.HyperlinkedIdentityField(view_name='dcim-api:device-detail')\n    device_type = DeviceTypeNestedSerializer()\n    device_role = NestedDeviceRoleSerializer()\n";
const DRF_HEAD: &str = "class DeviceSerializer(NetBoxModelSerializer):\n    url = serializers.HyperlinkedIdentityField(view_name='dcim-api:device-detail')\n    role = NestedDeviceRoleSerializer()\n    device_role = NestedDeviceRoleSerializer(read_only=True, help_text='Deprecated in v3.6 in favor of `role`.')\n";
/// Ansible consumer: mentions device_role on line 3 as a module parameter.
const ANSIBLE: &str = "argument_spec = dict(\n    name=dict(required=True, type=\"raw\"),\n    device_role=dict(required=False, type=\"raw\"),\n)\n";

fn drf(head: &str, base: &str, consumer: &str) -> Vec<(&'static str, String)> {
    vec![
        ("head.py", head.to_string()),
        ("base.py", base.to_string()),
        ("c.py", consumer.to_string()),
        ("invar.toml", toml("head.py", "base.py", "c.py")),
    ]
}

#[test]
fn rs_f12_became_read_only_is_stated_never_convicted() {
    let (code, out) = run("f12", drf(DRF_HEAD, DRF_BASE, ANSIBLE));
    assert_eq!(code, 4, "{out}");
    assert!(!out.contains("[VIOLATED]"), "a consumer mention alone never convicts:\n{out}");
    assert!(
        out.contains("'device_role' became read_only in head.py (head.py:4); consumer mentions it at c.py:3; whether it writes it is not evaluated [not-evaluated:read-only-field]"),
        "{out}"
    );
}

#[test]
fn rs_f13_a_field_new_at_head_and_read_only_gained_nothing() {
    let base = "class DeviceSerializer(NetBoxModelSerializer):\n    url = serializers.HyperlinkedIdentityField(view_name='dcim-api:device-detail')\n";
    let (code, out) = run("f13", drf(DRF_HEAD.replace("    role = NestedDeviceRoleSerializer()\n", "").as_str(), base, ANSIBLE));
    assert!(!out.contains("[not-evaluated:read-only-field]"), "{out}");
    assert_ne!(code, 4, "{out}");
}

#[test]
fn rs_f11_a_statement_never_overrides_a_conviction() {
    // One Python producer both drops an inbound read the consumer writes
    // (RS-1) and turns device_role read-only (RS-2): VIOLATED, and the RS-2
    // statement is not added.
    let base = format!("{DRF_BASE}\ndef create(request):\n    fmt = request.data[\"pdfFormat\"]\n");
    let head = format!("{DRF_HEAD}\ndef create(request):\n    return None\n");
    let consumer = format!("{ANSIBLE}payload = {{}}\npayload[\"pdfFormat\"] = \"PDF/A-2b\"\n");
    let (code, out) = run("f11", drf(&head, &base, &consumer));
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] rs (schema): [\"pdfFormat\"]"), "{out}");
    assert!(!out.contains("[not-evaluated:read-only-field]"), "{out}");
}

// --------------------------------------------- RS-0 (SYNTHETIC: no corpus case)

fn rs0(tag: &str, consumer: &str, name: &str) -> (i32, String) {
    let files = vec![
        ("head.go", GO_HEAD.to_string()),
        ("base.go", GO_BASE.to_string()),
        (name_static(name), consumer.to_string()),
        ("invar.toml", toml("head.go", "base.go", name)),
    ];
    run(tag, files)
}

fn name_static(n: &str) -> &'static str {
    match n {
        "c.py" => "c.py",
        "c.js" => "c.js",
        _ => panic!("unknown consumer name"),
    }
}

#[test]
fn rs_f06_dynamic_subscript_key_is_opaque_and_stated() {
    let (code, out) = rs0("f06", "data = {}\nfor key, value in opts.items():\n    data[key] = value\n", "c.py");
    assert_eq!(code, 4, "{out}");
    assert!(
        out.contains("inbound read of 'pdfFormat' removed from head.go (read at base.go:6 at base); consumer payload is opaque at c.py:3, so whether it sends 'pdfFormat' is not evaluated [not-evaluated:opaque-payload]"),
        "{out}"
    );
}

#[test]
fn rs_f07_data_name_payload_is_opaque() {
    let (code, out) = rs0("f07", "payload = build(opts)\nrequests.post(url, data=payload)\n", "c.py");
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer payload is opaque at c.py:2"), "{out}");
}

#[test]
fn rs_f08_spread_payload_is_opaque() {
    let (code, out) = rs0("f08", "await fetch(url, {\n  method: \"POST\",\n  body: JSON.stringify({ ...opts }),\n});\n", "c.js");
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer payload is opaque at c.js:3"), "{out}");
}

#[test]
fn rs_f08b_kwargs_payload_is_opaque() {
    let (code, out) = rs0("f08b", "client.convert(**options)\n", "c.py");
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer payload is opaque at c.py:1"), "{out}");
}

#[test]
fn rs_f08c_a_suffix_word_is_not_an_opaque_form() {
    // `metadata = x` contains "data = x" but `data` is not a whole word.
    let (code, out) = rs0("f08c", "metadata = describe(opts)\nprint(metadata)\n", "c.py");
    assert_eq!(code, 0, "{out}");
    assert!(no_rs(&out), "{out}");
}

// ------------------------------------------- interface and determinism

#[test]
fn rs_f14_strict_mode_is_untouched() {
    let t = "[[invariant]]\nkind = \"schema\"\nid = \"rs\"\nproducer = \"head.go\"\nconsumer = \"c.py\"\n";
    let (_, out) = check("f14", &[("head.go", GO_HEAD), ("c.py", PY_WRITES), ("invar.toml", t)]);
    assert!(no_rs(&out), "no producer_base, no request-side analysis:\n{out}");
}

#[test]
fn rs_f19_identical_bytes_give_identical_output() {
    let d = ws("f19", &[("head.go", GO_HEAD), ("base.go", GO_BASE), ("c.py", PY_WRITES), ("invar.toml", &toml("head.go", "base.go", "c.py"))]);
    let a = invar(&d, &["check", "--no-reuse"]);
    let b = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(a, b);
}
