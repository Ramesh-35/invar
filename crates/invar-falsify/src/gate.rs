//! INVAR-GATE-001: the execution gate (`invar gate -- <command>`).
//!
//! Check before, run the command only if the world holds, hash-watch the
//! bound state, re-check after, and STATE any drift. HONEST BOUNDS: the gate
//! brackets, it does not serialize (B1): a non-cooperative writer can still
//! race the command, and drift is detected, never prevented. The gate is a
//! wrapper, not a verdict command (B2): when the wrapped command runs, the
//! exit code follows R-G-5, and only a refusal carries an ISA code. The argv
//! after `--` is executed verbatim, never through a shell (G-D5). Every gate
//! line goes to stderr with the `GATE` prefix; the wrapped command's stdio
//! is inherited untouched (G-D2).

use std::path::{Path, PathBuf};
use std::process::Command;

fn base_dir_of(config: &str) -> PathBuf {
    let p = Path::new(config).parent().map(Path::to_path_buf).unwrap_or_default();
    if p.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        p
    }
}

/// Run this same binary's `check` (the exact semantics, no reimplementation),
/// relay its output to stderr with a phase prefix, and return its exit code.
fn run_check(config: &str, proof: bool, phase: &str) -> i32 {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("GATE: cannot locate this binary for the {phase}-check: {e}");
            return 3;
        }
    };
    let mut c = Command::new(exe);
    c.args(["check", "--config", config]);
    if proof {
        c.arg("--proof");
    }
    let o = match c.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("GATE: the {phase}-check could not run: {e}");
            return 3;
        }
    };
    for l in String::from_utf8_lossy(&o.stdout).lines().chain(String::from_utf8_lossy(&o.stderr).lines()) {
        eprintln!("GATE {phase}: {l}");
    }
    o.status.code().unwrap_or(3)
}

/// G-D4: the watch set is the manifest, the lock, every locked (or declared)
/// local producer file, and, with --proof, every bound proof file. Digests
/// stay in memory; nothing is written (R-G-3).
fn watch_set(config: &str, proof: bool) -> Vec<PathBuf> {
    let base = base_dir_of(config);
    let mut v = vec![PathBuf::from(config), base.join("invar.lock")];
    if let Ok(text) = std::fs::read_to_string(base.join("invar.lock")) {
        if let Ok(lockf) = crate::lock::parse("invar.lock", &text) {
            for c in &lockf.contracts {
                if !c.producer.contains("://") && !c.producer.starts_with("github:") {
                    v.push(base.join(&c.producer));
                }
            }
        }
    }
    if let Ok(raw) = std::fs::read_to_string(config) {
        if let Ok(sections) = crate::config::parse_sections(config, &raw) {
            for (n, kv) in &sections {
                let get = |k: &str| kv.iter().find(|(key, _)| key == k).map(|(_, val)| val.clone());
                if n == "invariant" {
                    if let Some(p) = get("producer") {
                        if !p.contains("://") && !p.starts_with("github:") {
                            v.push(base.join(p));
                        }
                    }
                }
                if proof && n == "proof.import" {
                    if let Some(p) = get("proof") {
                        let full = base.join(&p);
                        if full.is_dir() {
                            if let Ok(rd) = std::fs::read_dir(&full) {
                                let mut es: Vec<PathBuf> = rd
                                    .filter_map(|e| e.ok())
                                    .map(|e| e.path())
                                    .filter(|q| q.extension().is_some_and(|x| x == "proof"))
                                    .collect();
                                es.sort();
                                v.extend(es);
                            }
                        } else {
                            v.push(full);
                        }
                    }
                }
            }
        }
    }
    v.sort();
    v.dedup();
    v
}

fn digest(p: &Path) -> String {
    std::fs::read(p).map(|b| blake3::hash(&b).to_hex().to_string()).unwrap_or_else(|_| "absent".to_string())
}

/// R-G-2, R-G-4, R-G-5. Returns the gate's process exit code.
pub fn run(config: &str, proof: bool, command: &[String]) -> i32 {
    if command.is_empty() {
        eprintln!("GATE: nothing to run; usage: invar gate [--config <path>] [--proof] -- <command> [args...]");
        return 3;
    }
    let pre = run_check(config, proof, "pre");
    if pre != 0 {
        eprintln!("GATE REFUSED (pre-check exit {pre}): command not run");
        return pre;
    }
    let watch = watch_set(config, proof);
    let before: Vec<(PathBuf, String)> = watch.iter().map(|p| (p.clone(), digest(p))).collect();
    eprintln!("GATE: pre-check held; running the command ({} file(s) watched)", before.len());
    let cmd_code = match Command::new(&command[0]).args(&command[1..]).status() {
        Ok(st) => match st.code() {
            Some(c) => c,
            None => {
                // G-INT-2: killed by a signal, no exit code exists; stated,
                // and the gate exits 3 (an error, never a verdict).
                eprintln!("GATE: the command was terminated by a signal (no exit code)");
                3
            }
        },
        Err(e) => {
            eprintln!("GATE: the command could not be started: {e}");
            return 3;
        }
    };
    for (p, d0) in &before {
        if &digest(p) != d0 {
            eprintln!("GATE DRIFT: {} changed while the command ran", p.display());
        }
    }
    let post = run_check(config, proof, "post");
    if cmd_code != 0 {
        eprintln!("GATE: command exit {cmd_code} forwarded (post-check exit {post} stated, never masking the command)");
        return cmd_code;
    }
    eprintln!("GATE: command exit 0; post-check exit {post} inherited");
    post
}
