#!/usr/bin/env python3
"""SECURITY S1 mutation operators S1-M1 to S1-M6 (docs/falsification/
SECURITY-SPEC.md section F), each one textual edit of the implemented tree
written as a unified diff with the operator's DECLARED killer on its first
line. Usage: mkmutants.py <tree> <out dir>"""
import difflib, os, sys

tree, out = sys.argv[1], sys.argv[2]
RU = "crates/invar-falsify/src/reuse.rs"
MA = "crates/invar-falsify/src/main.rs"

M = [
    ("S1-M1", "s1_f02", "unsigned lock records reused locally again (the pre-S1 rule)", MA,
     "            from_lock: args.lock_signers.is_some(),",
     "            from_lock: !args.ci || args.lock_signers.is_some(),"),
    ("S1-M2", "s1_f01", "cache MAC not verified on load", RU,
     "                .filter(|(id, k)| record_mac(&key, id, &k.held) == k.mac)",
     "                .filter(|(id, k)| { let _ = (&key, id, k); true })"),
    ("S1-M3", "s1_f06", "key read from the working directory (the repository) instead of HOME", RU,
     '    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;',
     '    let home = std::ffi::OsString::from(".");'),
    ("S1-M4", "s1_f07", "key file created readable by others", RU,
     "        opts.mode(0o600);",
     "        opts.mode(0o644);"),
    ("S1-M5", "s1_f08", "without a key, cache records are read unverified", RU,
     "    let Some(key) = user_key() else { return BTreeMap::new() };",
     "    let Some(key) = user_key() else {\n        return std::fs::read_to_string(base.join(CACHE_FILE)).ok().and_then(|t| serde_json::from_str::<CacheFile>(&t).ok()).map(|c| c.records.into_iter().map(|(id, k)| (id, k.held)).collect()).unwrap_or_default();\n    };"),
    ("S1-M6", "s1_f09", "MAC computed with a fixed key, not the user's", RU,
     "    blake3::keyed_hash(key, body.as_bytes()).to_hex().to_string()",
     "    let _ = key;\n    blake3::keyed_hash(&[7u8; 32], body.as_bytes()).to_hex().to_string()"),
]

os.makedirs(out, exist_ok=True)
for mid, killer, what, path, old, new in M:
    src = open(os.path.join(tree, path)).read()
    assert src.count(old) == 1, (mid, src.count(old))
    mut = src.replace(old, new)
    diff = difflib.unified_diff(src.splitlines(True), mut.splitlines(True), "a/" + path, "b/" + path)
    with open(os.path.join(out, f"{mid}.patch"), "w") as f:
        f.write(f"# {mid} declared-killer={killer}: {what}\n" + "".join(diff))
    print(mid, killer, what)
