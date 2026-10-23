//! CHECK-INCR-1 (ruling I-D5): the engine identity, INVAR_ENGINE =
//! "<crate version>-<BLAKE3>", where the hash covers every file under src/
//! (paths and bytes, sorted), this crate's Cargo.toml, the workspace
//! Cargo.lock and the compiler version. A build whose verdict logic can
//! differ gets a different identity, so its hold records are never reused.

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut list = Vec::new();
    files(&root.join("src"), &mut list);
    list.push(root.join("Cargo.toml"));
    list.push(root.join("../../Cargo.lock"));
    list.sort();
    let mut h = blake3::Hasher::new();
    for p in &list {
        let rel = p.strip_prefix(&root).unwrap_or(p).to_string_lossy().replace('\\', "/");
        h.update(rel.as_bytes());
        h.update(&[0]);
        h.update(&std::fs::read(p).unwrap_or_default());
        h.update(&[0]);
    }
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let version = std::process::Command::new(rustc)
        .arg("-V")
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();
    h.update(&version);
    println!(
        "cargo:rustc-env=INVAR_ENGINE={}-{}",
        std::env::var("CARGO_PKG_VERSION").unwrap(),
        h.finalize().to_hex()
    );
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../../Cargo.lock");
}
