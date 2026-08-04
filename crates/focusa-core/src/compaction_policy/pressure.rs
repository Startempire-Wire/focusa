use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_SAMPLES: usize = 128;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PressureStatistics {
    input_growth: VecDeque<u64>,
    tool_growth: VecDeque<u64>,
    ewma_input: Option<f64>,
    ewma_tool: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PressurePredictionInput {
    pub current_context: u64,
    pub context_window: u64,
    pub configured_reserve_floor: u64,
    pub configured_reserve_percent: u8,
    pub max_output_tokens: Option<u64>,
    pub projection_budget_tokens: u64,
    pub persistence_growth_allowance: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PressurePrediction {
    pub schema: String,
    pub sample_count: usize,
    pub p95_next_turn_input_growth: u64,
    pub p95_tool_output_growth: u64,
    pub required_reserve: u64,
    pub safe_context_limit: u64,
    pub predicted_peak: u64,
    pub checkpoint_margin: u64,
    pub checkpoint_at: u64,
}

impl PressureStatistics {
    pub fn observe(&mut self, input_growth: u64, tool_growth: u64, context_window: u64) {
        let winsor_cap = (context_window / 4).max(4_096);
        let input = input_growth.min(winsor_cap);
        let tool = tool_growth.min(winsor_cap);
        push_bounded(&mut self.input_growth, input);
        push_bounded(&mut self.tool_growth, tool);
        self.ewma_input = Some(ewma(self.ewma_input, input));
        self.ewma_tool = Some(ewma(self.ewma_tool, tool));
    }

    pub fn predict(&self, input: &PressurePredictionInput) -> PressurePrediction {
        let p95_input =
            p95(&self.input_growth).max(self.ewma_input.unwrap_or_default().ceil() as u64);
        let p95_tool = p95(&self.tool_growth).max(self.ewma_tool.unwrap_or_default().ceil() as u64);
        let projected_growth = p95_input
            .saturating_add(p95_tool)
            .saturating_add(input.projection_budget_tokens);
        let required_reserve = input
            .configured_reserve_floor
            .max(
                input
                    .context_window
                    .saturating_mul(input.configured_reserve_percent.into())
                    / 100,
            )
            .max(input.max_output_tokens.unwrap_or_default())
            .max(projected_growth.saturating_add(4_096));
        let safe_context_limit = input.context_window.saturating_sub(required_reserve);
        let predicted_peak = input.current_context.saturating_add(projected_growth);
        let checkpoint_margin = 8_192_u64
            .max(p95_input.saturating_add(p95_tool))
            .max(input.projection_budget_tokens)
            .max(input.persistence_growth_allowance);
        PressurePrediction {
            schema: "focusa.compaction_pressure_prediction.v1".into(),
            sample_count: self.input_growth.len().min(self.tool_growth.len()),
            p95_next_turn_input_growth: p95_input,
            p95_tool_output_growth: p95_tool,
            required_reserve,
            safe_context_limit,
            predicted_peak,
            checkpoint_margin,
            checkpoint_at: safe_context_limit.saturating_sub(checkpoint_margin),
        }
    }
}

fn push_bounded(queue: &mut VecDeque<u64>, value: u64) {
    queue.push_back(value);
    while queue.len() > MAX_SAMPLES {
        queue.pop_front();
    }
}

fn ewma(prior: Option<f64>, sample: u64) -> f64 {
    prior.map_or(sample as f64, |prior| 0.2 * sample as f64 + 0.8 * prior)
}

fn p95(values: &VecDeque<u64>) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut values: Vec<_> = values.iter().copied().collect();
    values.sort_unstable();
    let index = ((values.len() - 1) as f64 * 0.95).ceil() as usize;
    values[index]
}
