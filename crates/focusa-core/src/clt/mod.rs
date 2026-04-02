//! Context Lineage Tree (CLT) — docs/17-context-lineage-tree.md
//!
//! Append-only, immutable tree of interaction history.
//! 7 design rules (non-negotiable):
//!   1. Append-only
//!   2. Nodes immutable once written
//!   3. Never mutates Focus State
//!   4. Focus State refs exactly one CLT head
//!   5. Compaction inserts — never deletes
//!   6. Branches may be abandoned but never erased
//!   7. Inspectable, navigable, replayable

use crate::types::*;
use chrono::Utc;

/// Append an interaction node to the CLT.
pub fn append_interaction(
    clt: &mut CltState,
    session_id: Option<SessionId>,
    role: &str,
    content_ref: Option<&str>,
    metadata: CltMetadata,
) -> String {
    let node_id = format!("clt_{:06}", clt.nodes.len());
    let parent_id = clt.head_id.clone();

    let node = CltNode {
        node_id: node_id.clone(),
        node_type: CltNodeType::Interaction,
        parent_id,
        created_at: Utc::now(),
        session_id,
        payload: CltPayload::Interaction {
            role: role.into(),
            content_ref: content_ref.map(String::from),
        },
        metadata,
    };

    clt.nodes.push(node);
    clt.head_id = Some(node_id.clone());
    node_id
}

/// Insert a summary (compaction) node — never deletes existing nodes.
pub fn insert_summary(
    clt: &mut CltState,
    session_id: Option<SessionId>,
    summary: &str,
    covered_range: Vec<String>,
    compression_ratio: f64,
) -> String {
    let node_id = format!("clt_{:06}", clt.nodes.len());
    let parent_id = clt.head_id.clone();

    let node = CltNode {
        node_id: node_id.clone(),
        node_type: CltNodeType::Summary,
        parent_id,
        created_at: Utc::now(),
        session_id,
        payload: CltPayload::Summary {
            summary: summary.into(),
            covered_range,
            compression_ratio,
        },
        metadata: CltMetadata::default(),
    };

    clt.nodes.push(node);
    clt.head_id = Some(node_id.clone());
    node_id
}

/// Insert a branch marker.
pub fn insert_branch_marker(clt: &mut CltState, reason: &str, branches: Vec<String>) -> String {
    let node_id = format!("clt_{:06}", clt.nodes.len());
    let parent_id = clt.head_id.clone();

    let node = CltNode {
        node_id: node_id.clone(),
        node_type: CltNodeType::BranchMarker,
        parent_id,
        created_at: Utc::now(),
        session_id: None,
        payload: CltPayload::BranchMarker {
            reason: reason.into(),
            branches,
        },
        metadata: CltMetadata::default(),
    };

    clt.nodes.push(node);
    clt.head_id = Some(node_id.clone());
    node_id
}

/// Walk the path from head to root.
pub fn lineage_path(clt: &CltState) -> Vec<&CltNode> {
    let mut path = Vec::new();
    let mut current_id = clt.head_id.as_deref();
    while let Some(id) = current_id {
        if let Some(node) = clt.nodes.iter().find(|n| n.node_id == id) {
            path.push(node);
            current_id = node.parent_id.as_deref();
        } else {
            break;
        }
    }
    path
}

