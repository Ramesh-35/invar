//! `invar`: the INVAR command-line instrument (built from crate `invar-falsify`).
//!
//! Exit codes:
//!   0  execution completed (verdicts may include UNKNOWN unless --strict)
//!   1  configuration / manifest / dependency error; for `check`, at least one
//!      contract VIOLATED
//!   2  strict epistemic failure: any UNKNOWN under --strict, or a ledger break
//!   3  `check` could not evaluate the manifest: an ERROR, never a verdict;
//!      `lock`: LOCK_DRIFT (invar.lock would change) or an error; `lock`,
//!      `mcp` and `fix` never use 1, which means VIOLATED
//!   4  `check`: nothing violated, but some contracts were SKIPPED

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use invar_falsify::check::{evaluate_with, render_report_md};
use invar_falsify::lock;
use invar_falsify::receipt;
use invar_falsify::evaluator::{evaluate_case, CaseResult};
use invar_falsify::init::{discover, render_manifest};
use invar_falsify::ledger::Ledger;
use invar_falsify::manifest::{CompositeConfig, EpistemicStatus, Manifest};
use invar_falsify::report::{RunReport, ENGINE_VERSION};
use invar_falsify::resolver::{CurlTransport, Resolver, Transport};

#[derive(Parser)]
#[command(
    name = "invar",
    version,
    about = "Checks cross-repository contracts: the fields your code reads from another repository's API, locked, verified, and stated honestly (a skip is never a pass). Exit codes: 0 holds, 1 violated, 2 ledger break, 3 error or lock drift, 4 skipped or not evaluated."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Check a cross-repository contract from an invar.toml (CI entrypoint).
    Check(CheckArgs),
    /// Record the producer fields this repo reads in invar.lock.
    Lock(LockArgs),
    /// Scan a workspace and propose an invar.toml (heuristic discovery).
    Init(InitArgs),
    /// Who depends on a producer field, from local invar.lock files.
    Dependents(DependentsArgs),
    /// Replay real caught breaks from bytes pinned in demo/, fully offline.
    /// The VIOLATED verdicts shown are the demonstration, not a failure.
    Demo,
    /// What INVAR evaluates per contract, and what it states instead
    /// (disclosure, never a verdict).
    Coverage(CoverageArgs),
    /// Print a shell completion script (bash | zsh | fish).
    Completions(CompletionsArgs),
    /// Re-run an evidence receipt and confirm it reproduces byte for byte.
    VerifyReceipt(VerifyReceiptArgs),
    /// Receipt views: `receipt export --in-toto` (format 1 stays canonical).
    Receipt(ReceiptArgs),
    /// Suggest consumer patches for unambiguous field renames (never writes).
    Fix(FixArgs),
    /// Read-only MCP server on stdio (tools: check, contract).
    Mcp(McpArgs),
    /// Research harness: evaluate the falsification corpus and persist the run.
    Run(RunArgs),
    /// Research harness: structurally validate a corpus manifest.
    Validate(ValidateArgs),
    /// Research harness: evaluate the corpus and print a report (no persist).
    Report(ReportArgs),
    /// Research harness: re-walk a ledger's bi-temporal BLAKE3 chain.
    Verify(VerifyArgs),
    /// Research harness: verify and summarize a ledger.
    Audit(AuditArgs),
    /// Research harness: reconstruct ledger state at a bi-temporal coordinate.
    Replay(ReplayArgs),
}

#[derive(Parser)]
struct RunArgs {
    #[arg(long, default_value = "corpus_manifest.json")]
    manifest: String,
    #[arg(long, default_value = "composite.toml")]
    composite: String,
    #[arg(long, default_value = "runs/latest")]
    output: String,
    #[arg(long, default_value = "md")]
    format: String,
    #[arg(long, default_value_t = false)]
    strict: bool,
    #[arg(long, default_value_t = 60)]
    timeout: u64,
}

#[derive(Parser)]
struct ValidateArgs {
    #[arg(long, default_value = "corpus_manifest.json")]
    manifest: String,
    #[arg(long)]
    composite: Option<String>,
}

#[derive(Parser)]
struct ReportArgs {
    #[arg(long, default_value = "corpus_manifest.json")]
    manifest: String,
    #[arg(long, default_value = "composite.toml")]
    composite: String,
    #[arg(long, default_value = "md")]
    format: String,
    #[arg(long, default_value_t = 60)]
    timeout: u64,
}

#[derive(Parser)]
struct VerifyArgs {
    #[arg(long)]
    ledger: String,
}

#[derive(Parser)]
struct CheckArgs {
    #[arg(long, default_value = "invar.toml")]
    config: String,
    #[arg(long)]
    ledger: Option<String>,
    /// Allow remote github:OWNER/REPO@REF:PATH artifacts (fetched into memory;
    /// GITHUB_TOKEN and GITHUB_BASE_REF are read from the environment).
    #[arg(long, default_value_t = false)]
    ci: bool,
    /// Also write a markdown report (PR comment / job summary) to this path.
    #[arg(long)]
    report_md: Option<String>,
    /// Also write an A2 evidence receipt (canonical, byte-deterministic) here.
    #[arg(long)]
    receipt: Option<String>,
    /// Baseline ceremony: require invar.lock.sig to verify against this
    /// allowed-signers file before evaluating (fail closed, exit 3).
    #[arg(long)]
    lock_signers: Option<String>,
    /// Principal for --lock-signers (required with it).
    #[arg(long)]
    lock_principal: Option<String>,
    /// Observed reality: compare this fresh payload's shape against the
    /// contract's locked sample (invar lock --sample).
    #[arg(long)]
    sample: Option<String>,
    /// Contract id the fresh sample belongs to (optional when exactly one
    /// contract has a locked sample).
    #[arg(long)]
    contract: Option<String>,
}

#[derive(Parser)]
struct DependentsArgs {
    /// `Type.field` or a bare `field` (naming-form match, as the engine).
    query: String,
    /// Directory to scan for invar.lock files (any depth). Local only: this
    /// command performs no network calls by construction.
    #[arg(long)]
    locks: String,
    /// Only contracts whose producer ref contains this substring.
    #[arg(long)]
    producer: Option<String>,
    /// "text" (default) or "json" (canonical, deterministic).
    #[arg(long, default_value = "text")]
    format: String,
}

#[derive(Parser)]
struct CompletionsArgs {
    /// Which shell: "bash", "zsh" or "fish".
    shell: String,
}

#[derive(Parser)]
struct CoverageArgs {
    #[arg(long, default_value = "invar.toml")]
    config: String,
    /// Only this contract id.
    #[arg(long)]
    contract: Option<String>,
    /// "text" (default) or "json" (canonical, deterministic).
    #[arg(long, default_value = "text")]
    format: String,
}

#[derive(Parser)]
struct ReceiptArgs {
    #[command(subcommand)]
    cmd: ReceiptCmd,
}

