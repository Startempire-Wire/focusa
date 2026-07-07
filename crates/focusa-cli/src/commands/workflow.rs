//! `focusa workflow` — canonical operator/agent workflow templates.
//!
//! Gives LLM agents a fast, accurate scaffold instead of ad-hoc command guessing.

use clap::Subcommand;
use serde::Serialize;

#[derive(Subcommand, Debug)]
pub enum WorkflowCmd {
    /// List available workflow templates.
    List,
    /// Show one workflow template by id.
    Show { id: String },
    /// Print a paste-ready command sequence for one template.
    Apply {
        id: String,
        /// Safe project root to substitute into command examples.
        #[arg(long, default_value = "<project-root>")]
        project_root: String,
        /// Continuity id to substitute into command examples.
        #[arg(long, default_value = "<continuity-id>")]
        continuity_id: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowTemplate {
    pub id: &'static str,
    pub when_to_use: &'static str,
    pub expected_outcome: &'static str,
    pub commands: Vec<&'static str>,
    pub recovery_hint: &'static str,
}

pub fn templates() -> Vec<WorkflowTemplate> {
    vec![
        WorkflowTemplate {
            id: "long-refactor",
            when_to_use: "Multi-file change where scope drift, stale context, or compile/test latency could mislead the agent.",
            expected_outcome: "Small proof-backed refactor slices with Workpoint checkpoints and final pushed commit.",
            commands: vec![
                "focusa project identity --project-root <project-root>",
                "focusa trajectory view --project-root <project-root> --continuity-id <continuity-id>",
                "focusa workpoint checkpoint --project-root <project-root> --continuity-id <continuity-id> --mission '<mission>' --next-action '<next slice>'",
                "git status && git diff --stat",
                "<run targeted tests>",
                "focusa workpoint evidence-link --target-ref <target> --result '<proof>' --evidence-ref <test-or-commit>",
                "git pull --rebase && git push",
            ],
            recovery_hint: "If context becomes stale, run focusa workpoint resume before editing and avoid broad repo rewrites.",
        },
        WorkflowTemplate {
            id: "multi-session-resume",
            when_to_use: "A Pi/agent session resumes after compaction, model switch, or unsafe cwd like /root.",
            expected_outcome: "Verified project scope and canonical Workpoint before durable writes.",
            commands: vec![
                "focusa project identity --project-root <project-root>",
                "focusa workpoint resume --project-root <project-root> --continuity-id <continuity-id>",
                "focusa trajectory view --project-root <project-root> --continuity-id <continuity-id>",
                "git status --short --branch",
                "bd ready",
                "focusa workpoint checkpoint --project-root <project-root> --continuity-id <continuity-id> --mission '<mission>' --next-action '<next slice>'",
            ],
            recovery_hint: "If Workpoint is degraded, checkpoint from latest operator instruction rather than trusting transcript tail.",
        },
        WorkflowTemplate {
            id: "incident-response",
            when_to_use: "Daemon, service, pairing, or release path appears broken and operator needs fast safe triage.",
            expected_outcome: "Root cause class, bounded recovery command, evidence handle, and no destructive guesswork.",
            commands: vec![
                "focusa doctor --scope host",
                "focusa recover --dry-run --project-root <project-root> --continuity-id <continuity-id>",
                "focusa resource mode",
                "focusa workpoint resume --project-root <project-root> --continuity-id <continuity-id>",
                "<run targeted health or static test>",
                "focusa workpoint evidence-link --target-ref incident --result '<diagnosis>' --evidence-ref <log-or-test>",
            ],
            recovery_hint: "If daemon is down, prefer focusa recover --dry-run before restart/kill commands.",
        },
        WorkflowTemplate {
            id: "agent-handoff",
            when_to_use: "Stopping, compacting, or handing work to another agent/model.",
            expected_outcome: "Portable continuation packet with mission, proof, blockers, and exact next command.",
            commands: vec![
                "focusa workpoint checkpoint --project-root <project-root> --continuity-id <continuity-id> --mission '<mission>' --next-action '<next action>'",
                "focusa trajectory checkpoint --project-root <project-root> --continuity-id <continuity-id>",
                "git status --short --branch",
                "bd list --status in_progress",
                "focusa predict record --type next_action_success --outcome '<expected outcome>'",
            ],
            recovery_hint: "If Focusa tools reject stale frames, write scratch notes and resume/checkpoint after project identity verification.",
        },
        WorkflowTemplate {
            id: "feature-add",
            when_to_use: "Adding a bounded feature or command surface with tests/docs.",
            expected_outcome: "Implemented feature, static/live proof, docs, bead closure, pushed commits.",
            commands: vec![
                "bd update <bead-id> --status in_progress",
                "focusa call-stack design --project-root <project-root> --entry-name <command-or-route> --mission '<feature>'",
                "git status && rg '<feature>' crates apps docs tests",
                "<edit smallest implementation slice>",
                "<run targeted tests>",
                "bd close <bead-id> --reason 'Completed: ... Evidence: tests/... PASS; docs/...; crates/...'",
                "git pull --rebase && git push",
            ],
            recovery_hint: "If tests require unavailable toolchains, record the environment blocker and keep static guards precise.",
        },
        WorkflowTemplate {
            id: "doc-update",
            when_to_use: "Behavior changed or operator docs are stale/confusing.",
            expected_outcome: "Docs match implementation and proof commands remain accurate.",
            commands: vec![
                "focusa project identity --project-root <project-root>",
                "rg '<old behavior>' README.md docs/current docs tests",
                "<edit docs and any static guard markers>",
                "<run docs/static tests>",
                "focusa workpoint evidence-link --target-ref docs --result '<doc proof>' --evidence-ref <test>",
                "git pull --rebase && git push",
            ],
            recovery_hint: "Prefer generated summary refs over hard-coded counts; update generators when generated docs change.",
        },
    ]
}

pub async fn run(cmd: WorkflowCmd, json_output: bool) -> anyhow::Result<()> {
    match cmd {
        WorkflowCmd::List => print_templates(&templates(), json_output),
        WorkflowCmd::Show { id } => {
            let template = find_template(&id)?;
            print_templates(&[template], json_output)
        }
        WorkflowCmd::Apply {
            id,
            project_root,
            continuity_id,
        } => {
            let template = find_template(&id)?;
            let applied = apply_template(&template, &project_root, &continuity_id);
            if json_output {
                println!("{}", serde_json::to_string_pretty(&applied)?);
            } else {
                println!("focusa workflow apply {}", template.id);
                println!("when: {}", template.when_to_use);
                println!("outcome: {}", template.expected_outcome);
                println!("commands:");
                for command in applied.commands {
                    println!("  - {command}");
                }
                println!("recovery_hint: {}", template.recovery_hint);
            }
            Ok(())
        }
    }
}

fn find_template(id: &str) -> anyhow::Result<WorkflowTemplate> {
    templates()
        .into_iter()
        .find(|template| template.id == id)
        .ok_or_else(|| anyhow::anyhow!("unknown workflow template: {id}"))
}

#[derive(Debug, Serialize)]
struct AppliedWorkflow {
    id: &'static str,
    when_to_use: &'static str,
    expected_outcome: &'static str,
    commands: Vec<String>,
    recovery_hint: &'static str,
}

fn apply_template(
    template: &WorkflowTemplate,
    project_root: &str,
    continuity_id: &str,
) -> AppliedWorkflow {
    AppliedWorkflow {
        id: template.id,
        when_to_use: template.when_to_use,
        expected_outcome: template.expected_outcome,
        commands: template
            .commands
            .iter()
            .map(|command| {
                command
                    .replace("<project-root>", project_root)
                    .replace("<continuity-id>", continuity_id)
            })
            .collect(),
        recovery_hint: template.recovery_hint,
    }
}

fn print_templates(templates: &[WorkflowTemplate], json_output: bool) -> anyhow::Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(&templates)?);
    } else {
        for template in templates {
            println!("{}", template.id);
            println!("  when: {}", template.when_to_use);
            println!("  outcome: {}", template.expected_outcome);
            println!("  commands: {}", template.commands.len());
            println!("  recovery_hint: {}", template.recovery_hint);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_list_has_six_templates() {
        assert_eq!(templates().len(), 6);
    }
}
