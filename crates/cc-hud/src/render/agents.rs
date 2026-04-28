use crate::render::colors::*;
use crate::types::{RenderContext, ToolStatus};

/// Render the agents activity line.
pub fn render(ctx: &RenderContext) -> Option<String> {
    if ctx.transcript.agents.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    for agent in &ctx.transcript.agents {
        let icon = match agent.status {
            ToolStatus::Running => "◐",
            ToolStatus::Completed => "✓",
            ToolStatus::Error => "✗",
        };

        let color_fn = match agent.status {
            ToolStatus::Running => yellow,
            ToolStatus::Completed => green,
            ToolStatus::Error => red,
        };

        let mut display = agent.agent_type.clone();
        if let Some(model) = &agent.model {
            display = format!("{} [{}]", agent.agent_type, model);
        }

        if let Some(desc) = &agent.description {
            // Truncate long descriptions
            let short = if desc.len() > 40 {
                format!("{}...", &desc[..37])
            } else {
                desc.clone()
            };
            display = format!("{}: {}", display, short);
        }

        // Add elapsed time for running agents
        if agent.status == ToolStatus::Running {
            let elapsed = (chrono::Utc::now() - agent.start_time)
                .num_seconds();
            let time_str = if elapsed < 60 {
                format!("{}s", elapsed)
            } else {
                format!("{}m {}s", elapsed / 60, elapsed % 60)
            };
            display = format!("{} ({})", display, time_str);
        }

        parts.push(format!("{} {}", color_fn(icon), dim(&display)));
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(" | "))
}