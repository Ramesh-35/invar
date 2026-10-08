//! Minimal YAML reader for API specs (no new dependency).
//!
//! Reads the subset that OpenAPI, AsyncAPI and JSON Schema files use: block
//! mappings and sequences, flow `[...]` / `{...}`, plain and quoted scalars
//! (including multi-line), literal `|` and folded `>` block scalars, and
//! comments. Produces a `serde_json::Value`, so the rest of INVAR treats a
//! YAML spec exactly like a JSON one.
//!
//! Anything outside the subset (anchors, aliases, merge keys, tags, complex
//! keys, several documents, tab indentation) is an explicit error naming the
//! line, never a silent guess. Scalars follow YAML 1.2 core typing: `yes` and
//! `on` stay strings.

use serde_json::{Map, Number, Value};

pub fn parse_yaml(text: &str) -> Result<Value, String> {
    let mut p = Parser {
        lines: text.lines().map(|l| l.to_string()).collect(),
        i: 0,
    };
    p.skip(true)?;
    if p.i >= p.lines.len() {
        return Ok(Value::Null);
    }
    let v = p.node(0)?;
    p.skip(false)?;
    if p.i < p.lines.len() {
        return Err(p.err("unexpected content (bad indentation?)"));
    }
    Ok(v)
}

struct Parser {
    lines: Vec<String>,
    i: usize,
}

fn indent_of(l: &str) -> usize {
    l.len() - l.trim_start_matches(' ').len()
}

/// Cut a trailing comment: `#` at line start or after whitespace, outside
/// quotes and flow brackets is irrelevant here (YAML comments ignore them).
fn strip_comment(s: &str) -> &str {
    let b = s.as_bytes();
    let (mut sq, mut dq) = (false, false);
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let opens = i == 0 || matches!(b[i - 1], b' ' | b':' | b'[' | b'{' | b',' | b'-');
        match c {
            b'\'' if !dq && (sq || opens) => sq = !sq,
            b'"' if !sq && (dq || opens) => {
                if !(dq && i > 0 && b[i - 1] == b'\\') {
                    dq = !dq;
                }
            }
            b'#' if !sq && !dq && (i == 0 || b[i - 1] == b' ' || b[i - 1] == b'\t') => {
                return s[..i].trim_end();
            }
            _ => {}
        }
        i += 1;
    }
    s.trim_end()
}

/// Position of the `:` that separates a mapping key, if the content is a
/// mapping entry. A quote only opens a quoted key at the very start; inside a
/// plain key it is a literal character (real specs contain keys like
/// `DeviceIDs"`).
fn key_colon(content: &str) -> Option<usize> {
    let b = content.as_bytes();
    match b.first() {
        Some(b'[') | Some(b'{') | None => None,
        Some(&q) if q == b'"' || q == b'\'' => {
            let end = quote_end(content, q as char)?;
            let rest = &content[end + 1..];
            let off = rest.len() - rest.trim_start().len();
            let pos = end + 1 + off;
            (b.get(pos) == Some(&b':') && (pos + 1 == b.len() || b[pos + 1] == b' ')).then_some(pos)
        }
        _ => (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' ')),
    }
}

fn is_seq_item(content: &str) -> bool {
    content == "-" || content.starts_with("- ")
}

impl Parser {
    fn err(&self, msg: &str) -> String {
        format!("YAML line {}: {msg}", self.i + 1)
    }

    fn unsupported(&self, what: &str) -> String {
        self.err(&format!("unsupported YAML feature: {what}"))
    }

    /// Skip blank and comment lines; allow one leading `---`.
    fn skip(&mut self, at_start: bool) -> Result<(), String> {
        let mut seen_doc = !at_start;
        while self.i < self.lines.len() {
            let l = &self.lines[self.i];
            if l.contains('\t') && l.trim_start_matches(' ').starts_with('\t') {
                return Err(self.unsupported("tab indentation"));
            }
            let t = l.trim();
            if t.is_empty() || t.starts_with('#') {
                self.i += 1;
            } else if t == "---" || t.starts_with("--- ") {
                if seen_doc {
                    return Err(self.unsupported("multiple documents"));
                }
                seen_doc = true;
                self.i += 1;
            } else if t == "..." {
                self.i = self.lines.len();
            } else {
                return Ok(());
            }
        }
        Ok(())
    }

    fn content(&self) -> String {
        strip_comment(self.lines[self.i].trim_start()).to_string()
    }

