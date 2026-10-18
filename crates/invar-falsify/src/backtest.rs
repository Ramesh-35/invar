//! `invar backtest`: replay today's locked bindings over the producer's own
//! git history.
//!
//! The question it answers: for each field this repository's lock binds,
//! when in the producer's past was that field absent from the producer's
//! extracted surface? The evidence is the user's own commits, cited by hash
//! and date; the replay is offline, deterministic, and zero network (git is
//! invoked as a local subprocess, the only one this command runs).
//!
//! HONEST SCOPE, stated in the output coda too: timelines replay TODAY'S
//! extraction over past file contents. They are not replayed verdicts:
//! waivers, selectors and samples are today's, and a past consumer may not
//! have read the field yet. A commit whose content cannot be read or
//! extracted is stated (`unreadable` / `unextractable`), never guessed, and
//! makes the exit code 4 (answered, with stated blind spots). Remote
//! (`github:`/`gitlab:`) producers are skipped and stated: backtest replays
//! local git history only. Renames are not followed (no `--follow`) in this
//! version. Never a verdict: exits 0, 4 on blind spots, 3 on error.

use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::json;

use crate::check::source_fields;
use crate::lock::{Binding, LockedContract};
use crate::surface::name_key;
use crate::valuepairs::strings::read_any_shape;
use crate::valuepairs::ShapeRead;

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .context("running git (is git installed?)")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Is `binding` present in the extracted surface? The engine's naming-form
/// rule (case and `_`/`-` insignificant); an unscoped binding matches any
/// scope, a scoped one must match its declaring scope.
fn present(fields: &crate::surface::Fields, b: &Binding) -> bool {
    let want_field = name_key(&b.field);
    let want_scope = if b.scope.is_empty() { None } else { Some(name_key(&b.scope)) };
    fields.keys().any(|(scope, name)| {
        name_key(name) == want_field
            && want_scope.as_ref().map_or(true, |w| &name_key(scope) == w)
    })
}

#[derive(Clone, PartialEq)]
enum State {
    Present,
    Absent,
    Unreadable,
    Unextractable,
}

impl State {
    fn label(&self) -> &'static str {
        match self {
            State::Present => "present",
            State::Absent => "absent",
            State::Unreadable => "unreadable",
            State::Unextractable => "unextractable",
        }
    }
}

struct Timeline {
    binding: String,
    /// (hash, date, state): the first commit plus every state transition.
    events: Vec<(String, String, State)>,
    /// Maximal absent-runs occurring after the binding's first present
    /// commit (absence before introduction is not a window).
    absence_windows: usize,
    blind_spots: usize,
    /// Present at one replayed commit or more. Text only (R-B2-2): a
    /// binding never seen present is never reassured about absence.
    ever_present: bool,
}

struct ContractReplay {
    id: String,
    producer: String,
    commits: usize,
    timelines: Vec<Timeline>,
}

pub const BACKTEST_CODA: &str = "Timelines replay today's extraction over past file contents. They are\nnot replayed verdicts: waivers, selectors and samples are today's, and a\npast consumer may not have read the field yet. Unreadable commits are\nstated, never guessed.";

fn replay_contract(base: &Path, prefix: &str, c: &LockedContract, depth: usize) -> Result<ContractReplay> {
    let rel = format!("{prefix}{}", c.producer.trim_start_matches("./"));
    let log = git(
        base,
        &["log", "-n", &depth.to_string(), "--format=%H %cs", "--", &rel],
    )?;
    let mut commits: Vec<(String, String)> = log
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(h, d)| (h.to_string(), d.to_string()))
        .collect();
    if commits.is_empty() {
        bail!("no git history for {rel:?} (untracked, or renamed: renames are not followed)");
    }
    commits.reverse(); // oldest first
    // Extract once per commit, then read each binding off the same surface.
    // INVAR-BACKTEST-002 (B-D1, B-D2): a strings contract's producer is a
    // Zod shape, read with the very reader check uses on the live producer
    // (read_any_shape), so the two can never disagree on the same bytes.
    // The shape is the bindings' scope. NotFound is absent (the shape did
    // not exist yet, B-D3); Dynamic is a stated blind spot (B-D4).
    let strings_shape = c
        .id
        .starts_with("strings:")
        .then(|| c.bindings.first().map(|b| b.scope.clone()))
        .flatten();
    let mut surfaces = Vec::with_capacity(commits.len());
    for (hash, _) in &commits {
        let text = git(base, &["show", &format!("{hash}:{rel}")]);
        surfaces.push(match (text, &strings_shape) {
            (Err(_), _) => Err(State::Unreadable),
            (Ok(t), Some(shape)) => match read_any_shape(&t, shape) {
                ShapeRead::Found(keys) => Ok(keys.into_iter().map(|k| ((shape.clone(), k), None)).collect()),
                ShapeRead::NotFound => Ok(crate::surface::Fields::new()),
                ShapeRead::Dynamic(_) => Err(State::Unextractable),
            },
            (Ok(t), None) => match source_fields(&c.id, &c.producer, &t, c.pointer.as_deref()) {
                Ok(f) => Ok(f),
                Err(_) => Err(State::Unextractable),
            },
        });
    }
    let mut timelines = Vec::new();
    for b in &c.bindings {
        let states: Vec<State> = surfaces
            .iter()
            .map(|s| match s {
                Ok(f) => {
                    if present(f, b) {
                        State::Present
                    } else {
                        State::Absent
                    }
                }
                Err(e) => e.clone(),
            })
            .collect();
        let mut events = Vec::new();
        for (i, st) in states.iter().enumerate() {
            if i == 0 || states[i - 1] != *st {
                let (h, d) = &commits[i];
                events.push((h[..12.min(h.len())].to_string(), d.clone(), st.clone()));
            }
        }
        let first_present = states.iter().position(|s| *s == State::Present);
        let mut windows = 0usize;
        if let Some(fp) = first_present {
            let mut in_absent = false;
            for st in &states[fp..] {
                match st {
                    State::Absent if !in_absent => {
                        in_absent = true;
                        windows += 1;
                    }
                    State::Absent => {}
                    _ => in_absent = false,
                }
            }
        }
        let blind = states
            .iter()
            .filter(|s| matches!(s, State::Unreadable | State::Unextractable))
            .count();
        timelines.push(Timeline {
            binding: b.display(),
            events,
            absence_windows: windows,
            blind_spots: blind,
            ever_present: first_present.is_some(),
        });
    }
    Ok(ContractReplay {
        id: c.id.clone(),
        producer: c.producer.clone(),
        commits: commits.len(),
        timelines,
    })
}