#[derive(clap::Subcommand)]
enum ReceiptCmd {
    /// Convert a format 1 receipt into another view. Never alters format 1.
    Export(ReceiptExportArgs),
}

#[derive(Parser)]
struct ReceiptExportArgs {
    /// The format 1 receipt file to export.
    path: String,
    /// Emit an in-toto Statement v1.0 wrapping the receipt body.
    #[arg(long, default_value_t = false)]
    in_toto: bool,
    /// Write here instead of stdout.
    #[arg(long)]
    out: Option<String>,
}

#[derive(Parser)]
struct VerifyReceiptArgs {
    /// The receipt file to verify.
    path: String,
    #[arg(long, default_value = "invar.toml")]
    config: String,
    /// Verify fully offline from a directory of artifacts named by BLAKE3 hex.
    #[arg(long)]
    artifacts: Option<String>,
    /// Also verify the detached SSH signature (<path>.sig) against this
    /// allowed-signers file (ssh-keygen -Y verify, namespace invar-receipt).
    #[arg(long)]
    signers: Option<String>,
    /// Principal to verify the signature as (required with --signers).
    #[arg(long)]
    principal: Option<String>,
}

#[derive(Parser)]
struct LockArgs {
    #[arg(long, default_value = "invar.toml")]
    config: String,
    /// Accept a reviewed producer change: rewrite invar.lock from the live
    /// producer (waivers are kept).
    #[arg(long, default_value_t = false)]
    update: bool,
    /// Baseline ceremony: require invar.lock.sig to verify against this
    /// allowed-signers file (fail closed, exit 3). Sign with:
    /// ssh-keygen -Y sign -n invar-lock -f <key> invar.lock
    #[arg(long)]
    lock_signers: Option<String>,
    /// Principal for --lock-signers (required with it).
    #[arg(long)]
    lock_principal: Option<String>,
    /// Observed reality: merge this real payload file's SHAPE into the
    /// contract's presence witnesses (repeatable; values are never copied;
    /// the file itself stays where it is, and may hold real data: keep it
    /// out of version control unless scrubbed).
    #[arg(long)]
    sample: Vec<String>,
    /// Contract id the sample belongs to (optional when there is exactly one).
    #[arg(long)]
    contract: Option<String>,
}

#[derive(Parser)]
struct McpArgs {
    #[arg(long, default_value = "invar.toml")]
    config: String,
    /// Read local files only (no github: fetches).
    #[arg(long, default_value_t = false)]
    offline: bool,
}

#[derive(Parser)]
struct FixArgs {
    #[arg(long, default_value = "invar.toml")]
    config: String,
    /// Print suggestions as unified diffs. The only mode: fix never writes.
    #[arg(long, default_value_t = false)]
    suggest: bool,
}

#[derive(Parser)]
struct InitArgs {
    /// Workspace root to scan; invar.toml is written here.
    #[arg(long, default_value = ".")]
    root: String,
    /// Overwrite an existing invar.toml.
    #[arg(long, default_value_t = false)]
    force: bool,
    /// Print the proposed manifest instead of writing it.
    #[arg(long, default_value_t = false)]
    stdout: bool,
}

#[derive(Parser)]
struct AuditArgs {
    #[arg(long)]
    ledger: String,
}

#[derive(Parser)]
struct ReplayArgs {
    #[arg(long)]
    ledger: String,
    /// Transaction-time bound: admit only rows recorded at or before this.
    #[arg(long)]
    as_of_tr: i64,
    /// Valid-time bound (default: open).
    #[arg(long)]
    as_of_tv: Option<i64>,
}

fn main() {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        // A bare `invar` renders orientation, never a verdict: no contract
        // was evaluated, so none of the frozen verdict exits apply, and the
        // orientation itself cannot fail. clap's own bare-invocation exit
        // of 2 would collide with "ledger break" in the frozen map.
        print!("{}", ORIENTATION);
        std::process::exit(0);
    };
    let code = match command {
        Command::Run(a) => dispatch(cmd_run(a)),
        Command::Validate(a) => dispatch(cmd_validate(a)),
        Command::Report(a) => dispatch(cmd_report(a)),
        Command::Verify(a) => dispatch(cmd_verify(a)),
        Command::Check(a) => dispatch(cmd_check(a)),
        Command::Lock(a) => dispatch_r2(cmd_lock(a)),
        Command::Mcp(a) => dispatch_r2(cmd_mcp(a)),
        Command::Fix(a) => dispatch_r2(cmd_fix(a)),
        Command::VerifyReceipt(a) => dispatch_r2(cmd_verify_receipt(a)),
        Command::Receipt(a) => dispatch_r2(cmd_receipt(a)),
        Command::Dependents(a) => dispatch_r2(cmd_dependents(a)),
        Command::Demo => dispatch_r2(cmd_demo()),
        Command::Coverage(a) => dispatch_r2(cmd_coverage(a)),
        Command::Completions(a) => dispatch_r2(cmd_completions(a)),
        Command::Init(a) => dispatch(cmd_init(a)),
        Command::Audit(a) => dispatch(cmd_audit(a)),
        Command::Replay(a) => dispatch(cmd_replay(a)),
    };
    std::process::exit(code);
}

/// Map a handler result to a process exit code. Handlers return the intended
/// code on success; any surfaced error is a code-1 (config/dependency) failure.
/// R2 commands keep exit 1 for VIOLATED only: their errors exit 3 (for
/// `invar lock`, 3 also means LOCK_DRIFT: the lock is not confirmed current).
fn dispatch_r2(result: Result<i32>) -> i32 {
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("ERROR (not a verdict): {e:#}");
            3
        }
    }
}

fn dispatch(result: Result<i32>) -> i32 {
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            1
        }
    }
}

fn now_us() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

fn build_report(manifest_path: &str, composite_path: &str, timeout: Duration) -> Result<RunReport> {
    let manifest = Manifest::load(manifest_path)
        .with_context(|| format!("loading manifest {manifest_path}"))?;
    let composite = CompositeConfig::load(composite_path)
        .with_context(|| format!("loading composite config {composite_path}"))?;

    let manifest_bytes = std::fs::read(manifest_path)
        .with_context(|| format!("re-reading manifest {manifest_path} for hashing"))?;
    let manifest_hash = blake3::hash(&manifest_bytes).to_hex().to_string();

    let cases: Vec<CaseResult> = manifest
        .cases
        .iter()
        .map(|tc| evaluate_case(tc, &composite, timeout))
        .collect();

    let timestamp_us = now_us();
    let run_id = blake3::hash(format!("{timestamp_us}:{manifest_hash}").as_bytes())
        .to_hex()
        .to_string()[..16]
        .to_string();

    Ok(RunReport {
        run_id,
        timestamp_us,
        engine_version: ENGINE_VERSION.to_string(),
        manifest_path: manifest_path.to_string(),
        manifest_hash,
        composite_path: composite_path.to_string(),
        cases,
    })
}

