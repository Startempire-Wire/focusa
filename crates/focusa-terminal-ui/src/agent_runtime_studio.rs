//! Deterministic terminal projection for the Spec 140 Agent Runtime Studio.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AgentRuntimeStudio {
    pub constitution_id: String,
    #[serde(default)]
    pub panels: Vec<AgentRuntimeStudioPanel>,
}

#[derive(Debug, Deserialize)]
pub struct AgentRuntimeStudioPanel {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub data: Value,
}

pub fn render_agent_runtime_studio(value: &Value) -> Result<String, serde_json::Error> {
    let studio: AgentRuntimeStudio = serde_json::from_value(value.clone())?;
    let mut lines = vec![
        "FOCUSA AGENT RUNTIME STUDIO".to_string(),
        format!("Constitution: {}", studio.constitution_id),
        "─".repeat(72),
    ];
    for panel in studio.panels {
        lines.push(format!("[{}] {}", panel.id, panel.title));
        lines.push(format!("  {}", compact(&panel.data)));
    }
    Ok(lines.join("\n"))
}

fn compact(value: &Value) -> String {
    let rendered = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    if rendered.chars().count() <= 160 {
        rendered
    } else {
        format!("{}…", rendered.chars().take(159).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_bounded_studio_panels() {
        let rendered = render_agent_runtime_studio(&json!({
            "constitution_id":"constitution-1",
            "panels":[
                {"id":"role-grounding","title":"Role & Grounding","data":{"role":"builder"}},
                {"id":"rollback","title":"Rollback","data":{"receipt_required":true}}
            ]
        }))
        .unwrap();
        assert!(rendered.contains("FOCUSA AGENT RUNTIME STUDIO"));
        assert!(rendered.contains("role-grounding"));
        assert!(rendered.contains("receipt_required"));
    }
}
