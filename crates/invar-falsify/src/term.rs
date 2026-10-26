//! SECURITY-SPEC S2 (SEC-D4): repository text never reaches a terminal raw.
//!
//! Two functions, one rule. A character is terminal-active when it is a
//! control character or a bidirectional override or isolate (U+202A to
//! U+202E, U+2066 to U+2069). `escape_value` turns every terminal-active
//! character of a repository value into its Rust escape (`\n`, `\r`,
//! `\u{1b}`, `\u{202e}`), so a value prints on one line and acts on
//! nothing. `neutralize` does the same at a print sink, except for newline
//! and tab, which the engine's own layout uses. Text with nothing to escape
//! is returned unchanged, byte for byte.

fn active(c: char) -> bool {
    c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn escape(s: &str, keep_layout: bool) -> String {
    if !s.chars().any(|c| active(c) && !(keep_layout && (c == '\n' || c == '\t'))) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if active(c) && !(keep_layout && (c == '\n' || c == '\t')) {
            match c {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                _ => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// A repository value for the verdict renderer: every terminal-active
/// character escaped, newline and tab included.
pub fn escape_value(s: &str) -> String {
    escape(s, false)
}

/// A whole line or block at a print sink: every terminal-active character
/// escaped except newline and tab.
pub fn neutralize(s: &str) -> String {
    escape(s, true)
}