fn cmd_validate(args: ValidateArgs) -> Result<i32> {
    let manifest = Manifest::load(&args.manifest)
        .with_context(|| format!("validating manifest {}", args.manifest))?;
    println!(
        "manifest OK: {} case(s), corpus_version {}",
        manifest.cases.len(),
        manifest.corpus_version
    );
    if let Some(composite) = args.composite {
        CompositeConfig::load(&composite)
            .with_context(|| format!("validating composite config {composite}"))?;
        println!("composite config OK: {composite}");
    }
    Ok(0)
}

fn cmd_report(args: ReportArgs) -> Result<i32> {
    let report = build_report(
        &args.manifest,
        &args.composite,
        Duration::from_secs(args.timeout),
    )?;
    let results_json = report
        .to_results_json()
        .context("serializing run results")?;
    let repl = RunReport::replicability_hash(&results_json);
    if args.format == "json" {
        println!("{results_json}");
    } else {
        println!("{}", report.to_markdown(&repl));
    }
    Ok(0)
}

fn cmd_run(args: RunArgs) -> Result<i32> {
    let report = build_report(
        &args.manifest,
        &args.composite,
        Duration::from_secs(args.timeout),
    )?;
    let out_dir = PathBuf::from(&args.output);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating output directory {}", out_dir.display()))?;

    // 1. Write the machine artifact, then hash exactly those bytes.
    let results_json = report
        .to_results_json()
        .context("serializing run results")?;
    let results_path = out_dir.join("run_results.json");
    std::fs::write(&results_path, &results_json)
        .with_context(|| format!("writing {}", results_path.display()))?;
    let repl = RunReport::replicability_hash(&results_json);

    // 2. Render the human report.
    let md = report.to_markdown(&repl);
    let report_path = out_dir.join("EPISTEMIC_REPORT.md");
    std::fs::write(&report_path, &md)
        .with_context(|| format!("writing {}", report_path.display()))?;

    // 3. Persist each verdict into the bi-temporal ledger, then re-verify.
    let ledger_path = out_dir.join("ledger.db");
    let ledger = Ledger::open(ledger_path.to_string_lossy().as_ref())
        .with_context(|| format!("opening ledger {}", ledger_path.display()))?;
    for case in &report.cases {
        let content = serde_json::to_string(case).context("serializing case for ledger")?;
        ledger
            .append_now("case_verdict", &case.id, &content)
            .with_context(|| format!("appending verdict for {}", case.id))?;
    }

    let verified = match ledger.verify() {
        Ok(count) => count,
        Err(e) => {
            eprintln!("LEDGER INTEGRITY BREAK: {e}");
            return Ok(2);
        }
    };

    // 4. Summarize.
    let unknown = report
        .cases
        .iter()
        .filter(|c| c.status == EpistemicStatus::UNKNOWN)
        .count();
    if args.format == "json" {
        println!("{results_json}");
    } else {
        println!("run {} complete", report.run_id);
        for c in &report.cases {
            println!("  {} -> {:?}", c.id, c.status);
        }
        println!("category status: {}", report.category_status());
        println!(
            "artifacts: {} | {} | {} ({} ledger nodes verified)",
            results_path.display(),
            report_path.display(),
            ledger_path.display(),
            verified
        );
    }

    if args.strict && unknown > 0 {
        eprintln!("strict mode: {unknown} case(s) resolved UNKNOWN");
        return Ok(2);
    }
    Ok(0)
}

fn cmd_verify(args: VerifyArgs) -> Result<i32> {
    if !Path::new(&args.ledger).exists() {
        anyhow::bail!("ledger file does not exist: {}", args.ledger);
    }
    let ledger =
        Ledger::open(&args.ledger).with_context(|| format!("opening ledger {}", args.ledger))?;
    match ledger.verify() {
        Ok(count) => {
            let mode = ledger
                .journal_mode()
                .unwrap_or_else(|_| "unknown".to_string());
            println!(
                "ledger OK: {count} node(s), bi-temporal BLAKE3 chain intact, journal_mode={mode}"
            );
            Ok(0)
        }
        Err(e) => {
            eprintln!("ledger integrity FAILED: {e}");
            Ok(2)
        }
    }
}

/// `invar check`: the CI entrypoint. Evaluates every `[[invariant]]` block (or
/// the legacy single schema form) and exits 0 if all contracts hold, 1 if any
/// is VIOLATED, 2 on a ledger integrity break, and 3 if the manifest could not
/// be evaluated (an ERROR, never reported as a verdict). Local paths resolve
/// against the directory holding invar.toml. With `--ci`, remote
/// `github:OWNER/REPO@REF:PATH` artifacts are fetched into memory, hashed with
/// BLAKE3 and never written to disk; their provenance is printed. The reality
/// gate and the PROVEN_RESIDUE verdict stay in the resolving path, not here.
/// The honest number, printed on every check (A1, research adjudication
/// 2026-10-11). Updated only when a new held-out batch is scored; the full
/// record lives in docs/epistemic/STATUS.md.
const MEASURED_RECORD: &str =
    "38 of 70 held-out real cross-repository breaks caught (batches 4-8; docs/epistemic/STATUS.md)";

