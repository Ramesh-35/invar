//! Minimal hand parser for the flat `composite.toml`.
//!
//! We deliberately do NOT pull in the `toml` crate. The composite config is a
//! flat set of `[section]` headers followed by `key = value` lines, with `#`
//! comments and quoted or bare scalar values. A tiny purpose-built parser keeps
//! the dependency surface honest and the format explicit.

use crate::manifest::{CompositeConfig, ToolConfig};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read composite config {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("composite config {path} line {line}: {reason}")]
    Syntax {
        path: String,
        line: usize,
        reason: String,
    },
}

/// Parse a flat TOML-subset file into `(section -> [(key, value)])` pairs,
/// preserving order and rejecting keys that appear before any section.
fn parse_flat(path: &str, text: &str) -> Result<Vec<(String, Vec<(String, String)>)>, ConfigError> {
    let mut sections: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            // Support [[array.table]] by stripping the inner brackets too, so a
            // repeated section like [[invariant]] becomes another "invariant".
            let inner = inner.trim();
            let inner = inner
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .unwrap_or(inner);
            let name = inner.trim().to_string();
            if name.is_empty() {
                return Err(ConfigError::Syntax {
                    path: path.to_string(),
                    line: line_no,
                    reason: "empty section header".to_string(),
                });
            }
            sections.push((name, Vec::new()));
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| ConfigError::Syntax {
            path: path.to_string(),
            line: line_no,
            reason: format!("expected `key = value`, found {line:?}"),
        })?;
        let key = key.trim().to_string();
        let value = unquote(value.trim());
        let current = sections.last_mut().ok_or_else(|| ConfigError::Syntax {
            path: path.to_string(),
            line: line_no,
            reason: format!("key {key:?} appears before any [section]"),
        })?;
        current.1.push((key, value));
    }
    Ok(sections)
}

/// Public reuse of the flat-TOML-subset parser for other configs (e.g.
/// `invar.toml`), so the CLI parses config the same way, with no `toml` crate.
pub fn parse_sections(
    path: &str,
    text: &str,
) -> Result<Vec<(String, Vec<(String, String)>)>, ConfigError> {
    parse_flat(path, text)
}

/// Strip an inline `#` comment, but not one inside a quoted string.
fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_quote = !in_quote,
            '#' if !in_quote => return &line[..i],
            _ => {}
        }
    }
    line
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn tool_from(pairs: &[(String, String)]) -> ToolConfig {
    let mut cfg = ToolConfig::default();
    for (k, v) in pairs {
        match k.as_str() {
            "enabled" => cfg.enabled = v == "true",
            "bin_path" => cfg.bin_path = Some(v.clone()),
            "rules" => cfg.rules = Some(v.clone()),
            _ => {}
        }
    }
    cfg
}

impl CompositeConfig {
    /// Load a `composite.toml`. Missing sections fall back to `ToolConfig`
    /// defaults (enabled, no bin_path, no rules) — presence of a section never
    /// asserts the underlying binary exists.
    pub fn load(path: &str) -> Result<CompositeConfig, ConfigError> {
        let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_string(),
            source,
        })?;
        let sections = parse_flat(path, &raw)?;
        let mut cfg = CompositeConfig::default();
        for (name, pairs) in &sections {
            match name.as_str() {
                "semgrep" => cfg.semgrep = tool_from(pairs),
                "opa" => cfg.opa = tool_from(pairs),
                "opentelemetry" => cfg.opentelemetry = tool_from(pairs),
                "property_testing" => cfg.property_testing = tool_from(pairs),
                _ => {}
            }
        }
        Ok(cfg)
    }
}
