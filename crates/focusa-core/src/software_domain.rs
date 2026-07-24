//! Evidence-backed Software domain projection using Tree-sitter, ast-grep, and petgraph.

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tree_sitter::{Language, Parser};

const MAX_CHANGED_FILES: usize = 128;
const MAX_PATTERNS: usize = 16;
const MAX_MATCHES_PER_PATTERN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareFileChange {
    pub path: PathBuf,
    pub source: String,
    pub removed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareNode {
    pub node_id: String,
    pub kind: String,
    pub path: String,
    pub label: String,
    pub content_sha256: String,
    pub parsed_root_kind: String,
    pub parser_has_error: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareEdge {
    pub relation: String,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstGrepMatch {
    pub pattern: String,
    pub path: String,
    pub line: Option<u64>,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareGraphProjection {
    pub schema: String,
    pub project_root: String,
    pub revision: u64,
    pub nodes: Vec<SoftwareNode>,
    pub edges: Vec<(String, String, SoftwareEdge)>,
    pub ast_grep_matches: Vec<AstGrepMatch>,
    pub changed_file_count: usize,
    pub bounded: bool,
    pub canonical_state_unchanged: bool,
}

pub struct SoftwareDomainProjector {
    project_root: PathBuf,
    graph: StableDiGraph<SoftwareNode, SoftwareEdge>,
    by_path: HashMap<PathBuf, NodeIndex>,
    matches: Vec<AstGrepMatch>,
    revision: u64,
    ast_grep_binary: String,
}

impl SoftwareDomainProjector {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            graph: StableDiGraph::new(),
            by_path: HashMap::new(),
            matches: Vec::new(),
            revision: 0,
            ast_grep_binary: std::env::var("FOCUSA_AST_GREP_BIN").unwrap_or_else(|_| "ast-grep".into()),
        }
    }

    pub fn apply_changes(
        &mut self,
        changes: &[SoftwareFileChange],
        language: &Language,
        ast_patterns: &[String],
    ) -> Result<SoftwareGraphProjection, String> {
        if changes.len() > MAX_CHANGED_FILES {
            return Err(format!("software graph change set exceeds {MAX_CHANGED_FILES} files"));
        }
        if ast_patterns.len() > MAX_PATTERNS {
            return Err(format!("software graph pattern set exceeds {MAX_PATTERNS}"));
        }
        for change in changes {
            if !change.path.starts_with(&self.project_root) {
                return Err("software graph path is outside project root".into());
            }
            self.remove_path(&change.path);
            if change.removed {
                continue;
            }
            let node = parse_file(&change.path, &change.source, language)?;
            let index = self.graph.add_node(node);
            self.by_path.insert(change.path.clone(), index);
            for pattern in ast_patterns {
                let pattern_node = self.graph.add_node(pattern_node(pattern));
                let evidence = format!("evidence:ast-grep:{}:{}", stable_path(&change.path), digest(pattern));
                self.graph.add_edge(index, pattern_node, SoftwareEdge {
                    relation: "matches_structural_pattern".into(),
                    evidence_ref: evidence.clone(),
                });
                self.matches.extend(run_ast_grep(&self.ast_grep_binary, &change.path, pattern, &evidence));
            }
        }
        self.revision = self.revision.saturating_add(1);
        Ok(self.projection(changes.len()))
    }

    fn remove_path(&mut self, path: &Path) {
        if let Some(index) = self.by_path.remove(path) {
            self.graph.remove_node(index);
        }
        let stable = stable_path(path);
        self.matches.retain(|item| item.path != stable);
    }

    fn projection(&self, changed_file_count: usize) -> SoftwareGraphProjection {
        let nodes = self.graph.node_weights().cloned().collect();
        let edges = self.graph.edge_references().map(|edge| {
            (
                self.graph[edge.source()].node_id.clone(),
                self.graph[edge.target()].node_id.clone(),
                edge.weight().clone(),
            )
        }).collect();
        SoftwareGraphProjection {
            schema: "focusa.software_graph_projection.v1".into(),
            project_root: self.project_root.display().to_string(),
            revision: self.revision,
            nodes,
            edges,
            ast_grep_matches: self.matches.clone(),
            changed_file_count,
            bounded: true,
            canonical_state_unchanged: true,
        }
    }
}

fn parse_file(path: &Path, source: &str, language: &Language) -> Result<SoftwareNode, String> {
    let mut parser = Parser::new();
    parser.set_language(language).map_err(|_| "tree-sitter language is incompatible")?;
    let tree = parser.parse(source, None).ok_or("tree-sitter did not produce a syntax tree")?;
    let root = tree.root_node();
    let hash = digest(source);
    Ok(SoftwareNode {
        node_id: format!("software-file:{}", stable_path(path)),
        kind: "source_file".into(),
        path: stable_path(path),
        label: path.file_name().and_then(|name| name.to_str()).unwrap_or("source").into(),
        content_sha256: hash.clone(),
        parsed_root_kind: root.kind().into(),
        parser_has_error: root.has_error(),
        evidence_refs: vec![format!("evidence:tree-sitter:{hash}")],
    })
}

fn pattern_node(pattern: &str) -> SoftwareNode {
    let hash = digest(pattern);
    SoftwareNode {
        node_id: format!("software-pattern:{hash}"),
        kind: "structural_pattern".into(),
        path: String::new(),
        label: pattern.chars().take(120).collect(),
        content_sha256: hash.clone(),
        parsed_root_kind: "ast_grep_pattern".into(),
        parser_has_error: false,
        evidence_refs: vec![format!("evidence:ast-grep-pattern:{hash}")],
    }
}

fn run_ast_grep(binary: &str, path: &Path, pattern: &str, evidence_ref: &str) -> Vec<AstGrepMatch> {
    let output = Command::new(binary)
        .args(["scan", "--json=stream", "--pattern", pattern])
        .arg(path)
        .output();
    let Ok(output) = output else { return Vec::new(); };
    if !output.status.success() { return Vec::new(); }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .take(MAX_MATCHES_PER_PATTERN)
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .map(|value| AstGrepMatch {
            pattern: pattern.into(),
            path: stable_path(path),
            line: value.pointer("/range/start/line").and_then(Value::as_u64),
            evidence_ref: evidence_ref.into(),
        })
        .collect()
}

fn stable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