fn cmd_check(args: CheckArgs) -> Result<i32> {
    let raw = match std::fs::read_to_string(&args.config) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "ERROR (not a verdict): {} not found; run `invar init` to propose one from this repository, or point at one with --config",
                args.config
            );
            return Ok(3);
        }
        Err(e) => {
            eprintln!("ERROR (not a verdict): reading {}: {e}", args.config);
            return Ok(3);
        }
    };
    let sections = match invar_falsify::config::parse_sections(&args.config, &raw) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR (not a verdict): {e}");
            return Ok(3);
        }
    };
    let base_dir = base_dir_of(&args.config);
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let max_bytes = match max_remote_bytes(&sections) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ERROR (not a verdict): {e}");
            return Ok(3);
        }
    };
    let curl = CurlTransport {
        max_bytes,
        ..CurlTransport::default()
    };
    let resolver = Resolver {
        base_dir,
        allow_remote: args.ci,
        token: if args.ci { env("GITHUB_TOKEN") } else { None },
        base_ref: env("GITHUB_BASE_REF"),
        // Live transport only under --ci: curl as an argv subprocess (the
        // zero-dependency build has no HTTPS client). Any fetch failure is an
        // ERROR (exit 3), never a verdict.
        transport: if args.ci {
            Some(&curl as &dyn Transport)
        } else {
            None
        },
        record_local: args.receipt.is_some(),
    };
    let lock_path = base_dir_of(&args.config).join("invar.lock");
    if let Some(signers) = &args.lock_signers {
        if let Err(e) = require_lock_signature(&lock_path, signers, args.lock_principal.as_deref()) {
            eprintln!("ERROR (not a verdict): {e:#}");
            return Ok(3);
        }
    }
    let mut lock_text: Option<String> = None;
    let lock_file = if lock_path.exists() {
        let text = match std::fs::read_to_string(&lock_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("ERROR (not a verdict): reading {}: {e}", lock_path.display());
                return Ok(3);
            }
        };
        match lock::parse(&lock_path.display().to_string(), &text) {
            Ok(l) => {
                lock_text = Some(text);
                Some(l)
            }
            Err(e) => {
                eprintln!("ERROR (not a verdict): {e:#}");
                return Ok(3);
            }
        }
    } else {
        None
    };
    let today = lock::today_utc();
    let mut outcomes = match evaluate_with(&sections, &resolver, lock_file.as_ref(), today) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("ERROR (not a verdict): {e:#}");
            return Ok(3);
        }
    };
    // Observed reality (docs/architecture/OBSERVED-REALITY.md): fresh sample
    // vs locked sample, and staleness. Drift is residue (waivers respected);
    // a stale sample is stated, never silent.
    if let Some(lockf) = lock_file.as_ref() {
        let fresh_v;
        let fresh = match &args.sample {
            Some(path) => {
                let bytes = match std::fs::read(path) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("ERROR (not a verdict): reading sample {path}: {e}");
                        return Ok(3);
                    }
                };
                fresh_v = match serde_json::from_slice::<serde_json::Value>(&bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("ERROR (not a verdict): {path} is not JSON: {e}");
                        return Ok(3);
                    }
                };
                Some((path.as_str(), &fresh_v, args.contract.as_deref()))
            }
            None => None,
        };
        let findings = match sample_findings(lockf, fresh, today) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("ERROR (not a verdict): {e:#}");
                return Ok(3);
            }
        };
        for (cid, residue, stated) in findings {
            let Some(o) = outcomes.iter_mut().find(|o| o.id == cid) else {
                eprintln!("ERROR (not a verdict): locked sample names contract {cid:?}, which invar.toml does not define; if the contract was renamed or removed, re-record its sample with `invar lock --sample --contract <id>`");
                return Ok(3);
            };
            for (field, detail) in residue {
                let waived = lockf.waivers.iter().find(|w| w.contract == cid && w.covers(&field) && w.days_left(today) >= 0);
                if let Some(w) = waived {
                    o.waived.push((field, w.owner.clone(), w.reason.clone(), w.days_left(today)));
                } else {
                    o.residue.push(field.clone());
                    o.witnesses.push(invar_falsify::check::Witness {
                        residue: field,
                        sites: Vec::new(),
                        missing_from: Some(detail),
                        type_change: None,
                        read_as: Vec::new(),
                    });
                }
            }
            o.unevaluated.extend(stated);
        }
    } else if args.sample.is_some() {
        eprintln!("ERROR (not a verdict): --sample needs an invar.lock with a locked sample (run `invar lock`, then `invar lock --sample`)");
        return Ok(3);
    }

    println!(
        "INVAR check ({} contract(s){})",
        outcomes.len(),
        if args.ci { ", --ci" } else { "" }
    );
    let mut violations = 0usize;
    let mut skipped = 0usize;
    let mut summaries: Vec<String> = Vec::new();
    for o in &outcomes {
        let (tag, text) = invar_falsify::check::outcome_lines(o);
        print!("{text}");
        match tag {
            "VIOLATED" => violations += 1,
            "SKIPPED" | "NOT_EVALUATED" => skipped += 1,
            _ => {}
        }
        summaries.push(format!("{}:{}={tag}", o.id, o.kind));
        for f in &o.provenance {
            summaries.push(format!("fetched {} blake3={}", f.reference, f.blake3_hex));
        }
    }
    // Exit 4: nothing violated, but some contracts were skipped (an
    // unsupported language). A skip is never reported as a pass.
    let code = if violations == 0 && skipped > 0 {
        println!(
            "result: NOT FULLY EVALUATED, {skipped} of {} contract(s) skipped or not evaluated, the rest hold",
            outcomes.len()
        );
        4
    } else if violations == 0 {
        println!("result: ALL {} contract(s) hold", outcomes.len());
        0
    } else {
        println!(
            "result: {violations} of {} contract(s) VIOLATED",
            outcomes.len()
        );
        1
    };
    // Fixed summary, printed on every run whatever the counts (A1): "clean"
    // and "unexamined" are never shown the same way, and the measured record
    // travels with every verdict.
    println!(
        "summary: held {} | violated {violations} | skipped or not evaluated {skipped}",
        outcomes.len() - violations - skipped
    );
    println!("measured: {MEASURED_RECORD}");

    if let Some(path) = &args.report_md {
        if let Err(e) = std::fs::write(path, render_report_md(&outcomes)) {
            eprintln!("ERROR (not a verdict): writing report {path}: {e}");
            return Ok(3);
        }
    }

    if let Some(path) = &args.receipt {
        let config_name = std::path::Path::new(&args.config)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| args.config.clone());
        let config_b3 = blake3::hash(raw.as_bytes()).to_hex().to_string();
        let lock_b3 = lock_text
            .as_ref()
            .map(|t| blake3::hash(t.as_bytes()).to_hex().to_string());
        let body = receipt::body(
            env!("CARGO_PKG_VERSION"),
            &lock::format_date(today),
            &config_name,
            &config_b3,
            lock_b3.as_deref(),
            &outcomes,
        );
        if let Err(e) = std::fs::write(path, receipt::render(&body)) {
            eprintln!("ERROR (not a verdict): writing receipt {path}: {e}");
            return Ok(3);
        }
        println!("receipt written: {path}");
    }

    if let Some(path) = &args.ledger {
        let ledger = match Ledger::open(path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("ERROR (not a verdict): opening ledger {path}: {e}");
                return Ok(3);
            }
        };
        let kind = if violations == 0 {
            "all_hold"
        } else {
            "violations"
        };
        if let Err(e) = ledger.append_now(kind, "invar_check", &summaries.join("; ")) {
            eprintln!("ERROR (not a verdict): appending to ledger: {e}");
            return Ok(3);
        }
        match ledger.verify() {
            Ok(n) => println!("ledger: {path} ({n} node(s) verified)"),
            Err(e) => {
                eprintln!("LEDGER INTEGRITY BREAK: {e}");
                return Ok(2);
            }
        }
    }
    Ok(code)
}

/// `invar init`: scan a workspace and propose an invar.toml. Discovery is
/// heuristic; every generated contract is a proposal to review.
fn cmd_init(args: InitArgs) -> Result<i32> {
    let root = PathBuf::from(&args.root);
    if !root.is_dir() {
        anyhow::bail!("{} is not a directory", args.root);
    }
    let d = discover(&root);
    let manifest = render_manifest(&d);
    if args.stdout {
        print!("{manifest}");
        return Ok(0);
    }
    let out = root.join("invar.toml");
    if out.exists() && !args.force {
        anyhow::bail!(
            "{} already exists; pass --force to overwrite or --stdout to print",
            out.display()
        );
    }
    std::fs::write(&out, &manifest).with_context(|| format!("writing {}", out.display()))?;
    println!(
        "INVAR init: scanned {} file(s){}",
        d.files_scanned,
        if d.truncated { " (truncated)" } else { "" }
    );
    println!("  schemas found      : {}", d.schemas.len());
    println!("  consumers found    : {}", d.consumers.len());
    println!(
        "  contracts proposed : {} (heuristic; review before trusting)",
        d.proposals.len()
    );
    println!(
        "  inventory only     : {} item(s) found but not yet checkable",
        d.inventory.len()
    );
    println!("  wrote              : {}", out.display());
    Ok(0)
}

