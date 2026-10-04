//! `invar-falsify` — the INVAR Phase-1 falsification instrument CLI.
//!
//! Exit codes:
//!   0  execution completed (verdicts may include UNKNOWN unless --strict)
//!   1  configuration / manifest / dependency error
//!   2  strict epistemic failure: any UNKNOWN under --strict, or a ledger break

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use invar_falsify::evaluator::{evaluate_case, CaseResult};
use invar_falsify::ledger::Ledger;
use invar_falsify::manifest::{CompositeConfig, EpistemicStatus, EvidenceTier, Manifest};
use invar_falsify::reality::{
    derive_probe_hash, differential_residue, verify_real_tier, RealArtifactProof,
};
use invar_falsify::report::{RunReport, ENGINE_VERSION};

#[derive(Parser)]
#[command(
    name = "invar-falsify",
    version = "0.1.0-alpha",
    about = "Deterministic falsification harness for the INVAR category-residue hypothesis."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Evaluate the corpus and persist the run (results, report, ledger).
    Run(RunArgs),
    /// Structurally validate a manifest (and optionally a composite config).
    Validate(ValidateArgs),
    /// Evaluate the corpus and print a report to stdout without persisting.
    Report(ReportArgs),
    /// Re-walk a ledger and confirm its bi-temporal BLAKE3 chain.
    Verify(VerifyArgs),
    /// Check a cross-repository contract from an invar.toml (CI entrypoint).
    Check(CheckArgs),
    /// Verify and summarize a ledger; point to the adequacy matrix.
    Audit(AuditArgs),
    /// Reconstruct ledger state as known at a bi-temporal coordinate.
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
    let code = match cli.command {
        Command::Run(a) => dispatch(cmd_run(a)),
        Command::Validate(a) => dispatch(cmd_validate(a)),
        Command::Report(a) => dispatch(cmd_report(a)),
        Command::Verify(a) => dispatch(cmd_verify(a)),
        Command::Check(a) => dispatch(cmd_check(a)),
        Command::Audit(a) => dispatch(cmd_audit(a)),
        Command::Replay(a) => dispatch(cmd_replay(a)),
    };
    std::process::exit(code);
}

/// Map a handler result to a process exit code. Handlers return the intended
/// code on success; any surfaced error is a code-1 (config/dependency) failure.
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

fn build_report(
    manifest_path: &str,
    composite_path: &str,
    timeout: Duration,
) -> Result<RunReport> {
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
    let report = build_report(&args.manifest, &args.composite, Duration::from_secs(args.timeout))?;
    let results_json = report.to_results_json().context("serializing run results")?;
    let repl = RunReport::replicability_hash(&results_json);
    if args.format == "json" {
        println!("{results_json}");
    } else {
        println!("{}", report.to_markdown(&repl));
    }
    Ok(0)
}

