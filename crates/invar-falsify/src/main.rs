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
use invar_falsify::manifest::{CompositeConfig, EpistemicStatus, Manifest};
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

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Run(a) => dispatch(cmd_run(a)),
        Command::Validate(a) => dispatch(cmd_validate(a)),
        Command::Report(a) => dispatch(cmd_report(a)),
        Command::Verify(a) => dispatch(cmd_verify(a)),
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