/// `invar audit`: verify a ledger and summarize its nodes. The adequacy matrix
/// is NOT emitted from the binary: its statuses are adjudicated in the doc, and
/// hardcoding them here could silently drift from reality, so we point to it.
fn cmd_audit(args: AuditArgs) -> Result<i32> {
    if !Path::new(&args.ledger).exists() {
        anyhow::bail!("ledger file does not exist: {}", args.ledger);
    }
    let ledger =
        Ledger::open(&args.ledger).with_context(|| format!("opening ledger {}", args.ledger))?;
    let count = match ledger.verify() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ledger integrity FAILED: {e}");
            return Ok(2);
        }
    };
    let mode = ledger
        .journal_mode()
        .unwrap_or_else(|_| "unknown".to_string());
    let nodes = ledger.replay_as_of(i64::MAX, i64::MAX)?;
    println!("INVAR audit");
    println!(
        "  ledger        : {} ({count} node(s) verified, journal_mode={mode})",
        args.ledger
    );
    for n in &nodes {
        println!(
            "  - id={} kind={} subject={} t_v={} t_r={}",
            n.id, n.kind, n.subject, n.valid_time_us, n.transaction_time_us
        );
    }
    println!("  adequacy matrix: docs/epistemic/MATHEMATICAL-ADEQUACY.md (adjudicated there,");
    println!("                   not emitted by this binary, so it cannot silently drift).");
    Ok(0)
}

/// `invar replay`: reconstruct ledger state as known at a bi-temporal
/// coordinate, excluding anything recorded or valid after the bound.
fn cmd_replay(args: ReplayArgs) -> Result<i32> {
    if !Path::new(&args.ledger).exists() {
        anyhow::bail!("ledger file does not exist: {}", args.ledger);
    }
    let ledger =
        Ledger::open(&args.ledger).with_context(|| format!("opening ledger {}", args.ledger))?;
    let tv = args.as_of_tv.unwrap_or(i64::MAX);
    let nodes = ledger
        .replay_as_of(args.as_of_tr, tv)
        .map_err(|e| anyhow::anyhow!("replay failed (ledger integrity?): {e}"))?;
    println!("INVAR replay as-of t_r={} t_v={}", args.as_of_tr, tv);
    println!("  admissible nodes: {}", nodes.len());
    for n in &nodes {
        println!(
            "  - id={} kind={} subject={} t_v={} t_r={}",
            n.id, n.kind, n.subject, n.valid_time_us, n.transaction_time_us
        );
    }
    Ok(0)
}

fn base_dir_of(config: &str) -> PathBuf {
    Path::new(config)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `[check] max_remote_mb = N` (1 to the hard cap) or the default.
fn max_remote_bytes(sections: &[(String, Vec<(String, String)>)]) -> Result<usize, String> {
    use invar_falsify::resolver::{HARD_MAX_REMOTE_BYTES, MAX_REMOTE_BYTES};
    match sections
        .iter()
        .find(|(n, _)| n == "check")
        .and_then(|(_, p)| p.iter().find(|(k, _)| k == "max_remote_mb"))
    {
        None => Ok(MAX_REMOTE_BYTES),
        Some((_, v)) => match v.trim().parse::<usize>() {
            Ok(mb) if mb >= 1 && mb * 1_000_000 <= HARD_MAX_REMOTE_BYTES => Ok(mb * 1_000_000),
            _ => Err(format!(
                "[check] max_remote_mb must be a whole number from 1 to {}",
                HARD_MAX_REMOTE_BYTES / 1_000_000
            )),
        },
    }
}

/// `invar lock`: write invar.lock beside invar.toml. Without `--update` it
/// creates the file, or confirms it is current; it never overwrites a lock
/// whose bindings would change (that is a reviewed `--update`).
fn cmd_lock(args: LockArgs) -> Result<i32> {
    if !args.sample.is_empty() {
        return cmd_lock_sample(&args);
    }
    let raw = std::fs::read_to_string(&args.config)
        .with_context(|| format!("reading {}", args.config))?;
    let sections = invar_falsify::config::parse_sections(&args.config, &raw)?;
    let base_dir = base_dir_of(&args.config);
    let lock_path = base_dir.join("invar.lock");
    if let Some(signers) = &args.lock_signers {
        require_lock_signature(&lock_path, signers, args.lock_principal.as_deref())?;
    }
    let prev = if lock_path.exists() {
        lock::parse(
            &lock_path.display().to_string(),
            &std::fs::read_to_string(&lock_path)?,
        )?
    } else {
        lock::Lock::default()
    };
    let curl = CurlTransport {
        max_bytes: max_remote_bytes(&sections).map_err(anyhow::Error::msg)?,
        ..CurlTransport::default()
    };
    let resolver = Resolver {
        base_dir,
        allow_remote: true,
        token: std::env::var("GITHUB_TOKEN").ok().filter(|v| !v.is_empty()),
        base_ref: None,
        transport: Some(&curl as &dyn Transport),
        record_local: false,
    };
    lock::validate_waivers(&prev.waivers, lock::today_utc())?;
    let next = lock::compute(&sections, &resolver, &prev, lock::today_utc())?;
    let changes = lock::diff(&prev, &next);
    let exists = lock_path.exists();
    if exists && changes.is_empty() {
        println!("invar.lock is current ({} contract(s))", next.contracts.len());
        return Ok(0);
    }
    for c in &changes {
        println!("  {c}");
    }
    if exists && !args.update {
        println!("LOCK_DRIFT: invar.lock would change; review the lines above, then run `invar lock --update`");
        return Ok(3);
    }
    std::fs::write(&lock_path, lock::render(&next)?)
        .with_context(|| format!("writing {}", lock_path.display()))?;
    let n: usize = next.contracts.iter().map(|c| c.bindings.len()).sum();
    println!(
        "wrote {} ({} contract(s), {n} binding(s))",
        lock_path.display(),
        next.contracts.len()
    );
    Ok(0)
}

/// `invar mcp`: serve `check` and `contract` over stdio until EOF.
fn cmd_mcp(args: McpArgs) -> Result<i32> {
    let curl = CurlTransport::default();
    let server = invar_falsify::mcp::Server {
        config: PathBuf::from(&args.config),
        transport: if args.offline { None } else { Some(&curl as &dyn Transport) },
        token: std::env::var("GITHUB_TOKEN").ok().filter(|v| !v.is_empty()),
        today: lock::today_utc(),
    };
    let stdin = std::io::stdin();
    server.serve(stdin.lock(), std::io::stdout())?;
    Ok(0)
}

/// `invar fix --suggest`: for each locked contract, propose a consumer patch
/// where a bound field was renamed 1:1 in its type. Prints only.
fn cmd_fix(args: FixArgs) -> Result<i32> {
    if !args.suggest {
        anyhow::bail!("`invar fix` only suggests: run `invar fix --suggest` (it never writes files)");
    }
    if std::env::var("CI").map(|v| v == "true" || v == "1").unwrap_or(false) {
        anyhow::bail!("`invar fix --suggest` is for local review and does not run in CI");
    }
    let raw = std::fs::read_to_string(&args.config)
        .with_context(|| format!("reading {}", args.config))?;
    let sections = invar_falsify::config::parse_sections(&args.config, &raw)?;
    let base_dir = base_dir_of(&args.config);
    let lock_file = invar_falsify::mcp::load_lock(Path::new(&args.config))?
        .context("no invar.lock beside invar.toml; run `invar lock` first")?;
    let curl = CurlTransport::default();
    let resolver = Resolver {
        base_dir,
        allow_remote: true,
        token: std::env::var("GITHUB_TOKEN").ok().filter(|v| !v.is_empty()),
        base_ref: None,
        transport: Some(&curl as &dyn Transport),
        record_local: false,
    };
    let mut any = false;
    for b in invar_falsify::check::gather_blocks(&sections)? {
        let get = |k: &str| b.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone());
        let Some(id) = get("id") else { continue };
        let Some(entry) = lock_file.contracts.iter().find(|c| c.id == id) else { continue };
        let (producer, consumer) = (get("producer").unwrap_or_default(), get("consumer").unwrap_or_default());
        let r = lock::check_contract(
            entry,
            &producer,
            get("pointer").as_deref(),
            get("producer_code").as_deref(),
            &consumer,
            &resolver,
            &mut Vec::new(),
        )?;
        if r.residue.is_empty() {
            continue;
        }
        let text = resolver.resolve(&consumer).map_err(|e| anyhow::anyhow!("{e}"))?.text;
        let residue: Vec<(String, bool)> = r
            .witnesses
            .iter()
            .map(|w| (w.residue.clone(), w.missing_from.is_some() && w.type_change.is_none()))
            .collect();
        let name = invar_falsify::check::artifact_name(&consumer);
        for s in invar_falsify::fix::suggest(entry, &r.live_members, &residue, &name, &text) {
            any = true;
            match (&s.rename_to, &s.patch) {
                (Some(to), Some(p)) => println!("# {id}: {} renamed to {to}\n{p}", s.residue),
                (Some(to), None) => println!("# {id}: {} renamed to {to}; {}", s.residue, s.note.unwrap_or_default()),
                _ => println!("# {id}: {}: no suggestion ({})", s.residue, s.note.unwrap_or_default()),
            }
        }
    }
    if !any {
        println!("no residue in locked contracts: nothing to suggest");
    }
    Ok(0)
}