fn cmd_run(args: RunArgs) -> Result<i32> {
    let report = build_report(&args.manifest, &args.composite, Duration::from_secs(args.timeout))?;
    let out_dir = PathBuf::from(&args.output);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating output directory {}", out_dir.display()))?;

    // 1. Write the machine artifact, then hash exactly those bytes.
    let results_json = report.to_results_json().context("serializing run results")?;
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
    let ledger = Ledger::open(&args.ledger)
        .with_context(|| format!("opening ledger {}", args.ledger))?;
    match ledger.verify() {
        Ok(count) => {
            let mode = ledger.journal_mode().unwrap_or_else(|_| "unknown".to_string());
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

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn section_get(
    sections: &[(String, Vec<(String, String)>)],
    sec: &str,
    key: &str,
) -> Option<String> {
    sections
        .iter()
        .find(|(n, _)| n == sec)
        .and_then(|(_, p)| p.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()))
}

/// `invar check`: the CI entrypoint. Reads an invar.toml, runs the
/// cross-repository differential probe over the two LOCAL checkouts, binds the
/// derived residue through the reality gate when commit provenance is given,
/// and exits non-zero when the contract is violated. It reports INVAR's own
/// differential finding; the full PROVEN_RESIDUE epistemic verdict additionally
/// requires an executed composite baseline, which `check` does not fabricate.
fn cmd_check(args: CheckArgs) -> Result<i32> {
    let raw = std::fs::read_to_string(&args.config)
        .with_context(|| format!("reading {}", args.config))?;
    let sections = invar_falsify::config::parse_sections(&args.config, &raw)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let producer_schema_path = section_get(&sections, "producer", "schema")
        .context("invar.toml needs [producer] schema = \"<path>\"")?;
    let consumer_source_path = section_get(&sections, "consumer", "source")
        .context("invar.toml needs [consumer] source = \"<path>\"")?;
    let producer_schema = std::fs::read_to_string(&producer_schema_path)
        .with_context(|| format!("reading producer schema {producer_schema_path}"))?;
    let consumer_source = std::fs::read_to_string(&consumer_source_path)
        .with_context(|| format!("reading consumer source {consumer_source_path}"))?;

    let residue = differential_residue(&producer_schema, &consumer_source);
    let probe = derive_probe_hash(&residue);

    let p_repo = section_get(&sections, "producer", "repo").unwrap_or_default();
    let p_commit = section_get(&sections, "producer", "commit").unwrap_or_default();
    let c_repo = section_get(&sections, "consumer", "repo").unwrap_or_default();
    let c_commit = section_get(&sections, "consumer", "commit").unwrap_or_default();
    let gate = if !p_commit.is_empty() && !c_commit.is_empty() {
        let proof = RealArtifactProof {
            producer_repo: p_repo.clone(),
            producer_commit: p_commit.clone(),
            consumer_repo: c_repo.clone(),
            consumer_commit: c_commit.clone(),
            diff_probe_hash: probe,
        };
        match verify_real_tier(EvidenceTier::REAL, Some(&proof), Some(probe)) {
            Ok(()) => "ADMITTED (proof binds to derived residue)",
            Err(_) => "REFUSED",
        }
    } else {
        "n/a (no commit provenance; treated as FIXTURE)"
    };

    println!("INVAR check");
    println!("  producer schema : {producer_schema_path}");
    println!("  consumer source : {consumer_source_path}");
    if !p_commit.is_empty() {
        println!("  producer        : {p_repo}@{p_commit}");
    }
    if !c_commit.is_empty() {
        println!("  consumer        : {c_repo}@{c_commit}");
    }
    println!("  reality gate    : {gate}");
    println!("  probe hash      : {}", hex(&probe));

    let code = if residue.is_empty() {
        println!("  contract        : HOLDS");
        0
    } else {
        println!("  contract        : VIOLATED");
        println!("  residue         : consumer requires {residue:?} the producer does not define");
        println!("  note            : INVAR differential-probe finding. The full PROVEN_RESIDUE");
        println!("                    verdict additionally requires an executed composite baseline.");
        1
    };

    if let Some(path) = &args.ledger {
        let ledger = Ledger::open(path).with_context(|| format!("opening ledger {path}"))?;
        let kind = if residue.is_empty() { "contract_holds" } else { "contract_violated" };
        ledger
            .append_now(kind, "invar_check", &format!("probe={}; residue={residue:?}", hex(&probe)))
            .context("appending to ledger")?;
        let n = ledger.verify().context("verifying ledger")?;
        println!("  ledger          : {path} ({n} node(s) verified)");
    }
    Ok(code)
}

/// `invar audit`: verify a ledger and summarize its nodes. The adequacy matrix
/// is NOT emitted from the binary: its statuses are adjudicated in the doc, and
/// hardcoding them here could silently drift from reality, so we point to it.
fn cmd_audit(args: AuditArgs) -> Result<i32> {
    if !Path::new(&args.ledger).exists() {
        anyhow::bail!("ledger file does not exist: {}", args.ledger);
    }
    let ledger = Ledger::open(&args.ledger)
        .with_context(|| format!("opening ledger {}", args.ledger))?;
    let count = match ledger.verify() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ledger integrity FAILED: {e}");
            return Ok(2);
        }
    };
    let mode = ledger.journal_mode().unwrap_or_else(|_| "unknown".to_string());
    let nodes = ledger.replay_as_of(i64::MAX, i64::MAX)?;
    println!("INVAR audit");
    println!("  ledger        : {} ({count} node(s) verified, journal_mode={mode})", args.ledger);
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
    let ledger = Ledger::open(&args.ledger)
        .with_context(|| format!("opening ledger {}", args.ledger))?;
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