    fn node(&mut self, min_indent: usize) -> Result<Value, String> {
        self.skip(false)?;
        if self.i >= self.lines.len() {
            return Ok(Value::Null);
        }
        let ind = indent_of(&self.lines[self.i]);
        if ind < min_indent {
            return Ok(Value::Null);
        }
        let c = self.content();
        if is_seq_item(&c) {
            self.seq(ind)
        } else if key_colon(&c).is_some() {
            self.map(ind)
        } else {
            self.i += 1;
            self.inline(&c, ind.saturating_sub(1))
        }
    }

    fn map(&mut self, ind: usize) -> Result<Value, String> {
        let mut m = Map::new();
        loop {
            self.skip(false)?;
            if self.i >= self.lines.len() {
                break;
            }
            let li = indent_of(&self.lines[self.i]);
            if li < ind {
                break;
            }
            if li > ind {
                return Err(self.err("unexpected indentation inside a mapping"));
            }
            let c = self.content();
            if is_seq_item(&c) {
                break;
            }
            if c.starts_with("? ") || c == "?" {
                return Err(self.unsupported("complex mapping keys"));
            }
            let pos = key_colon(&c).ok_or_else(|| self.err("expected `key: value`"))?;
            let raw_key = c[..pos].trim();
            if raw_key == "<<" {
                return Err(self.unsupported("merge keys (<<)"));
            }
            let key = scalar_key(raw_key).map_err(|e| self.err(&e))?;
            let rest = c[pos + 1..].trim().to_string();
            self.i += 1;
            let v = if rest.is_empty() {
                self.skip(false)?;
                if self.i < self.lines.len() {
                    let ni = indent_of(&self.lines[self.i]);
                    let nc = self.content();
                    if ni > ind {
                        self.node(ni)?
                    } else if ni == ind && is_seq_item(&nc) {
                        self.seq(ind)?
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            } else {
                self.inline(&rest, ind)?
            };
            m.insert(key, v);
        }
        Ok(Value::Object(m))
    }

    fn seq(&mut self, ind: usize) -> Result<Value, String> {
        let mut out = Vec::new();
        loop {
            self.skip(false)?;
            if self.i >= self.lines.len() {
                break;
            }
            let li = indent_of(&self.lines[self.i]);
            let c = self.content();
            if li != ind || !is_seq_item(&c) {
                if li > ind {
                    return Err(self.err("unexpected indentation inside a sequence"));
                }
                break;
            }
            let item = c[1..].trim_start().to_string();
            if item.is_empty() {
                self.i += 1;
                self.skip(false)?;
                let v = if self.i < self.lines.len() && indent_of(&self.lines[self.i]) > ind {
                    let ni = indent_of(&self.lines[self.i]);
                    self.node(ni)?
                } else {
                    Value::Null
                };
                out.push(v);
                continue;
            }
            // Column where the item's own content starts.
            let line = self.lines[self.i].clone();
            let after_dash = &line[ind + 1..];
            let col = ind + 1 + (after_dash.len() - after_dash.trim_start().len());
            if is_seq_item(&item) || key_colon(&item).is_some() {
                // Re-read the item as a nested node starting at its column.
                self.lines[self.i] = format!("{}{}", " ".repeat(col), &line[col..]);
                out.push(self.node(col)?);
            } else {
                self.i += 1;
                out.push(self.inline(&item, ind)?);
            }
        }
        Ok(Value::Array(out))
    }

    /// A value written after `key:` or `- `. `parent` is the indentation of
    /// the owning entry; continuation lines must be indented further.
    fn inline(&mut self, rest: &str, parent: usize) -> Result<Value, String> {
        let first = rest.chars().next().unwrap_or(' ');
        match first {
            '&' | '*' => return Err(self.unsupported("anchors and aliases")),
            '!' => return Err(self.unsupported("tags")),
            '|' | '>' => return self.block_scalar(rest, parent),
            _ => {}
        }
        if first == '[' || first == '{' {
            let mut text = rest.to_string();
            while !flow_balanced(&text) {
                if self.i >= self.lines.len() {
                    return Err(self.err("unterminated flow collection"));
                }
                text.push(' ');
                text.push_str(strip_comment(self.lines[self.i].trim()));
                self.i += 1;
            }
            let mut f = Flow {
                s: text.as_bytes(),
                i: 0,
            };
            let v = f.value().map_err(|e| self.err(&e))?;
            f.ws();
            if f.i < f.s.len() {
                return Err(self.err("trailing characters after flow collection"));
            }
            return Ok(v);
        }
        if first == '"' || first == '\'' {
            let mut text = rest.to_string();
            while !quote_closed(&text, first) {
                if self.i >= self.lines.len() {
                    return Err(self.err("unterminated quoted string"));
                }
                let next = self.lines[self.i].trim();
                text.push(' ');
                text.push_str(next);
                self.i += 1;
            }
            let end = quote_end(&text, first).unwrap();
            let tail = strip_comment(text[end + 1..].trim());
            if !tail.is_empty() {
                return Err(self.err("unexpected text after quoted string"));
            }
            return Ok(Value::String(unquote(&text[..=end], first)));
        }
        // Plain scalar, possibly continued on more-indented lines.
        let mut text = rest.to_string();
        while self.i < self.lines.len() {
            let l = &self.lines[self.i];
            let t = l.trim();
            if t.is_empty() || t.starts_with('#') {
                self.i += 1;
                continue;
            }
            if indent_of(l) <= parent {
                break;
            }
            text.push(' ');
            text.push_str(strip_comment(t));
            self.i += 1;
        }
        Ok(typed_scalar(&text))
    }

    fn block_scalar(&mut self, header: &str, parent: usize) -> Result<Value, String> {
        let folded = header.starts_with('>');
        let chomp = if header.contains('-') {
            '-'
        } else if header.contains('+') {
            '+'
        } else {
            'c'
        };
        let mut raw: Vec<String> = Vec::new();
        let mut content_indent: Option<usize> = None;
        while self.i < self.lines.len() {
            let l = &self.lines[self.i];
            if l.trim().is_empty() {
                raw.push(String::new());
                self.i += 1;
                continue;
            }
            let li = indent_of(l);
            if li <= parent {
                break;
            }
            let ci = *content_indent.get_or_insert(li);
            if li < ci {
                break;
            }
            raw.push(l[ci..].to_string());
            self.i += 1;
        }
        // Trailing blank lines belong to chomping, not content.
        let mut trailing = 0;
        while raw.last().map(|s| s.is_empty()).unwrap_or(false) {
            raw.pop();
            trailing += 1;
        }
        let mut s = if folded {
            let mut out = String::new();
            for (k, line) in raw.iter().enumerate() {
                if k > 0 {
                    let prev = &raw[k - 1];
                    if line.is_empty()
                        || prev.is_empty()
                        || line.starts_with(' ')
                        || prev.starts_with(' ')
                    {
                        out.push('\n');
                    } else {
                        out.push(' ');
                    }
                }
                out.push_str(line);
            }
            out
        } else {
            raw.join("\n")
        };
        if !raw.is_empty() {
            match chomp {
                '-' => {}
                '+' => s.push_str(&"\n".repeat(trailing + 1)),
                _ => s.push('\n'),
            }
        }
        Ok(Value::String(s))
    }
}

fn quote_end(s: &str, q: char) -> Option<usize> {
    let b = s.as_bytes();
    let q = q as u8;
    let mut i = 1;
    while i < b.len() {
        if b[i] == q {
            if q == b'\'' && i + 1 < b.len() && b[i + 1] == b'\'' {
                i += 2;
                continue;
            }
            if q == b'"' && b[i - 1] == b'\\' {
                // count preceding backslashes
                let mut n = 0;
                let mut j = i;
                while j > 0 && b[j - 1] == b'\\' {
                    n += 1;
                    j -= 1;
                }
                if n % 2 == 1 {
                    i += 1;
                    continue;
                }
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

fn quote_closed(s: &str, q: char) -> bool {
    quote_end(s, q).is_some()
}

fn unquote(s: &str, q: char) -> String {
    let inner = &s[1..s.len() - 1];
    if q == '\'' {
        return inner.replace("''", "'");
    }
    let mut out = String::new();
    let mut it = inner.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn scalar_key(raw: &str) -> Result<String, String> {
    let c = raw.chars().next().unwrap_or(' ');
    if c == '&' || c == '*' {
        return Err("unsupported YAML feature: anchors and aliases".into());
    }
    if c == '!' {
        return Err("unsupported YAML feature: tags".into());
    }
    if (c == '"' || c == '\'') && raw.len() >= 2 {
        return Ok(unquote(raw, c));
    }
    Ok(raw.to_string())
}

fn typed_scalar(s: &str) -> Value {
    let t = s.trim();
    match t {
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        "true" | "True" | "TRUE" => return Value::Bool(true),
        "false" | "False" | "FALSE" => return Value::Bool(false),
        _ => {}
    }
    let digits = t
        .strip_prefix('-')
        .or_else(|| t.strip_prefix('+'))
        .unwrap_or(t);
    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        if let Ok(n) = t.parse::<i64>() {
            return Value::Number(n.into());
        }
    }
    let looks_float = !digits.is_empty()
        && digits.bytes().all(|b| {
            b.is_ascii_digit() || b == b'.' || b == b'e' || b == b'E' || b == b'-' || b == b'+'
        })
        && digits
            .bytes()
            .next()
            .map(|b| b.is_ascii_digit() || b == b'.')
            .unwrap_or(false)
        && digits.contains('.');
    if looks_float {
        if let Ok(f) = t.parse::<f64>() {
            if let Some(n) = Number::from_f64(f) {
                return Value::Number(n);
            }
        }
    }
    Value::String(t.to_string())
}

fn flow_balanced(s: &str) -> bool {
    let b = s.as_bytes();
    let (mut sq, mut dq, mut depth) = (false, false, 0i32);
    for i in 0..b.len() {
        match b[i] {
            b'\'' if !dq => sq = !sq,
            b'"' if !sq && !(dq && i > 0 && b[i - 1] == b'\\') => dq = !dq,
            b'[' | b'{' if !sq && !dq => depth += 1,
            b']' | b'}' if !sq && !dq => depth -= 1,
            _ => {}
        }
    }
    depth <= 0 && !sq && !dq
}

struct Flow<'a> {
    s: &'a [u8],
    i: usize,
}

impl Flow<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] == b' ' || self.s[self.i] == b'\n') {
            self.i += 1;
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.ws();
        match self.s.get(self.i) {
            Some(b'[') => {
                self.i += 1;
                let mut v = Vec::new();
                loop {
                    self.ws();
                    if self.s.get(self.i) == Some(&b']') {
                        self.i += 1;
                        return Ok(Value::Array(v));
                    }
                    let item = self.value()?;
                    self.ws();
                    // `[key: value]` is a single-pair mapping inside a sequence.
                    if self.s.get(self.i) == Some(&b':')
                        && !matches!(item, Value::Array(_) | Value::Object(_))
                    {
                        self.i += 1;
                        let k = match item {
                            Value::String(s) => s,
                            other => other.to_string(),
                        };
                        let mut m = Map::new();
                        m.insert(k, self.value()?);
                        v.push(Value::Object(m));
                        self.ws();
                    } else {
                        v.push(item);
                    }
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {}
                        _ => return Err("expected `,` or `]` in flow sequence".into()),
                    }
                }
            }
            Some(b'{') => {
                self.i += 1;
                let mut m = Map::new();
                loop {
                    self.ws();
                    if self.s.get(self.i) == Some(&b'}') {
                        self.i += 1;
                        return Ok(Value::Object(m));
                    }
                    let k = match self.scalar(true)? {
                        Value::String(s) => s,
                        other => other.to_string(),
                    };
                    self.ws();
                    let v = if self.s.get(self.i) == Some(&b':') {
                        self.i += 1;
                        self.value()?
                    } else {
                        Value::Null
                    };
                    m.insert(k, v);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {}
                        _ => return Err("expected `,` or `}` in flow mapping".into()),
                    }
                }
            }
            _ => self.scalar(false),
        }
    }

    fn scalar(&mut self, is_key: bool) -> Result<Value, String> {
        self.ws();
        let start = self.i;
        match self.s.get(self.i) {
            Some(&q) if q == b'"' || q == b'\'' => {
                let rest = std::str::from_utf8(&self.s[start..]).map_err(|e| e.to_string())?;
                let end = quote_end(rest, q as char).ok_or("unterminated quoted string")?;
                self.i += end + 1;
                Ok(Value::String(unquote(&rest[..=end], q as char)))
            }
            Some(b'&') | Some(b'*') => Err("unsupported YAML feature: anchors and aliases".into()),
            Some(b'!') => Err("unsupported YAML feature: tags".into()),
            _ => {
                while self.i < self.s.len() {
                    let c = self.s[self.i];
                    if c == b',' || c == b']' || c == b'}' {
                        break;
                    }
                    if c == b':'
                        && (is_key || self.s.get(self.i + 1).map(|&n| n == b' ').unwrap_or(true))
                    {
                        break;
                    }
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?;
                Ok(if is_key {
                    Value::String(t.trim().to_string())
                } else {
                    typed_scalar(t)
                })
            }
        }
    }
}
