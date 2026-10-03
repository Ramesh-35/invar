//! Bi-temporal, BLAKE3-chained lineage ledger over SQLite (WAL).
//!
//! Two clocks are kept *distinct* and never collapsed:
//!   * `valid_time_us`  (t_v) — when the fact was true in the modelled world.
//!   * `transaction_time_us` (t_r) — when we recorded it.
//!
//! Integrity is a hash chain: `state_hash = BLAKE3(content)` and
//! `lineage_hash = BLAKE3(prev_lineage ++ state_hash ++ context)` where
//! `context = "{kind}|{subject}|{t_v}|{t_r}"`. `verify()` re-derives BOTH hashes
//! by recomputation while re-walking the rows in id order — it never trusts a
//! stored "valid" flag.

use rusqlite::{params, Connection, OptionalExtension};

/// A single committed lineage row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageNode {
    pub id: i64,
    pub valid_time_us: i64,
    pub transaction_time_us: i64,
    pub kind: String,
    pub subject: String,
    pub content: String,
    pub state_hash: [u8; 32],
    pub lineage_hash: [u8; 32],
}

#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("ledger state hash mismatch at row id {id}: content does not match stored state_hash")]
    StateMismatch { id: i64 },
    #[error(
        "ledger lineage broken at row id {id}: recomputed chain hash does not match stored lineage_hash"
    )]
    LineageBroken { id: i64 },
    #[error("corrupt hash blob at row id {id}: expected 32 bytes, found {len}")]
    CorruptHash { id: i64, len: usize },
}

fn state_hash(content: &str) -> [u8; 32] {
    *blake3::hash(content.as_bytes()).as_bytes()
}

fn lineage_hash(prev: &[u8; 32], state: &[u8; 32], context: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prev);
    hasher.update(state);
    hasher.update(context.as_bytes());
    *hasher.finalize().as_bytes()
}