/// Replay and render. Exit 0 (replayed), 4 (replayed with stated blind
/// spots), errors bubble to 3. Never a verdict.
pub fn run(
    config_path: &str,
    only_contract: Option<&str>,
    depth: usize,
    format: &str,
    out: &mut String,
) -> Result<i32> {
    use std::fmt::Write as _;
    if format != "text" && format != "json" {
        bail!("--format must be \"text\" or \"json\", got {format:?}");
    }
    if depth == 0 {
        bail!("--depth must be at least 1");
    }
    let config = Path::new(config_path);
    let lock = crate::mcp::load_lock(config)?
        .ok_or_else(|| anyhow!("no invar.lock beside {config_path}; run `invar lock` first"))?;
    let base = config
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let prefix = git(base, &["rev-parse", "--show-prefix"])
        .map_err(|e| anyhow!("{config_path} is not inside a git repository ({e}); backtest replays git history"))?
        .trim()
        .to_string();
    let mut replays = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    for c in &lock.contracts {
        if only_contract.is_some_and(|id| id != c.id) {
            continue;
        }
        if crate::resolver::is_remote(&c.producer) {
            skipped.push((
                c.id.clone(),
                format!("remote producer {}; backtest replays local git history only", c.producer),
            ));
            continue;
        }
        match replay_contract(base, &prefix, c, depth) {
            Ok(r) => replays.push(r),
            Err(e) => skipped.push((c.id.clone(), format!("{e:#}"))),
        }
    }
    if replays.is_empty() && skipped.is_empty() {
        bail!(match only_contract {
            Some(id) => format!("no locked contract with id {id:?}"),
            None => "no locked contracts to replay".to_string(),
        });
    }
    let blind = skipped.len()
        + replays
            .iter()
            .flat_map(|r| &r.timelines)
            .map(|t| t.blind_spots)
            .sum::<usize>();
    match format {
        "json" => {
            let v = json!({
                "backtest": replays.iter().map(|r| json!({
                    "commits_replayed": r.commits,
                    "contract": r.id,
                    "producer": r.producer,
                    "bindings": r.timelines.iter().map(|t| json!({
                        "absence_windows": t.absence_windows,
                        "binding": t.binding,
                        "blind_spot_commits": t.blind_spots,
                        "events": t.events.iter().map(|(h, d, s)| json!({
                            "commit": h, "date": d, "state": s.label(),
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "coda": BACKTEST_CODA,
                "depth": depth,
                "skipped": skipped.iter().map(|(id, why)| json!({"contract": id, "reason": why})).collect::<Vec<_>>(),
            });
            let _ = writeln!(out, "{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
        _ => {
            let _ = writeln!(
                out,
                "invar backtest ({config_path}, {} contract(s), depth {depth})",
                replays.len() + skipped.len()
            );
            // R-B2-3 (B-D6, text only): a shallow history is stated, since
            // its oldest replayed commit is a boundary, not a change. An
            // older git that cannot answer prints nothing new.
            if git(base, &["rev-parse", "--is-shallow-repository"]).is_ok_and(|s| s.trim() == "true") {
                let _ = writeln!(out, "note: shallow history; the oldest replayed commit is the shallow boundary, and its date is the cutoff, not a change");
            }
            let _ = writeln!(out);
            for r in &replays {
                let _ = writeln!(out, "{}: {} ({} commit(s) replayed)", r.id, r.producer, r.commits);
                for t in &r.timelines {
                    let _ = writeln!(out, "  {}", t.binding);
                    for (h, d, s) in &t.events {
                        let _ = writeln!(out, "    {d} {h} {}", s.label());
                    }
                    let _ = match (t.absence_windows, t.blind_spots) {
                        (_, 0) if !t.ever_present => writeln!(out, "    never present in the replayed window"),
                        (_, b) if !t.ever_present => writeln!(out, "    never present in the replayed window; {b} commit(s) not readable (stated, not guessed)"),
                        (0, 0) => writeln!(out, "    never absent after introduction"),
                        (w, 0) => writeln!(out, "    {w} absence window(s) after introduction"),
                        (w, b) => writeln!(out, "    {w} absence window(s) after introduction; {b} commit(s) not readable (stated, not guessed)"),
                    };
                }
                let _ = writeln!(out);
            }
            for (id, why) in &skipped {
                let _ = writeln!(out, "skipped {id}: {why}");
            }
            if !skipped.is_empty() {
                let _ = writeln!(out);
            }
            let _ = writeln!(out, "{BACKTEST_CODA}");
        }
    }
    Ok(if blind > 0 { 4 } else { 0 })
}