/// `invar verify-receipt`: re-run an A2 evidence receipt and confirm it
/// reproduces byte for byte. Exit 0 reproduced; 1 not reproduced (input
/// drift, signature failure, or verdict/byte mismatch: the receipt's claim
/// does not hold today); 3 error (corrupted receipt, wrong engine version,
/// unreadable inputs). 1 is about the receipt's claim, never the contracts.
fn cmd_verify_receipt(args: VerifyReceiptArgs) -> Result<i32> {

    let text = std::fs::read_to_string(&args.path)
        .map_err(|e| anyhow::anyhow!("reading {}: {e}", args.path))?;
    let parsed = receipt::parse(&text)?;
    let body = &parsed.body;
    let format = body.get("receipt_format").and_then(|v| v.as_u64());
    if format != Some(receipt::RECEIPT_FORMAT) {
        anyhow::bail!(
            "receipt_format {:?} is not {} (this engine cannot evaluate it)",
            format,
            receipt::RECEIPT_FORMAT
        );
    }
    let recorded_engine = body
        .pointer("/engine/version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if recorded_engine != env!("CARGO_PKG_VERSION") {
        anyhow::bail!(
            "receipt was produced by engine {recorded_engine}; this is {} (reproduction is only promised by the same version)",
            env!("CARGO_PKG_VERSION")
        );
    }

    // Optional detached SSH signature over the whole receipt file.
    if let Some(signers) = &args.signers {
        let principal = args
            .principal
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--signers needs --principal"))?;
        let sig = format!("{}.sig", args.path);
        if !std::path::Path::new(&sig).exists() {
            println!("signature missing: {sig}");
            println!("result: receipt NOT verified");
            return Ok(1);
        }
        let mut child = std::process::Command::new("ssh-keygen")
            .args(["-Y", "verify", "-f", signers, "-I", principal, "-n", "invar-receipt", "-s", &sig])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("running ssh-keygen: {e}"))?;
        use std::io::Write as _;
        child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("ssh-keygen stdin unavailable"))?
            .write_all(text.as_bytes())?;
        if !child.wait()?.success() {
            println!("signature verification failed ({sig}, principal {principal})");
            println!("result: receipt NOT verified");
            return Ok(1);
        }
        println!("signature ok ({principal})");
    }

    // Inputs: the config and lock must be the recorded bytes.
    let raw = std::fs::read_to_string(&args.config)
        .map_err(|e| anyhow::anyhow!("reading {}: {e}", args.config))?;
    let config_b3 = blake3::hash(raw.as_bytes()).to_hex().to_string();
    if body.pointer("/config/blake3").and_then(|v| v.as_str()) != Some(config_b3.as_str()) {
        println!("input drift: {} is not the config the receipt was made from", args.config);
        println!("result: receipt NOT reproduced");
        return Ok(1);
    }
    let sections = invar_falsify::config::parse_sections(&args.config, &raw)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let lock_path = base_dir_of(&args.config).join("invar.lock");
    let recorded_lock = body.pointer("/lock/blake3").and_then(|v| v.as_str());
    let mut lock_file = None;
    match (recorded_lock, lock_path.exists()) {
        (None, false) => {}
        (None, true) => {
            println!("input drift: {} exists but the receipt was made without a lock", lock_path.display());
            println!("result: receipt NOT reproduced");
            return Ok(1);
        }
        (Some(_), false) => {
            println!("input drift: the receipt was made with invar.lock, which is now missing");
            println!("result: receipt NOT reproduced");
            return Ok(1);
        }
        (Some(h), true) => {
            let t = std::fs::read_to_string(&lock_path)
                .map_err(|e| anyhow::anyhow!("reading {}: {e}", lock_path.display()))?;
            if blake3::hash(t.as_bytes()).to_hex().to_string() != h {
                println!("input drift: invar.lock differs from the receipt");
                println!("result: receipt NOT reproduced");
                return Ok(1);
            }
            lock_file = Some(lock::parse(&lock_path.display().to_string(), &t)?);
        }
    }
    let as_of = body
        .get("as_of")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("receipt has no as_of"))?;
    let today = lock::parse_date(as_of)
        .ok_or_else(|| anyhow::anyhow!("unreadable as_of {as_of:?}"))?;

    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let max_bytes = max_remote_bytes(&sections).map_err(anyhow::Error::msg)?;
    let dir_t;
    let curl_t;
    let resolver = Resolver {
        base_dir: base_dir_of(&args.config),
        allow_remote: true,
        token: match &args.artifacts {
            Some(_) => None, // tokenless, so urls invert to refs
            None => env("GITHUB_TOKEN"),
        },
        base_ref: None,
        transport: match &args.artifacts {
            Some(d) => {
                dir_t = receipt::DirTransport::from_receipt(std::path::Path::new(d), body);
                Some(&dir_t as &dyn Transport)
            }
            None => {
                curl_t = CurlTransport { max_bytes, ..CurlTransport::default() };
                Some(&curl_t as &dyn Transport)
            }
        },
        record_local: true,
    };
    let outcomes = evaluate_with(&sections, &resolver, lock_file.as_ref(), today)?;

    // Input drift: any artifact whose bytes differ from the recorded hash.
    let mut recorded: std::collections::BTreeMap<&str, &str> = Default::default();
    if let Some(arr) = body.get("artifacts").and_then(|v| v.as_array()) {
        for a in arr {
            if let (Some(r), Some(h)) = (
                a.get("ref").and_then(|v| v.as_str()),
                a.get("blake3").and_then(|v| v.as_str()),
            ) {
                recorded.insert(r, h);
            }
        }
    }
    let mut drift = Vec::new();
    for o in &outcomes {
        for f in &o.provenance {
            match recorded.get(f.reference.as_str()) {
                Some(h) if *h == f.blake3_hex => {}
                Some(_) => drift.push(format!("{} (bytes changed)", f.reference)),
                None => drift.push(format!("{} (not in the receipt)", f.reference)),
            }
        }
    }
    drift.sort();
    drift.dedup();
    if !drift.is_empty() {
        for d in &drift {
            println!("input drift: {d}");
        }
        println!("result: receipt NOT reproduced (the inputs are no longer the recorded bytes)");
        return Ok(1);
    }

    let config_name = std::path::Path::new(&args.config)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| args.config.clone());
    let lock_b3 = recorded_lock.map(str::to_string);
    let rebuilt = receipt::body(
        env!("CARGO_PKG_VERSION"),
        as_of,
        &config_name,
        &config_b3,
        lock_b3.as_deref(),
        &outcomes,
    );
    let rebuilt_text = serde_json::to_string_pretty(&rebuilt).unwrap_or_default();
    if rebuilt_text == parsed.body_text {
        println!("result: receipt reproduced byte for byte ({} contract(s))", outcomes.len());
        Ok(0)
    } else {
        let diff = rebuilt_text
            .lines()
            .zip(parsed.body_text.lines())
            .position(|(a, b)| a != b)
            .map(|i| i + 1)
            .unwrap_or(0);
        println!("result: receipt NOT reproduced (first differing line {diff})");
        Ok(1)
    }
}