/// Count nodes by type.
pub fn node_counts(clt: &CltState) -> (usize, usize, usize) {
    let interactions = clt
        .nodes
        .iter()
        .filter(|n| n.node_type == CltNodeType::Interaction)
        .count();
    let summaries = clt
        .nodes
        .iter()
        .filter(|n| n.node_type == CltNodeType::Summary)
        .count();
    let markers = clt
        .nodes
        .iter()
        .filter(|n| n.node_type == CltNodeType::BranchMarker)
        .count();
    (interactions, summaries, markers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_interaction() {
        let mut clt = CltState::default();
        let id = append_interaction(
            &mut clt,
            None,
            "user",
            Some("ref://abc"),
            CltMetadata::default(),
        );
        assert_eq!(clt.head_id, Some(id));
        assert_eq!(clt.nodes.len(), 1);
        assert!(clt.nodes[0].parent_id.is_none());
    }

    #[test]
    fn test_lineage_chain() {
        let mut clt = CltState::default();
        append_interaction(&mut clt, None, "user", None, CltMetadata::default());
        append_interaction(&mut clt, None, "assistant", None, CltMetadata::default());
        append_interaction(&mut clt, None, "user", None, CltMetadata::default());
        let path = lineage_path(&clt);
        assert_eq!(path.len(), 3);
        // Head is first in path.
        assert_eq!(path[0].node_id, "clt_000002");
    }

    #[test]
    fn test_summary_does_not_delete() {
        let mut clt = CltState::default();
        append_interaction(&mut clt, None, "user", None, CltMetadata::default());
        append_interaction(&mut clt, None, "assistant", None, CltMetadata::default());
        let before = clt.nodes.len();
        insert_summary(&mut clt, None, "summarized", vec!["clt_000000".into()], 0.5);
        assert_eq!(clt.nodes.len(), before + 1); // Inserted, not deleted.
    }

    #[test]
    fn test_node_counts() {
        let mut clt = CltState::default();
        append_interaction(&mut clt, None, "user", None, CltMetadata::default());
        insert_summary(&mut clt, None, "sum", vec![], 1.0);
        insert_branch_marker(&mut clt, "fork", vec!["a".into(), "b".into()]);
        let (i, s, b) = node_counts(&clt);
        assert_eq!((i, s, b), (1, 1, 1));
    }
}

/// Compact the CLT when interaction nodes exceed threshold.
/// 
/// Per docs/17 rule 5: "Compaction inserts — never deletes."
/// Groups the oldest N interaction nodes, asks LLM for a summary,
/// inserts a Summary node covering that range.
/// Returns the number of nodes summarized, or 0 if no compaction needed.
pub async fn compact_if_needed(
    clt: &mut CltState,
    session_id: Option<SessionId>,
    threshold: usize,
    batch_size: usize,
) -> usize {
    let (interaction_count, _, _) = node_counts(clt);
    if interaction_count < threshold {
        return 0;
    }

    // Collect oldest interaction nodes not already covered by a summary
    let summary_covered: std::collections::HashSet<String> = clt.nodes.iter()
        .filter_map(|n| {
            if let CltPayload::Summary { ref covered_range, .. } = n.payload {
                Some(covered_range.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let uncovered_interactions: Vec<&CltNode> = clt.nodes.iter()
        .filter(|n| n.node_type == CltNodeType::Interaction && !summary_covered.contains(&n.node_id))
        .take(batch_size)
        .collect();

    if uncovered_interactions.is_empty() {
        return 0;
    }

    // Build content for summarization
    let content: Vec<String> = uncovered_interactions.iter().map(|n| {
        match &n.payload {
            CltPayload::Interaction { role, content_ref } => {
                format!("[{}] {}: {}", n.node_id, role, content_ref.as_deref().unwrap_or("(no content)"))
            }
            _ => format!("[{}] (non-interaction)", n.node_id),
        }
    }).collect();

    let covered_ids: Vec<String> = uncovered_interactions.iter().map(|n| n.node_id.clone()).collect();
    let count = covered_ids.len();

    // Try LLM summarization
    let api_key = std::env::var("MINIMAX_API_KEY").unwrap_or_default();
    let summary_text = if !api_key.is_empty() {
        let prompt = format!(
            "Summarize these {} conversation interactions in 2-3 sentences:\n\n{}",
            count,
            content.join("\n"),
        );
        let client = reqwest::Client::new();
        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.post("https://api.minimax.io/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&serde_json::json!({
                    "model": "MiniMax-M2.7",
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": 200,
                    "temperature": 0.3,
                }))
                .send(),
        ).await {
            Ok(Ok(resp)) => {
                resp.json::<serde_json::Value>().await.ok()
                    .and_then(|d| d.pointer("/choices/0/message/content").and_then(|v| v.as_str()).map(String::from))
                    .unwrap_or_else(|| format!("Summary of {} interactions", count))
            }
            _ => format!("Summary of {} interactions (LLM timeout)", count),
        }
    } else {
        format!("Summary of {} interactions", count)
    };

    let compression = count as f64 / 1.0; // original count compressed to 1 summary
    insert_summary(clt, session_id, &summary_text, covered_ids, compression);
    tracing::info!(summarized = count, "CLT compaction: {} nodes summarized", count);
    count
}
