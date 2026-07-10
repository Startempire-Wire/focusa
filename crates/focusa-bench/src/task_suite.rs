//! 150-task benchmark suite — public/private split (Spec 113).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Task kind — the category of work the agent must perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// Generate code for a given spec.
    CodeGen,
    /// Find and fix a bug in provided code.
    BugFix,
    /// Write tests for provided code.
    TestWrite,
    /// Read code/spec and produce documentation.
    DocGen,
    /// Refactor provided code while preserving behavior.
    Refactor,
    /// Multi-step agent task (tool use, planning, recovery).
    AgentWorkflow,
}

impl TaskKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskKind::CodeGen => "code_gen",
            TaskKind::BugFix => "bug_fix",
            TaskKind::TestWrite => "test_write",
            TaskKind::DocGen => "doc_gen",
            TaskKind::Refactor => "refactor",
            TaskKind::AgentWorkflow => "agent_workflow",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: TaskKind,
    /// True if public-facing (75 of 150); false for private held-out (75 of 150).
    pub public: bool,
    /// Short prompt body (input to the agent).
    pub prompt: String,
    /// Reference / canonical output (for grading; private for non-public tasks).
    pub reference: Option<String>,
    /// Grader function name (e.g., "code_compile", "tests_pass", "exact_match").
    pub grader: String,
    /// Difficulty 1..=5.
    pub difficulty: u8,
    /// Required capability tags (e.g., "long_context", "tool_use", "recovery").
    pub requires: Vec<String>,
    /// Estimated tokens to complete.
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskPool {
    pub tasks: BTreeMap<String, Task>,
}

impl TaskPool {
    /// Returns the canonical 150-task pool — 75 public + 75 private.
    pub fn canonical() -> Self {
        let mut pool = TaskPool::default();
        let kinds = [
            TaskKind::CodeGen,
            TaskKind::BugFix,
            TaskKind::TestWrite,
            TaskKind::DocGen,
            TaskKind::Refactor,
            TaskKind::AgentWorkflow,
        ];
        for kind in kinds {
            for i in 0..25 {
                let is_public = i < 13; // ~half public, ~half private per kind (13+12=25)
                let id = format!("{}_{:03}", kind.as_str(), i);
                let difficulty = ((i % 5) + 1) as u8;
                let task = Task {
                    id: id.clone(),
                    kind,
                    public: is_public,
                    prompt: format!(
                        "{} task #{}: implement the requested change.",
                        kind.as_str(),
                        i
                    ),
                    reference: if is_public {
                        Some(format!("REFERENCE_OK_{}_{}", kind.as_str(), i))
                    } else {
                        None
                    },
                    grader: match kind {
                        TaskKind::CodeGen => "code_compile".to_string(),
                        TaskKind::BugFix => "tests_pass".to_string(),
                        TaskKind::TestWrite => "coverage_threshold".to_string(),
                        TaskKind::DocGen => "exact_match".to_string(),
                        TaskKind::Refactor => "behavior_preserved".to_string(),
                        TaskKind::AgentWorkflow => "task_complete".to_string(),
                    },
                    difficulty,
                    requires: vec![
                        kind.as_str().to_string(),
                        format!("difficulty_{difficulty}"),
                    ],
                    estimated_tokens: 1_000 + (i as u32 * 500),
                };
                pool.tasks.insert(id, task);
            }
        }
        pool
    }

    pub fn public_tasks(&self) -> Vec<&Task> {
        self.tasks.values().filter(|t| t.public).collect()
    }

    pub fn private_tasks(&self) -> Vec<&Task> {
        self.tasks.values().filter(|t| !t.public).collect()
    }

    pub fn count(&self) -> usize {
        self.tasks.len()
    }

    pub fn public_count(&self) -> usize {
        self.public_tasks().len()
    }

    pub fn private_count(&self) -> usize {
        self.private_tasks().len()
    }
}