/// Baseline ceremony check: when a signers file is supplied, invar.lock.sig
/// must verify (namespace invar-lock) before any evaluation. Fail closed:
/// a missing lock, missing signature, or rejected signature is an error
/// (exit 3), never a verdict.
fn require_lock_signature(
    lock_path: &std::path::Path,
    signers: &str,
    principal: Option<&str>,
) -> Result<()> {
    let principal =
        principal.ok_or_else(|| anyhow::anyhow!("--lock-signers needs --lock-principal"))?;
    if !lock_path.exists() {
        anyhow::bail!(
            "--lock-signers given but {} does not exist",
            lock_path.display()
        );
    }
    let sig = lock_path.with_extension("lock.sig");
    if !sig.exists() {
        anyhow::bail!(
            "lock not confirmed: {} is missing; sign with: ssh-keygen -Y sign -n invar-lock -f <key> {}",
            sig.display(),
            lock_path.display()
        );
    }
    let payload = std::fs::read(lock_path)?;
    if !receipt::verify_ssh(
        &payload,
        &sig,
        std::path::Path::new(signers),
        principal,
        "invar-lock",
    )? {
        anyhow::bail!(
            "lock not confirmed: {} does not verify against {} as {}",
            sig.display(),
            signers,
            principal
        );
    }
    println!("lock signature ok ({principal})");
    Ok(())
}

/// `invar receipt export`: views over a format 1 receipt. Format 1 stays
/// canonical; exporting never alters or re-signs it.
fn cmd_receipt(args: ReceiptArgs) -> Result<i32> {
    match args.cmd {
        ReceiptCmd::Export(a) => {
            if !a.in_toto {
                anyhow::bail!("nothing to export: pass --in-toto");
            }
            let text = std::fs::read_to_string(&a.path)
                .map_err(|e| anyhow::anyhow!("reading {}: {e}", a.path))?;
            let parsed = receipt::parse(&text)?;
            let statement = receipt::to_in_toto(&parsed.body);
            let rendered = format!(
                "{}\n",
                serde_json::to_string_pretty(&statement).unwrap_or_default()
            );
            match &a.out {
                Some(p) => {
                    std::fs::write(p, &rendered)
                        .map_err(|e| anyhow::anyhow!("writing {p}: {e}"))?;
                    println!("in-toto statement written: {p}");
                }
                None => print!("{rendered}"),
            }
            Ok(0)
        }
    }
}

/// A4 blast radius: zero network, zero verdict. Exit 0 with the answer
/// (an empty answer included); errors exit 3.
/// What a bare `invar` prints: the three commands that matter, one line
/// each, and where everything else lives. Nothing more.
const ORIENTATION: &str = "invar checks cross-repository contracts: the fields your code reads\nfrom another repository's API, locked, verified, stated honestly.\n\n  invar demo    replay three real caught breaks, offline, in milliseconds\n  invar init    scan this repository and propose an invar.toml\n  invar check   evaluate the contracts in invar.toml\n\n  invar --help  every command and the exit codes\n";

/// `invar completions <shell>`: print the embedded static completion
/// script. Hand-written and shipped in completions/ (zero dependencies,
/// no clap_complete); test 43 keeps them in step with the command set.
fn cmd_completions(args: CompletionsArgs) -> Result<i32> {
    let script = match args.shell.as_str() {
        "bash" => include_str!("../completions/invar.bash"),
        "zsh" => include_str!("../completions/invar.zsh"),
        "fish" => include_str!("../completions/invar.fish"),
        other => {
            anyhow::bail!("unknown shell {other:?} (bash | zsh | fish)");
        }
    };
    print!("{script}");
    Ok(0)
}

/// `invar demo`: replay the shipped real breaks, offline, stock engine path.
fn cmd_demo() -> Result<i32> {
    let cwd = std::env::current_dir().context("getting the current directory")?;
    let mut out = String::new();
    let code = invar_falsify::demo::run_demo(&cwd, &mut out)?;
    print!("{out}");
    Ok(code)
}

