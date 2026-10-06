//! Spec resolver for `invar check` (local paths and in-memory remote refs).
//!
//! A spec reference in `invar.toml` is either:
//!   * a local path, resolved against the directory that holds `invar.toml`; or
//!   * a remote reference `github:OWNER/REPO@REF:PATH`, fetched into memory,
//!     hashed with BLAKE3, and never written to disk. `REF` may be `{base}`,
//!     which expands to `GITHUB_BASE_REF` in CI.
//!
//! Remote references are only honored when the caller allows them (`--ci`).
//! The network itself sits behind the `Transport` trait so the resolution and
//! evaluation path is deterministic and testable without a network.
//!
//! HONEST SCOPE: this module builds the request (URL + headers) and verifies
//! and hashes whatever bytes come back. It does not itself open sockets; a
//! live `Transport` is a separate decision (see docs/epistemic/STATUS.md).
//! GitLab references are recognized and refused as not yet supported.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRef {
    pub owner: String,
    pub repo: String,
    pub git_ref: String,
    pub path: String,
}

impl RemoteRef {
    pub fn canonical(&self) -> String {
        format!("github:{}/{}@{}:{}", self.owner, self.repo, self.git_ref, self.path)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ResolveError {
    #[error("bad remote reference {0:?} (expected github:OWNER/REPO@REF:PATH)")]
    BadReference(String),
    #[error("remote reference {0:?} requires --ci")]
    RemoteRequiresCi(String),
    #[error("{0:?} uses {{base}} but GITHUB_BASE_REF is not set")]
    MissingBaseRef(String),
    #[error("{0:?}: this build has no live network transport (decision pending, see docs/epistemic/STATUS.md)")]
    NoTransport(String),
    #[error("fetching {reference}: {reason}")]
    Fetch { reference: String, reason: String },
    #[error("reading {path}: {reason}")]
    Io { path: String, reason: String },
    #[error("{0} is not valid UTF-8")]
    NotUtf8(String),
    #[error("{0:?}: only github: remote references are supported so far")]
    Unsupported(String),
}

fn ok_name(s: &str) -> bool {
    !s.is_empty()
        && s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
}

fn ok_path_like(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('/')
        && s.split('/').all(|seg| !seg.is_empty() && seg != "..")
        && s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | '/'))
}

/// Is `spec` a remote reference (as opposed to a local path)?
pub fn is_remote(spec: &str) -> bool {
    spec.starts_with("github:") || spec.starts_with("gitlab:")
}

/// Parse `github:OWNER/REPO@REF:PATH`, expanding `{base}` from `base_ref`.
pub fn parse_ref(spec: &str, base_ref: Option<&str>) -> Result<RemoteRef, ResolveError> {
    if spec.starts_with("gitlab:") {
        return Err(ResolveError::Unsupported(spec.to_string()));
    }
    let bad = || ResolveError::BadReference(spec.to_string());
    let rest = spec.strip_prefix("github:").ok_or_else(bad)?;
    let (owner_repo, ref_path) = rest.split_once('@').ok_or_else(bad)?;
    let (owner, repo) = owner_repo.split_once('/').ok_or_else(bad)?;
    let (raw_ref, path) = ref_path.split_once(':').ok_or_else(bad)?;
    let git_ref = if raw_ref == "{base}" {
        base_ref
            .filter(|b| !b.is_empty())
            .ok_or_else(|| ResolveError::MissingBaseRef(spec.to_string()))?
            .to_string()
    } else {
        raw_ref.to_string()
    };
    if !ok_name(owner) || !ok_name(repo) || !ok_path_like(&git_ref) || !ok_path_like(path) {
        return Err(bad());
    }
    Ok(RemoteRef {
        owner: owner.to_string(),
        repo: repo.to_string(),
        git_ref,
        path: path.to_string(),
    })
}

/// A fully built HTTP GET. Headers may carry a token, so this type has no
/// Display and callers must never print `headers`.
#[derive(Clone, PartialEq, Eq)]
pub struct FetchRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl std::fmt::Debug for FetchRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchRequest")
            .field("url", &self.url)
            .field("headers", &format!("<{} redacted>", self.headers.len()))
            .finish()
    }
}

