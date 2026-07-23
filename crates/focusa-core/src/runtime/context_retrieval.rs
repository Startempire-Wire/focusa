//! Derived Context retrieval index for Spec 135 C2.
//!
//! Canonical Context records remain reducer/event state. This module maintains a
//! restart-safe, rebuildable SQLite FTS5 + sqlite-vec projection and performs
//! deterministic, exact-scope bounded retrieval with source-preserving citations.

use crate::runtime::persistence_sqlite::SqlitePersistence;
use crate::types::ContextSourceRecord;
use anyhow::{Context, anyhow};
#[cfg(feature = "context-vector-fastembed")]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Once;
#[cfg(feature = "context-vector-fastembed")]
use std::sync::{Mutex, OnceLock};

const EMBEDDING_DIMENSIONS: usize = 384;
const EMBEDDING_MODEL: &str = "fastembed/all-MiniLM-L6-v2";
const MAX_QUERY_TOKENS: usize = 32;
const MAX_CHUNKS_PER_SOURCE: usize = 256;
const MAX_PENDING_EMBEDDINGS: usize = 128;
const CHUNK_CHARS: usize = 1_200;
const CHUNK_OVERLAP_CHARS: usize = 180;

static SQLITE_VEC_REGISTER: Once = Once::new();
#[cfg(feature = "context-vector-fastembed")]
static FASTEMBED_MODEL: OnceLock<Result<Mutex<TextEmbedding>, String>> = OnceLock::new();

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRetrievalMode {
    Lexical,
    #[default]
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct ContextRetrievalQuery {
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: Option<String>,
    pub query: String,
    pub limit: usize,
    pub mode: ContextRetrievalMode,
    pub include_contradictions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalCapabilities {
    pub lexical: String,
    pub vector_index: String,
    pub embedding_provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    pub degraded_to_lexical: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degradation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCitation {
    pub citation_id: String,
    pub source_id: String,
    pub source_revision: String,
    pub source_kind: String,
    pub title: String,
    pub source_locator: String,
    pub content_hash: String,
    pub chunk_id: String,
    pub chunk_ordinal: u32,
    pub line_start: u32,
    pub line_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalHit {
    pub chunk_id: String,
    pub snippet: String,
    pub score: f64,
    pub retrieval_modes: Vec<String>,
    pub citation: ContextCitation,
    pub contradiction_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextContradictionCandidate {
    pub contradiction_id: String,
    pub status: String,
    pub summary: String,
    pub left_citation_id: String,
    pub right_citation_id: String,
    pub shared_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalResult {
    pub schema: String,
    pub query: String,
    pub mode_requested: ContextRetrievalMode,
    pub mode_used: ContextRetrievalMode,
    pub result_count: usize,
    pub indexed_source_count: usize,
    pub indexed_chunk_count: usize,
    pub hits: Vec<ContextRetrievalHit>,
    pub contradictions: Vec<ContextContradictionCandidate>,
    pub capabilities: ContextRetrievalCapabilities,
}

#[derive(Debug, Clone)]
struct IndexedChunk {
    row_id: i64,
    chunk_id: String,
    content: String,
    source_id: String,
    source_revision: String,
    source_kind: String,
    title: String,
    source_locator: String,
    content_hash: String,
    ordinal: u32,
    line_start: u32,
    line_end: u32,
}

#[derive(Debug, Clone)]
struct RankedChunk {
    chunk: IndexedChunk,
    score: f64,
    modes: BTreeSet<String>,
}

/// Rebuildable retrieval projection attached to the canonical Focusa SQLite DB.
#[derive(Debug, Clone)]
pub struct ContextRetrievalIndex {
    db_path: PathBuf,
}

impl ContextRetrievalIndex {
    pub fn from_persistence(persistence: &SqlitePersistence) -> Self {
        Self {
            db_path: persistence.database_path(),
        }
    }

    pub fn at_path(path: impl AsRef<Path>) -> Self {
        Self {
            db_path: path.as_ref().to_path_buf(),
        }
    }

    /// Synchronize exact-scope canonical sources into the derived index, then
    /// run deterministic bounded retrieval. Missing vectors never block FTS5.
    pub fn retrieve(
        &self,
        canonical_sources: &[ContextSourceRecord],
        request: ContextRetrievalQuery,
    ) -> anyhow::Result<ContextRetrievalResult> {
        let query = request.query.trim();
        if query.is_empty() {
            return Err(anyhow!("context retrieval query must not be empty"));
        }
        if request.project_root.trim().is_empty() || request.continuity_id.trim().is_empty() {
            return Err(anyhow!("project_root and continuity_id are required"));
        }

        let limit = request.limit.clamp(1, 50);
        let mut conn = self.open()?;
        self.synchronize_sources(&mut conn, canonical_sources, &request)?;

        let lexical = lexical_search(&conn, &request, limit.saturating_mul(4).max(16))?;
        let mut capabilities = ContextRetrievalCapabilities {
            lexical: "sqlite_fts5.available".to_string(),
            vector_index: "sqlite_vec.available".to_string(),
            embedding_provider: "fastembed.disabled".to_string(),
            embedding_model: None,
            degraded_to_lexical: false,
            degradation_reason: None,
        };

        let vector_mode_enabled = std::env::var("FOCUSA_CONTEXT_VECTOR_MODE")
            .map(|value| value.eq_ignore_ascii_case("fastembed"))
            .unwrap_or(false);
        let mut vector = Vec::new();
        let mut mode_used = request.mode;

        if request.mode == ContextRetrievalMode::Hybrid && vector_mode_enabled {
            match self.prepare_and_search_vectors(
                &mut conn,
                &request,
                limit.saturating_mul(4).max(16),
            ) {
                Ok(hits) => {
                    vector = hits;
                    capabilities.embedding_provider = "fastembed.available".to_string();
                    capabilities.embedding_model = Some(EMBEDDING_MODEL.to_string());
                }
                Err(error) => {
                    mode_used = ContextRetrievalMode::Lexical;
                    capabilities.embedding_provider = "fastembed.unavailable".to_string();
                    capabilities.degraded_to_lexical = true;
                    capabilities.degradation_reason =
                        Some(bounded_message(&error.to_string(), 320));
                }
            }
        } else if request.mode == ContextRetrievalMode::Hybrid {
            mode_used = ContextRetrievalMode::Lexical;
            capabilities.degraded_to_lexical = true;
            capabilities.degradation_reason = Some(
                "FOCUSA_CONTEXT_VECTOR_MODE is not fastembed; deterministic FTS5 fallback active"
                    .to_string(),
            );
        }

        let mut ranked = reciprocal_rank_fusion(lexical, vector);
        ranked.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.chunk.chunk_id.cmp(&right.chunk.chunk_id))
        });
        ranked.truncate(limit);

        let max_score = ranked
            .first()
            .map(|hit| hit.score)
            .unwrap_or(1.0)
            .max(f64::EPSILON);
        let mut hits: Vec<ContextRetrievalHit> = ranked
            .into_iter()
            .map(|ranked| {
                let citation = citation_for(&ranked.chunk);
                ContextRetrievalHit {
                    chunk_id: ranked.chunk.chunk_id,
                    snippet: bounded_message(&ranked.chunk.content, 1_600),
                    score: round_score(ranked.score / max_score),
                    retrieval_modes: ranked.modes.into_iter().collect(),
                    citation,
                    contradiction_refs: Vec::new(),
                }
            })
            .collect();

        let contradictions = if request.include_contradictions {
            detect_contradictions(&hits)
        } else {
            Vec::new()
        };
        let contradiction_members: HashMap<&str, Vec<String>> = contradictions
            .iter()
            .flat_map(|candidate| {
                [
                    (
                        candidate.left_citation_id.as_str(),
                        candidate.contradiction_id.clone(),
                    ),
                    (
                        candidate.right_citation_id.as_str(),
                        candidate.contradiction_id.clone(),
                    ),
                ]
            })
            .fold(
                HashMap::new(),
                |mut map, (citation_id, contradiction_id)| {
                    map.entry(citation_id).or_default().push(contradiction_id);
                    map
                },
            );
        for hit in &mut hits {
            hit.contradiction_refs = contradiction_members
                .get(hit.citation.citation_id.as_str())
                .cloned()
                .unwrap_or_default();
        }

        let (indexed_source_count, indexed_chunk_count) = scope_counts(&conn, &request)?;
        Ok(ContextRetrievalResult {
            schema: "focusa.context_retrieval_result.v1".to_string(),
            query: query.to_string(),
            mode_requested: request.mode,
            mode_used,
            result_count: hits.len(),
            indexed_source_count,
            indexed_chunk_count,
            hits,
            contradictions,
            capabilities,
        })
    }

    fn open(&self) -> anyhow::Result<Connection> {
        register_sqlite_vec();
        let conn = Connection::open(&self.db_path).with_context(|| {
            format!(
                "open Context retrieval SQLite at {}",
                self.db_path.display()
            )
        })?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        initialize_retrieval_schema(&conn)?;
        Ok(conn)
    }

    fn synchronize_sources(
        &self,
        conn: &mut Connection,
        canonical_sources: &[ContextSourceRecord],
        request: &ContextRetrievalQuery,
    ) -> anyhow::Result<()> {
        let scoped: Vec<&ContextSourceRecord> = canonical_sources
            .iter()
            .filter(|source| {
                source.project_root == request.project_root
                    && source.continuity_id == request.continuity_id
                    && request
                        .attachment_id
                        .as_ref()
                        .map(|attachment| source.attachment_id == *attachment)
                        .unwrap_or(true)
                    && !source.content.trim().is_empty()
            })
            .collect();

        let canonical_source_ids: HashSet<String> = scoped
            .iter()
            .map(|source| source.source_id.clone())
            .collect();
        let tx = conn.transaction()?;
        let indexed_source_ids: Vec<String> = {
            let (sql, values): (&str, Vec<&str>) = if let Some(attachment_id) =
                request.attachment_id.as_deref()
            {
                (
                    "SELECT DISTINCT source_id FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND attachment_id=?3",
                    vec![
                        request.project_root.as_str(),
                        request.continuity_id.as_str(),
                        attachment_id,
                    ],
                )
            } else {
                (
                    "SELECT DISTINCT source_id FROM context_chunks WHERE project_root=?1 AND continuity_id=?2",
                    vec![
                        request.project_root.as_str(),
                        request.continuity_id.as_str(),
                    ],
                )
            };
            let mut statement = tx.prepare(sql)?;
            statement
                .query_map(rusqlite::params_from_iter(values), |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?
        };
        for stale_source_id in indexed_source_ids
            .into_iter()
            .filter(|source_id| !canonical_source_ids.contains(source_id))
        {
            let stale_chunk_ids: Vec<String> = {
                let mut statement = tx.prepare(
                    "SELECT chunk_id FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND source_id=?3",
                )?;
                statement
                    .query_map(
                        params![request.project_root, request.continuity_id, stale_source_id],
                        |row| row.get(0),
                    )?
                    .collect::<Result<Vec<_>, _>>()?
            };
            for chunk_id in stale_chunk_ids {
                let _ = tx.execute(
                    "DELETE FROM context_embeddings WHERE chunk_id=?1",
                    [&chunk_id],
                );
            }
            tx.execute(
                "DELETE FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND source_id=?3",
                params![request.project_root, request.continuity_id, stale_source_id],
            )?;
        }
        for source in scoped {
            let indexed_revision: Option<String> = tx
                .query_row(
                    "SELECT source_revision FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND attachment_id=?3 AND source_id=?4 LIMIT 1",
                    params![source.project_root, source.continuity_id, source.attachment_id, source.source_id],
                    |row| row.get(0),
                )
                .optional()?;
            let source_revision = effective_revision(source);
            if indexed_revision.as_deref() == Some(source_revision.as_str()) {
                continue;
            }

            let stale_chunk_ids: Vec<String> = {
                let mut statement = tx.prepare(
                    "SELECT chunk_id FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND attachment_id=?3 AND source_id=?4",
                )?;
                statement
                    .query_map(
                        params![
                            source.project_root,
                            source.continuity_id,
                            source.attachment_id,
                            source.source_id
                        ],
                        |row| row.get(0),
                    )?
                    .collect::<Result<Vec<_>, _>>()?
            };
            for chunk_id in stale_chunk_ids {
                let _ = tx.execute(
                    "DELETE FROM context_embeddings WHERE chunk_id=?1",
                    [&chunk_id],
                );
            }
            tx.execute(
                "DELETE FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND attachment_id=?3 AND source_id=?4",
                params![source.project_root, source.continuity_id, source.attachment_id, source.source_id],
            )?;

            for chunk in chunk_source(source) {
                tx.execute(
                    r#"INSERT INTO context_chunks(
                      chunk_id, project_root, continuity_id, attachment_id, source_id,
                      source_revision, source_kind, title, source_locator, content_hash,
                      ordinal, line_start, line_end, content, embedding_model
                    ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,NULL)"#,
                    params![
                        chunk.chunk_id,
                        source.project_root,
                        source.continuity_id,
                        source.attachment_id,
                        source.source_id,
                        source_revision,
                        source.source_kind,
                        source.title,
                        source.source_locator,
                        source.content_hash,
                        chunk.ordinal,
                        chunk.line_start,
                        chunk.line_end,
                        chunk.content,
                    ],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn prepare_and_search_vectors(
        &self,
        conn: &mut Connection,
        request: &ContextRetrievalQuery,
        candidate_limit: usize,
    ) -> anyhow::Result<Vec<IndexedChunk>> {
        let pending = chunks_missing_embeddings(conn, request, MAX_PENDING_EMBEDDINGS)?;
        if !pending.is_empty() {
            let documents: Vec<String> = pending
                .iter()
                .map(|chunk| format!("passage: {}", chunk.content))
                .collect();
            let embeddings = embed_texts(documents)?;
            if embeddings.len() != pending.len() {
                return Err(anyhow!(
                    "fastembed result count did not match pending Context chunks"
                ));
            }
            let tx = conn.transaction()?;
            for (chunk, embedding) in pending.iter().zip(embeddings.iter()) {
                if embedding.len() != EMBEDDING_DIMENSIONS {
                    return Err(anyhow!(
                        "fastembed dimensions {} did not match sqlite-vec dimensions {}",
                        embedding.len(),
                        EMBEDDING_DIMENSIONS
                    ));
                }
                tx.execute(
                    "INSERT OR REPLACE INTO context_embeddings(chunk_id, embedding) VALUES (?1, ?2)",
                    params![chunk.chunk_id, encode_embedding(embedding)],
                )?;
                tx.execute(
                    "UPDATE context_chunks SET embedding_model=?1 WHERE chunk_id=?2",
                    params![EMBEDDING_MODEL, chunk.chunk_id],
                )?;
            }
            tx.commit()?;
        }

        let mut query_embeddings = embed_texts(vec![format!("query: {}", request.query.trim())])?;
        let query_embedding = query_embeddings
            .pop()
            .ok_or_else(|| anyhow!("fastembed returned no query embedding"))?;
        vector_search(conn, request, &query_embedding, candidate_limit)
    }
}

#[derive(Debug)]
struct ChunkDraft {
    chunk_id: String,
    ordinal: u32,
    line_start: u32,
    line_end: u32,
    content: String,
}

fn register_sqlite_vec() {
    SQLITE_VEC_REGISTER.call_once(|| unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *mut std::ffi::c_char,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> std::ffi::c_int,
        >(
            sqlite_vec::sqlite3_vec_init as *const ()
        )));
    });
}

fn initialize_retrieval_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS context_chunks (
          id INTEGER PRIMARY KEY,
          chunk_id TEXT NOT NULL UNIQUE,
          project_root TEXT NOT NULL,
          continuity_id TEXT NOT NULL,
          attachment_id TEXT NOT NULL,
          source_id TEXT NOT NULL,
          source_revision TEXT NOT NULL,
          source_kind TEXT NOT NULL,
          title TEXT NOT NULL,
          source_locator TEXT NOT NULL,
          content_hash TEXT NOT NULL,
          ordinal INTEGER NOT NULL,
          line_start INTEGER NOT NULL,
          line_end INTEGER NOT NULL,
          content TEXT NOT NULL,
          embedding_model TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_context_chunks_scope
          ON context_chunks(project_root, continuity_id, attachment_id, source_id, ordinal);
        CREATE VIRTUAL TABLE IF NOT EXISTS context_chunks_fts USING fts5(
          title,
          content,
          source_id UNINDEXED,
          content='context_chunks',
          content_rowid='id',
          tokenize='unicode61 remove_diacritics 2'
        );
        CREATE TRIGGER IF NOT EXISTS context_chunks_ai AFTER INSERT ON context_chunks BEGIN
          INSERT INTO context_chunks_fts(rowid,title,content,source_id)
          VALUES (new.id,new.title,new.content,new.source_id);
        END;
        CREATE TRIGGER IF NOT EXISTS context_chunks_ad AFTER DELETE ON context_chunks BEGIN
          INSERT INTO context_chunks_fts(context_chunks_fts,rowid,title,content,source_id)
          VALUES ('delete',old.id,old.title,old.content,old.source_id);
        END;
        CREATE TRIGGER IF NOT EXISTS context_chunks_au AFTER UPDATE OF title,content,source_id ON context_chunks BEGIN
          INSERT INTO context_chunks_fts(context_chunks_fts,rowid,title,content,source_id)
          VALUES ('delete',old.id,old.title,old.content,old.source_id);
          INSERT INTO context_chunks_fts(rowid,title,content,source_id)
          VALUES (new.id,new.title,new.content,new.source_id);
        END;
        CREATE VIRTUAL TABLE IF NOT EXISTS context_embeddings USING vec0(
          chunk_id TEXT PRIMARY KEY,
          embedding float[384]
        );
        "#,
    )?;
    Ok(())
}

fn chunk_source(source: &ContextSourceRecord) -> Vec<ChunkDraft> {
    let chars: Vec<char> = source.content.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < chars.len() && chunks.len() < MAX_CHUNKS_PER_SOURCE {
        let end = (start + CHUNK_CHARS).min(chars.len());
        let content: String = chars[start..end].iter().collect();
        let line_start = 1 + chars[..start].iter().filter(|ch| **ch == '\n').count() as u32;
        let line_end = line_start + content.chars().filter(|ch| *ch == '\n').count() as u32;
        let ordinal = chunks.len() as u32;
        let chunk_id = stable_id(
            "context-chunk",
            &format!(
                "{}\0{}\0{}\0{}\0{}",
                source.project_root,
                source.continuity_id,
                source.source_id,
                effective_revision(source),
                ordinal
            ),
        );
        chunks.push(ChunkDraft {
            chunk_id,
            ordinal,
            line_start,
            line_end,
            content,
        });
        if end == chars.len() {
            break;
        }
        start = end.saturating_sub(CHUNK_OVERLAP_CHARS);
    }
    chunks
}

fn effective_revision(source: &ContextSourceRecord) -> String {
    if source.source_revision.trim().is_empty() {
        format!("revision:{}:{}", source.revision, source.content_hash)
    } else {
        source.source_revision.clone()
    }
}

fn lexical_query(raw: &str) -> String {
    normalized_terms(raw)
        .into_iter()
        .take(MAX_QUERY_TOKENS)
        .map(|term| format!("\"{}\"", term.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn lexical_search(
    conn: &Connection,
    request: &ContextRetrievalQuery,
    limit: usize,
) -> anyhow::Result<Vec<IndexedChunk>> {
    let match_query = lexical_query(&request.query);
    if match_query.is_empty() {
        return Ok(Vec::new());
    }
    let mut sql = String::from(
        r#"SELECT c.id,c.chunk_id,c.content,c.source_id,c.source_revision,c.source_kind,
                  c.title,c.source_locator,c.content_hash,c.ordinal,c.line_start,c.line_end
           FROM context_chunks_fts f
           JOIN context_chunks c ON c.id=f.rowid
           WHERE context_chunks_fts MATCH ?1
             AND c.project_root=?2 AND c.continuity_id=?3"#,
    );
    if request.attachment_id.is_some() {
        sql.push_str(
            " AND c.attachment_id=?4 ORDER BY bm25(context_chunks_fts),c.chunk_id LIMIT ?5",
        );
    } else {
        sql.push_str(" ORDER BY bm25(context_chunks_fts),c.chunk_id LIMIT ?4");
    }
    let mut statement = conn.prepare(&sql)?;
    let rows = if let Some(attachment_id) = request.attachment_id.as_ref() {
        statement.query_map(
            params![
                match_query,
                request.project_root,
                request.continuity_id,
                attachment_id,
                limit as i64
            ],
            map_chunk,
        )?
    } else {
        statement.query_map(
            params![
                match_query,
                request.project_root,
                request.continuity_id,
                limit as i64
            ],
            map_chunk,
        )?
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn vector_search(
    conn: &Connection,
    request: &ContextRetrievalQuery,
    query_embedding: &[f32],
    limit: usize,
) -> anyhow::Result<Vec<IndexedChunk>> {
    let mut sql = String::from(
        r#"SELECT c.id,c.chunk_id,c.content,c.source_id,c.source_revision,c.source_kind,
                  c.title,c.source_locator,c.content_hash,c.ordinal,c.line_start,c.line_end
           FROM context_embeddings v
           JOIN context_chunks c ON c.chunk_id=v.chunk_id
           WHERE v.embedding MATCH ?1 AND k=?2
             AND c.project_root=?3 AND c.continuity_id=?4"#,
    );
    if request.attachment_id.is_some() {
        sql.push_str(" AND c.attachment_id=?5 ORDER BY v.distance,c.chunk_id");
    } else {
        sql.push_str(" ORDER BY v.distance,c.chunk_id");
    }
    let embedding = encode_embedding(query_embedding);
    let mut statement = conn.prepare(&sql)?;
    let rows = if let Some(attachment_id) = request.attachment_id.as_ref() {
        statement.query_map(
            params![
                embedding,
                limit as i64,
                request.project_root,
                request.continuity_id,
                attachment_id
            ],
            map_chunk,
        )?
    } else {
        statement.query_map(
            params![
                embedding,
                limit as i64,
                request.project_root,
                request.continuity_id
            ],
            map_chunk,
        )?
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn chunks_missing_embeddings(
    conn: &Connection,
    request: &ContextRetrievalQuery,
    limit: usize,
) -> anyhow::Result<Vec<IndexedChunk>> {
    let mut sql = String::from(
        r#"SELECT c.id,c.chunk_id,c.content,c.source_id,c.source_revision,c.source_kind,
                  c.title,c.source_locator,c.content_hash,c.ordinal,c.line_start,c.line_end
           FROM context_chunks c
           WHERE c.project_root=?1 AND c.continuity_id=?2 AND c.embedding_model IS NULL"#,
    );
    if request.attachment_id.is_some() {
        sql.push_str(" AND c.attachment_id=?3 ORDER BY c.chunk_id LIMIT ?4");
    } else {
        sql.push_str(" ORDER BY c.chunk_id LIMIT ?3");
    }
    let mut statement = conn.prepare(&sql)?;
    let rows = if let Some(attachment_id) = request.attachment_id.as_ref() {
        statement.query_map(
            params![
                request.project_root,
                request.continuity_id,
                attachment_id,
                limit as i64
            ],
            map_chunk,
        )?
    } else {
        statement.query_map(
            params![request.project_root, request.continuity_id, limit as i64],
            map_chunk,
        )?
    };
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn map_chunk(row: &rusqlite::Row<'_>) -> rusqlite::Result<IndexedChunk> {
    Ok(IndexedChunk {
        row_id: row.get(0)?,
        chunk_id: row.get(1)?,
        content: row.get(2)?,
        source_id: row.get(3)?,
        source_revision: row.get(4)?,
        source_kind: row.get(5)?,
        title: row.get(6)?,
        source_locator: row.get(7)?,
        content_hash: row.get(8)?,
        ordinal: row.get::<_, i64>(9)?.try_into().unwrap_or_default(),
        line_start: row.get::<_, i64>(10)?.try_into().unwrap_or_default(),
        line_end: row.get::<_, i64>(11)?.try_into().unwrap_or_default(),
    })
}

fn reciprocal_rank_fusion(
    lexical: Vec<IndexedChunk>,
    vector: Vec<IndexedChunk>,
) -> Vec<RankedChunk> {
    let mut fused: BTreeMap<String, RankedChunk> = BTreeMap::new();
    for (mode, candidates) in [("lexical", lexical), ("vector", vector)] {
        for (rank, chunk) in candidates.into_iter().enumerate() {
            let score = 1.0 / (60.0 + rank as f64 + 1.0);
            let entry = fused
                .entry(chunk.chunk_id.clone())
                .or_insert_with(|| RankedChunk {
                    chunk,
                    score: 0.0,
                    modes: BTreeSet::new(),
                });
            entry.score += score;
            entry.modes.insert(mode.to_string());
        }
    }
    fused.into_values().collect()
}

fn citation_for(chunk: &IndexedChunk) -> ContextCitation {
    let citation_id = stable_id("citation", &chunk.chunk_id);
    ContextCitation {
        citation_id,
        source_id: chunk.source_id.clone(),
        source_revision: chunk.source_revision.clone(),
        source_kind: chunk.source_kind.clone(),
        title: chunk.title.clone(),
        source_locator: chunk.source_locator.clone(),
        content_hash: chunk.content_hash.clone(),
        chunk_id: chunk.chunk_id.clone(),
        chunk_ordinal: chunk.ordinal,
        line_start: chunk.line_start,
        line_end: chunk.line_end,
    }
}

fn detect_contradictions(hits: &[ContextRetrievalHit]) -> Vec<ContextContradictionCandidate> {
    let mut candidates = Vec::new();
    for left_index in 0..hits.len() {
        for right_index in (left_index + 1)..hits.len() {
            let left = &hits[left_index];
            let right = &hits[right_index];
            if left.citation.source_id == right.citation.source_id {
                continue;
            }
            let left_terms: HashSet<String> = semantic_terms(&left.snippet);
            let right_terms: HashSet<String> = semantic_terms(&right.snippet);
            if left_terms.is_empty() || right_terms.is_empty() {
                continue;
            }
            let mut shared: Vec<String> = left_terms.intersection(&right_terms).cloned().collect();
            shared.sort();
            let overlap = shared.len() as f64 / left_terms.len().min(right_terms.len()) as f64;
            if overlap < 0.55
                || has_negative_polarity(&left.snippet) == has_negative_polarity(&right.snippet)
            {
                continue;
            }
            let ordered = if left.citation.citation_id <= right.citation.citation_id {
                (&left.citation.citation_id, &right.citation.citation_id)
            } else {
                (&right.citation.citation_id, &left.citation.citation_id)
            };
            candidates.push(ContextContradictionCandidate {
                contradiction_id: stable_id("contradiction", &format!("{}\0{}", ordered.0, ordered.1)),
                status: "candidate".to_string(),
                summary: "Retrieved source chunks share material terms but have opposite assertion polarity; operator review required".to_string(),
                left_citation_id: ordered.0.clone(),
                right_citation_id: ordered.1.clone(),
                shared_terms: shared.into_iter().take(12).collect(),
            });
        }
    }
    candidates.sort_by(|left, right| left.contradiction_id.cmp(&right.contradiction_id));
    candidates
}

fn normalized_terms(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-')
        .map(str::to_lowercase)
        .filter(|term| term.len() >= 2)
        .collect()
}

fn semantic_terms(text: &str) -> HashSet<String> {
    const STOP: &[&str] = &[
        "the", "and", "for", "with", "from", "that", "this", "into", "are", "was", "were", "not",
        "never", "cannot", "must", "should", "will", "would", "have", "has", "had",
    ];
    normalized_terms(text)
        .into_iter()
        .filter(|term| term.len() >= 4 && !STOP.contains(&term.as_str()))
        .collect()
}

fn has_negative_polarity(text: &str) -> bool {
    let normalized = format!(" {} ", text.to_lowercase().replace(['\n', '\t'], " "));
    [
        " not ",
        " never ",
        " cannot ",
        " must not ",
        " no longer ",
        " prohibited ",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

#[cfg(feature = "context-vector-fastembed")]
fn embed_texts(texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
    let model = FASTEMBED_MODEL.get_or_init(|| {
        let cache_dir = std::env::var("FOCUSA_CONTEXT_EMBEDDING_CACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("focusa-fastembed-cache"));
        let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_cache_dir(cache_dir)
            .with_show_download_progress(false);
        TextEmbedding::try_new(options)
            .map(Mutex::new)
            .map_err(|error| error.to_string())
    });
    let model = model.as_ref().map_err(|error| anyhow!(error.clone()))?;
    let model = model
        .lock()
        .map_err(|_| anyhow!("fastembed model lock poisoned"))?;
    model.embed(texts, Some(32)).map_err(Into::into)
}

#[cfg(not(feature = "context-vector-fastembed"))]
fn embed_texts(_texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
    Err(anyhow!(
        "fastembed provider is not built; rebuild focusa-core with context-vector-fastembed"
    ))
}

fn encode_embedding(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn scope_counts(
    conn: &Connection,
    request: &ContextRetrievalQuery,
) -> anyhow::Result<(usize, usize)> {
    let (sources, chunks): (i64, i64) = if let Some(attachment_id) = request.attachment_id.as_ref()
    {
        conn.query_row(
            "SELECT COUNT(DISTINCT source_id),COUNT(*) FROM context_chunks WHERE project_root=?1 AND continuity_id=?2 AND attachment_id=?3",
            params![request.project_root, request.continuity_id, attachment_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?
    } else {
        conn.query_row(
            "SELECT COUNT(DISTINCT source_id),COUNT(*) FROM context_chunks WHERE project_root=?1 AND continuity_id=?2",
            params![request.project_root, request.continuity_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?
    };
    Ok((
        sources.try_into().unwrap_or_default(),
        chunks.try_into().unwrap_or_default(),
    ))
}

fn stable_id(prefix: &str, value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("{prefix}:{}", hex::encode(&digest[..12]))
}

fn bounded_message(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn round_score(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContextSourceEvidence, ContextSourceHealth, ContextSourceReceipt};
    use chrono::Utc;

    fn source(source_id: &str, content: &str) -> ContextSourceRecord {
        ContextSourceRecord {
            source_id: source_id.to_string(),
            project_root: "/example".to_string(),
            continuity_id: "continuity".to_string(),
            attachment_id: "attachment".to_string(),
            source_kind: "markdown".to_string(),
            title: source_id.to_string(),
            content: content.to_string(),
            content_hash: stable_id("hash", content),
            idempotency_key: source_id.to_string(),
            revision: 1,
            committed_at: Utc::now(),
            evidence: ContextSourceEvidence {
                evidence_ref: format!("evidence:{source_id}"),
                target_ref: source_id.to_string(),
                result: "indexed".to_string(),
                content_hash: stable_id("hash", content),
                captured_at: Utc::now(),
            },
            receipt: ContextSourceReceipt {
                receipt_ref: format!("receipt:{source_id}"),
                operation_id: "focusa.context.source.ingest".to_string(),
                idempotency_key: source_id.to_string(),
                before_state_version: 0,
                after_state_version: 1,
                reversible: true,
                committed_at: Utc::now(),
            },
            source_locator: format!("file:///{source_id}.md"),
            source_revision: "git:1".to_string(),
            mime_type: "text/markdown".to_string(),
            adapter_id: "focusa.local_text.v1".to_string(),
            ingestion_status: "indexed".to_string(),
            extraction_diagnostics: Vec::new(),
            health: ContextSourceHealth::default(),
        }
    }

    #[test]
    fn lexical_retrieval_is_scoped_cited_contradiction_aware_and_restart_safe() {
        let path = std::env::temp_dir().join(format!(
            "focusa-c2-{}-{}.sqlite",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let index = ContextRetrievalIndex::at_path(&path);
        let sources = vec![
            source(
                "source-a",
                "The release policy requires signed artifacts for every deployment.",
            ),
            source(
                "source-b",
                "The release policy does not require signed artifacts for every deployment.",
            ),
            ContextSourceRecord {
                project_root: "/other".to_string(),
                ..source("source-c", "signed artifacts secret cross scope")
            },
        ];
        let request = ContextRetrievalQuery {
            project_root: "/example".to_string(),
            continuity_id: "continuity".to_string(),
            attachment_id: Some("attachment".to_string()),
            query: "release policy signed artifacts deployment".to_string(),
            limit: 8,
            mode: ContextRetrievalMode::Hybrid,
            include_contradictions: true,
        };
        let first = index.retrieve(&sources, request.clone()).unwrap();
        assert_eq!(first.mode_used, ContextRetrievalMode::Lexical);
        assert!(first.capabilities.degraded_to_lexical);
        assert_eq!(first.indexed_source_count, 2);
        assert_eq!(first.result_count, 2);
        assert_eq!(first.contradictions.len(), 1);
        assert!(
            first
                .hits
                .iter()
                .all(|hit| hit.citation.source_id != "source-c")
        );

        let resumed = ContextRetrievalIndex::at_path(&path)
            .retrieve(&sources, request)
            .unwrap();
        assert_eq!(
            first
                .hits
                .iter()
                .map(|hit| &hit.chunk_id)
                .collect::<Vec<_>>(),
            resumed
                .hits
                .iter()
                .map(|hit| &hit.chunk_id)
                .collect::<Vec<_>>()
        );
        let pruned = ContextRetrievalIndex::at_path(&path)
            .retrieve(
                &sources[..1],
                ContextRetrievalQuery {
                    project_root: "/example".to_string(),
                    continuity_id: "continuity".to_string(),
                    attachment_id: Some("attachment".to_string()),
                    query: "release policy signed artifacts deployment".to_string(),
                    limit: 8,
                    mode: ContextRetrievalMode::Lexical,
                    include_contradictions: true,
                },
            )
            .unwrap();
        assert_eq!(pruned.indexed_source_count, 1);
        assert!(pruned.contradictions.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[cfg(feature = "context-vector-fastembed")]
    #[test]
    fn fastembed_and_sqlite_vec_produce_hybrid_context_hits() {
        unsafe {
            std::env::set_var("FOCUSA_CONTEXT_VECTOR_MODE", "fastembed");
        }
        let path = std::env::temp_dir().join(format!(
            "focusa-c2-vector-{}-{}.sqlite",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let result = ContextRetrievalIndex::at_path(&path)
            .retrieve(
                &[source(
                    "source-vector",
                    "Production releases require cryptographic signatures on every deployment artifact.",
                )],
                ContextRetrievalQuery {
                    project_root: "/example".to_string(),
                    continuity_id: "continuity".to_string(),
                    attachment_id: Some("attachment".to_string()),
                    query: "deployment artifact signing policy".to_string(),
                    limit: 4,
                    mode: ContextRetrievalMode::Hybrid,
                    include_contradictions: false,
                },
            )
            .unwrap();
        assert_eq!(result.mode_used, ContextRetrievalMode::Hybrid);
        assert_eq!(
            result.capabilities.embedding_provider,
            "fastembed.available"
        );
        assert_eq!(
            result.capabilities.embedding_model.as_deref(),
            Some(EMBEDDING_MODEL)
        );
        assert_eq!(result.result_count, 1);
        assert!(
            result.hits[0]
                .retrieval_modes
                .contains(&"vector".to_string())
        );
        let _ = std::fs::remove_file(path);
    }
}