/// `invar coverage`: per-contract evaluation disclosure, never a verdict.
fn cmd_coverage(args: CoverageArgs) -> Result<i32> {
    let mut out = String::new();
    let code = invar_falsify::demo::run_coverage(
        &args.config,
        args.contract.as_deref(),
        &args.format,
        MEASURED_RECORD,
        &mut out,
    )?;
    print!("{out}");
    Ok(code)
}

fn cmd_dependents(args: DependentsArgs) -> Result<i32> {
    use invar_falsify::dependents;
    let a = dependents::find(
        std::path::Path::new(&args.locks),
        &args.query,
        args.producer.as_deref(),
    )?;
    match args.format.as_str() {
        "text" => print!("{}", dependents::render_text(&args.query, &a)),
        "json" => print!("{}", dependents::render_json(&args.query, args.producer.as_deref(), &a)),
        other => anyhow::bail!("unknown --format {other:?}: text or json"),
    }
    // An answer with unreadable lockfiles is an answer with stated blind
    // spots: exit 4, never a silent 0 (same convention as NOT EVALUATED).
    Ok(if a.parse_errors.is_empty() { 0 } else { 4 })
}

/// `invar lock --sample <payload.json>`: lock the SHAPE of one real payload
/// under a contract (docs/architecture/OBSERVED-REALITY.md). The payload's
/// values are never copied into the lock; only key paths, types, the file's
/// BLAKE3 and the date. Writing a sample moves the lock to format 2.
fn cmd_lock_sample(args: &LockArgs) -> Result<i32> {
    let lock_path = base_dir_of(&args.config).join("invar.lock");
    if let Some(signers) = &args.lock_signers {
        require_lock_signature(&lock_path, signers, args.lock_principal.as_deref())?;
    }
    if !lock_path.exists() {
        anyhow::bail!(
            "{} does not exist; run `invar lock` first, then add the sample",
            lock_path.display()
        );
    }
    let text = std::fs::read_to_string(&lock_path)?;
    let mut lockf = lock::parse(&lock_path.display().to_string(), &text)?;
    let contract = match (&args.contract, lockf.contracts.len()) {
        (Some(c), _) => c.clone(),
        (None, 1) => lockf.contracts[0].id.clone(),
        (None, n) => anyhow::bail!("the lock has {n} contracts; say which with --contract <id>"),
    };
    if !lockf.contracts.iter().any(|c| c.id == contract) {
        anyhow::bail!("no contract {contract:?} in {}", lock_path.display());
    }
    let today = lock::format_date(lock::today_utc());
    let mut sm = lockf
        .samples
        .iter()
        .find(|x| x.contract == contract)
        .cloned()
        .unwrap_or(lock::Sample {
            contract: contract.clone(),
            blake3: String::new(),
            taken: today.clone(),
            window_start: String::new(),
            observations: 0,
            witnesses: Vec::new(),
        });
    for sample_path in &args.sample {
        let bytes = std::fs::read(sample_path)
            .with_context(|| format!("reading sample {sample_path}"))?;
        let v: serde_json::Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("{sample_path} is not JSON"))?;
        let fields = lock::payload_shape(&v);
        if fields.is_empty() {
            anyhow::bail!("{sample_path}: no fields derived (is it an object or array of objects?)");
        }
        for p in lock::pii_suspects(&fields) {
            eprintln!("warning: sample key {p:?} looks like personal data or a credential; scrub the sample file before committing it anywhere");
        }
        lock::merge_observation(&mut sm, &fields, &today, &blake3::hash(&bytes).to_hex().to_string());
    }
    eprintln!("note: only payload SHAPES are locked; the sample files may hold real data, are not retained, and stay your responsibility");
    let (n_paths, n_obs) = (sm.witnesses.len(), sm.observations);
    lockf.samples.retain(|x| x.contract != contract);
    lockf.samples.push(sm);
    lockf.samples.sort_by(|a, b| a.contract.cmp(&b.contract));
    std::fs::write(&lock_path, lock::render(&lockf)?)?;
    println!(
        "witnessed {} observation(s) under {contract:?}: {n_paths} path(s), {n_obs} total observation(s); {} is format 3",
        args.sample.len(),
        lock_path.display()
    );
    if lock_path.with_extension("lock.sig").exists() {
        println!("note: invar.lock changed; re-sign it (ssh-keygen -Y sign -n invar-lock -f <key> {})", lock_path.display());
    }
    Ok(0)
}

/// Check-side observed-reality evaluation: fresh sample vs locked sample
/// (drift is residue, waivers respected), and staleness (stated, exit 4
/// path). Returns per-contract (residue additions, unevaluated additions).
fn sample_findings(
    lockf: &invar_falsify::lock::Lock,
    fresh: Option<(&str, &serde_json::Value, Option<&str>)>,
    today: i64,
) -> Result<Vec<(String, Vec<(String, String)>, Vec<String>)>> {
    use invar_falsify::lock::{parse_date, payload_shape, witness_drift, SAMPLE_STALE_DAYS};
    let mut out: Vec<(String, Vec<(String, String)>, Vec<String>)> = Vec::new();
    fn entry(out: &mut Vec<(String, Vec<(String, String)>, Vec<String>)>, id: &str) -> usize {
        if let Some(i) = out.iter().position(|(c, _, _)| c == id) {
            i
        } else {
            out.push((id.to_string(), Vec::new(), Vec::new()));
            out.len() - 1
        }
    }
    for sm in &lockf.samples {
        if let Some(taken) = parse_date(&sm.taken) {
            if today - taken > SAMPLE_STALE_DAYS {
                let i = entry(&mut out, &sm.contract);
                out[i].2.push(format!(
                    "observed sample is stale (taken {}, older than {SAMPLE_STALE_DAYS} days); refresh with `invar lock --sample` [not-evaluated:stale-sample]",
                    sm.taken
                ));
            }
        }
    }
    if let Some((path, v, wanted)) = fresh {
        let target = match wanted {
            Some(c) => c.to_string(),
            None => {
                let with: Vec<&str> = lockf.samples.iter().map(|s| s.contract.as_str()).collect();
                match with.as_slice() {
                    [one] => one.to_string(),
                    [] => anyhow::bail!("--sample given but no contract has a locked sample; run `invar lock --sample` first"),
                    _ => anyhow::bail!("several contracts have locked samples; say which with --contract <id>"),
                }
            }
        };
        let locked = lockf
            .samples
            .iter()
            .find(|s| s.contract == target)
            .ok_or_else(|| anyhow::anyhow!("no locked sample for contract {target:?}"))?;
        let fresh_shape = payload_shape(v);
        let i = entry(&mut out, &target);
        let (residue, stated) = witness_drift(locked, &fresh_shape, path);
        out[i].1.extend(residue);
        out[i].2.extend(stated);
    }
    Ok(out)
}