fn to_hash(id: i64, bytes: Vec<u8>) -> Result<[u8; 32], LedgerError> {
    if bytes.len() != 32 {
        return Err(LedgerError::CorruptHash {
            id,
            len: bytes.len(),
        });
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn now_us() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

pub struct Ledger {
    conn: Connection,
}

impl Ledger {
    /// Open (or create) a ledger at `path`, enabling WAL. WAL requires a
    /// file-backed database.
    pub fn open(path: &str) -> Result<Ledger, LedgerError> {
        let conn = Connection::open(path)?;
        // journal_mode returns a row, so it must be queried, not batch-executed.
        let _mode: String =
            conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        conn.execute_batch("PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS lineage (
                id                   INTEGER PRIMARY KEY AUTOINCREMENT,
                valid_time_us        INTEGER NOT NULL,
                transaction_time_us  INTEGER NOT NULL,
                kind                 TEXT    NOT NULL,
                subject              TEXT    NOT NULL,
                content              TEXT    NOT NULL,
                state_hash           BLOB    NOT NULL,
                lineage_hash         BLOB    NOT NULL
            );",
        )?;
        Ok(Ledger { conn })
    }

    /// The single private write path. Both public appenders route through here
    /// so the chaining rule has exactly one implementation.
    fn commit(
        &self,
        valid_time_us: i64,
        transaction_time_us: i64,
        kind: &str,
        subject: &str,
        content: &str,
    ) -> Result<LineageNode, LedgerError> {
        let prev = self.last_lineage_hash()?;
        let sh = state_hash(content);
        let context = format!("{kind}|{subject}|{valid_time_us}|{transaction_time_us}");
        let lh = lineage_hash(&prev, &sh, &context);
        self.conn.execute(
            "INSERT INTO lineage
                (valid_time_us, transaction_time_us, kind, subject, content, state_hash, lineage_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                valid_time_us,
                transaction_time_us,
                kind,
                subject,
                content,
                &sh[..],
                &lh[..]
            ],
        )?;
        Ok(LineageNode {
            id: self.conn.last_insert_rowid(),
            valid_time_us,
            transaction_time_us,
            kind: kind.to_string(),
            subject: subject.to_string(),
            content: content.to_string(),
            state_hash: sh,
            lineage_hash: lh,
        })
    }

    /// Append with an explicit valid time (t_v) distinct from the record time.
    pub fn append(
        &self,
        valid_time_us: i64,
        kind: &str,
        subject: &str,
        content: &str,
    ) -> Result<LineageNode, LedgerError> {
        self.commit(valid_time_us, now_us(), kind, subject, content)
    }

    /// Append a fact whose event *is* this run: t_v == t_r. Honest for a
    /// fixture verdict produced at record time.
    pub fn append_now(
        &self,
        kind: &str,
        subject: &str,
        content: &str,
    ) -> Result<LineageNode, LedgerError> {
        let now = now_us();
        self.commit(now, now, kind, subject, content)
    }

    /// Append with BOTH clocks explicit. This is the honest primitive for
    /// recording a fact whose knowledge time (t_r) is not "now" (a historical
    /// import, or a controlled reconstruction fixture). The recorded t_r is
    /// bound into `lineage_hash` via the context string, so a t_r written here
    /// cannot later be altered without `verify()` detecting it: an explicit t_r
    /// is a recorded claim, not a trusted-forever flag.
    pub fn append_at(
        &self,
        valid_time_us: i64,
        transaction_time_us: i64,
        kind: &str,
        subject: &str,
        content: &str,
    ) -> Result<LineageNode, LedgerError> {
        self.commit(valid_time_us, transaction_time_us, kind, subject, content)
    }

    fn last_lineage_hash(&self) -> Result<[u8; 32], LedgerError> {
        let row: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT lineage_hash FROM lineage ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        match row {
            Some(bytes) => to_hash(-1, bytes),
            None => Ok([0u8; 32]),
        }
    }

    /// Re-walk the whole chain in id order, recomputing both hashes. Returns the
    /// verified node count, or the precise row at which integrity broke.
    pub fn verify(&self) -> Result<usize, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, valid_time_us, transaction_time_us, kind, subject, content, state_hash, lineage_hash
             FROM lineage ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Vec<u8>>(6)?,
                r.get::<_, Vec<u8>>(7)?,
            ))
        })?;

        let mut prev = [0u8; 32];
        let mut count = 0usize;
        for row in rows {
            let (id, tv, tr, kind, subject, content, stored_state, stored_lin) = row?;
            let stored_state = to_hash(id, stored_state)?;
            let stored_lin = to_hash(id, stored_lin)?;

            let recomputed_state = state_hash(&content);
            if recomputed_state != stored_state {
                return Err(LedgerError::StateMismatch { id });
            }
            let context = format!("{kind}|{subject}|{tv}|{tr}");
            let recomputed_lin = lineage_hash(&prev, &recomputed_state, &context);
            if recomputed_lin != stored_lin {
                return Err(LedgerError::LineageBroken { id });
            }
            prev = stored_lin;
            count += 1;
        }
        Ok(count)
    }

    /// Bi-temporal as-of reconstruction (temporal admissibility gate).
    ///
    /// Re-walks and verifies the FULL chain first, exactly as `verify()` does,
    /// so tampering anywhere is still caught, then returns only the nodes that
    /// were admissible as known at `as_of_t_r` and true at or before
    /// `as_of_t_v`. A row is admitted only when BOTH hold:
    ///
    ///   * `transaction_time_us <= as_of_t_r`  (it had been recorded by then)
    ///   * `valid_time_us        <= as_of_t_v`  (it was true in the world by then)
    ///
    /// A fact recorded after `as_of_t_r` is therefore invisible to this
    /// reconstruction. That is the temporal-admissibility guarantee: later
    /// knowledge cannot contaminate an earlier judgment. The chain is verified
    /// over all rows (not just the admissible prefix) because `lineage_hash`
    /// binds every row to its predecessor, so integrity is a property of the
    /// whole log, not of a slice of it.
    pub fn replay_as_of(
        &self,
        as_of_t_r: i64,
        as_of_t_v: i64,
    ) -> Result<Vec<LineageNode>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, valid_time_us, transaction_time_us, kind, subject, content, state_hash, lineage_hash
             FROM lineage ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Vec<u8>>(6)?,
                r.get::<_, Vec<u8>>(7)?,
            ))
        })?;

        let mut prev = [0u8; 32];
        let mut admissible: Vec<LineageNode> = Vec::new();
        for row in rows {
            let (id, tv, tr, kind, subject, content, stored_state, stored_lin) = row?;
            let stored_state = to_hash(id, stored_state)?;
            let stored_lin = to_hash(id, stored_lin)?;

            let recomputed_state = state_hash(&content);
            if recomputed_state != stored_state {
                return Err(LedgerError::StateMismatch { id });
            }
            let context = format!("{kind}|{subject}|{tv}|{tr}");
            let recomputed_lin = lineage_hash(&prev, &recomputed_state, &context);
            if recomputed_lin != stored_lin {
                return Err(LedgerError::LineageBroken { id });
            }
            prev = stored_lin;

            // Admissibility filter applied only AFTER integrity is confirmed.
            if tr <= as_of_t_r && tv <= as_of_t_v {
                admissible.push(LineageNode {
                    id,
                    valid_time_us: tv,
                    transaction_time_us: tr,
                    kind,
                    subject,
                    content,
                    state_hash: recomputed_state,
                    lineage_hash: stored_lin,
                });
            }
        }
        Ok(admissible)
    }

    /// The on-disk journal mode (used to prove WAL is active).
    pub fn journal_mode(&self) -> Result<String, LedgerError> {
        let mode: String = self.conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
        Ok(mode)
    }
}