/// Build the GitHub request. Without a token: the public raw endpoint. With a
/// token: the contents API in raw mode, which also serves private repos.
pub fn github_request(r: &RemoteRef, token: Option<&str>) -> FetchRequest {
    match token.filter(|t| !t.is_empty()) {
        None => FetchRequest {
            url: format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                r.owner, r.repo, r.git_ref, r.path
            ),
            headers: vec![("User-Agent".into(), "invar".into())],
        },
        Some(t) => FetchRequest {
            url: format!(
                "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                r.owner, r.repo, r.path, r.git_ref
            ),
            headers: vec![
                ("Accept".into(), "application/vnd.github.raw".into()),
                ("Authorization".into(), format!("Bearer {t}")),
                ("X-GitHub-Api-Version".into(), "2022-11-28".into()),
                ("User-Agent".into(), "invar".into()),
            ],
        },
    }
}

/// The network seam. Implementations return the response body or a reason.
pub trait Transport {
    fn get(&self, req: &FetchRequest) -> Result<Vec<u8>, String>;
}

/// Deterministic transport for tests: URL -> body. Unknown URLs are 404.
#[derive(Default)]
pub struct MockTransport {
    pub bodies: HashMap<String, Vec<u8>>,
}

impl MockTransport {
    pub fn with(mut self, url: &str, body: &str) -> Self {
        self.bodies.insert(url.to_string(), body.as_bytes().to_vec());
        self
    }
}

impl Transport for MockTransport {
    fn get(&self, req: &FetchRequest) -> Result<Vec<u8>, String> {
        self.bodies
            .get(&req.url)
            .cloned()
            .ok_or_else(|| "HTTP 404".to_string())
    }
}

/// Provenance of one remote artifact fetched into memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub reference: String,
    pub url: String,
    pub bytes: usize,
    pub blake3_hex: String,
}

pub struct Resolved {
    pub text: String,
    pub provenance: Option<Fetched>,
}

pub struct Resolver<'a> {
    /// Directory local relative paths are resolved against (the config's dir).
    pub base_dir: PathBuf,
    /// Honor remote references (set by `--ci`).
    pub allow_remote: bool,
    pub token: Option<String>,
    pub base_ref: Option<String>,
    pub transport: Option<&'a dyn Transport>,
}

impl<'a> Resolver<'a> {
    pub fn local(base_dir: &Path) -> Resolver<'a> {
        Resolver {
            base_dir: base_dir.to_path_buf(),
            allow_remote: false,
            token: None,
            base_ref: None,
            transport: None,
        }
    }

    pub fn resolve(&self, spec: &str) -> Result<Resolved, ResolveError> {
        if !is_remote(spec) {
            let p = Path::new(spec);
            let full = if p.is_absolute() { p.to_path_buf() } else { self.base_dir.join(p) };
            let text = std::fs::read_to_string(&full).map_err(|e| ResolveError::Io {
                path: full.display().to_string(),
                reason: e.to_string(),
            })?;
            return Ok(Resolved { text, provenance: None });
        }
        if !self.allow_remote {
            return Err(ResolveError::RemoteRequiresCi(spec.to_string()));
        }
        let r = parse_ref(spec, self.base_ref.as_deref())?;
        let reference = r.canonical();
        let transport = self
            .transport
            .ok_or_else(|| ResolveError::NoTransport(reference.clone()))?;
        let req = github_request(&r, self.token.as_deref());
        let bytes = transport.get(&req).map_err(|reason| ResolveError::Fetch {
            reference: reference.clone(),
            reason,
        })?;
        let blake3_hex = blake3::hash(&bytes).to_hex().to_string();
        let n = bytes.len();
        let text = String::from_utf8(bytes).map_err(|_| ResolveError::NotUtf8(reference.clone()))?;
        Ok(Resolved {
            text,
            provenance: Some(Fetched {
                reference,
                url: req.url,
                bytes: n,
                blake3_hex,
            }),
        })
    }
}
