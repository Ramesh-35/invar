//! Provenance independence model (GAP-B).
//!
//! The danger this closes: two pieces of evidence that descend from ONE
//! originating observation must not be counted as independent corroboration.
//! This module records each source's ancestry and decides, deterministically,
//! whether two sources are independent (disjoint lineage) and whether a set of
//! sources rises to `Confident` corroboration or is capped at `Banded`.
//!
//! Honesty rules enforced here:
//!   * Independence is decided only from recorded ancestry, never assumed. If a
//!     source is unknown to the graph the verdict is `Unknown`, never
//!     `Independent`.
//!   * Correlation is closed transitively and conservatively: sources linked
//!     through a shared ancestor are merged into one independent group. This can
//!     only UNDER-count independence, never inflate it, which is the safe
//!     direction for a skeptic engine.
//!
//! Scope (stated, not hidden): this is a standalone, tested capability. It is
//! deliberately not yet invoked by `evaluator::classify`, because the Phase-1
//! composite legs are distinct tools, not multiple carriers of one observation,
//! so there is no correlated evidence to de-duplicate in the live path yet.

use std::collections::{BTreeSet, HashMap};

use crate::manifest::EvidenceTier;

/// One evidence source and its DIRECT ancestors (by id). The transitive
/// ancestor closure is computed by the graph, not stored here.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvenanceSource {
    pub source_id: String,
    pub ancestor_ids: Vec<String>,
    pub trust_tier: EvidenceTier,
}

/// Independence verdict between two sources. Three states, because "cannot
/// tell" is not "independent".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Independence {
    Independent,
    Correlated,
    Unknown,
}

/// Corroboration standing of a SET of sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corroboration {
    /// No admissible (known) source.
    Unsupported,
    /// Exactly one independent group of sources. Adding more carriers from the
    /// same lineage cannot promote it.
    Banded,
    /// Two or more mutually independent groups corroborate.
    Confident,
}

/// A directed ancestry graph over named sources.
#[derive(Debug, Default)]
pub struct ProvenanceGraph {
    sources: HashMap<String, ProvenanceSource>,
}

impl ProvenanceGraph {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn add_source(&mut self, source: ProvenanceSource) {
        self.sources.insert(source.source_id.clone(), source);
    }

    /// Transitive ancestor closure of `id`, INCLUDING `id` itself. An unknown
    /// node contributes itself but no further ancestry, which can never
    /// manufacture independence.
    fn closure(&self, id: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![id.to_string()];
        while let Some(cur) = stack.pop() {
            if !seen.insert(cur.clone()) {
                continue;
            }
            if let Some(src) = self.sources.get(&cur) {
                for a in &src.ancestor_ids {
                    if !seen.contains(a) {
                        stack.push(a.clone());
                    }
                }
            }
        }
        seen
    }

    /// Independence of two sources, decided only from recorded ancestry.
    pub fn independence(&self, s1: &str, s2: &str) -> Independence {
        if !self.sources.contains_key(s1) || !self.sources.contains_key(s2) {
            return Independence::Unknown;
        }
        if s1 == s2 {
            return Independence::Correlated;
        }
        let c1 = self.closure(s1);
        let c2 = self.closure(s2);
        if c1.is_disjoint(&c2) {
            Independence::Independent
        } else {
            Independence::Correlated
        }
    }

    /// Conservative boolean independence: TRUE only when provably independent.
    /// `Unknown` and `Correlated` both yield false, because a skeptic may not
    /// treat unverifiable independence as independence.
    pub fn are_independent(&self, s1: &str, s2: &str) -> bool {
        matches!(self.independence(s1, s2), Independence::Independent)
    }

    /// Corroboration standing of a set of sources. Sources that are not provably
    /// independent (correlated through shared ancestry) are merged into one
    /// group; promotion to `Confident` requires two or more mutually
    /// independent groups. Unknown ids are not admissible and are ignored.
    pub fn corroboration(&self, ids: &[&str]) -> Corroboration {
        let known: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|i| self.sources.contains_key(*i))
            .collect();
        if known.is_empty() {
            return Corroboration::Unsupported;
        }

        // Union-find over "not provably independent" pairs (conservative merge).
        let n = known.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut [usize], x: usize) -> usize {
            let mut r = x;
            while p[r] != r {
                r = p[r];
            }
            let mut c = x;
            while p[c] != c {
                let nx = p[c];
                p[c] = r;
                c = nx;
            }
            r
        }
        for i in 0..n {
            for j in (i + 1)..n {
                if !matches!(self.independence(known[i], known[j]), Independence::Independent) {
                    let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
            }
        }
        let mut roots = BTreeSet::new();
        for i in 0..n {
            let r = find(&mut parent, i);
            roots.insert(r);
        }
        if roots.len() >= 2 {
            Corroboration::Confident
        } else {
            Corroboration::Banded
        }
    }
}
