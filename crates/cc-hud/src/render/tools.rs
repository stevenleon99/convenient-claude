use crate::render::colors::*;
use crate::types::{RenderContext, ToolStatus};

/// Render the tools activity line.
pub fn render(ctx: &RenderContext) -> Option<String> {
    if ctx.transcript.tools.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    for tool in &ctx.transcript.tools {
        let icon = match tool.status {
            ToolStatus::Running => "◐",
            ToolStatus::Completed => "✓",
            ToolStatus::Error => "✗",
        };

        let color_fn = match tool.status {
            ToolStatus::Running => yellow,
            ToolStatus::Completed => green,
            ToolStatus::Error => red,
        };

        let mut display = tool.name.clone();
        if let Some(target) = &tool.target {
            // Shorten file paths
            let short = shorten_path(target);
            display = format!("{}: {}", tool.name, short);
        }

        parts.push(format!("{} {}", color_fn(icon), dim(&display)));
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(" | "))
}

fn shorten_path(path: &str) -> String {
    // Take just the filename from a full path
    let path = std::path::Path::new(path);
    match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => path.to_string_lossy().to_string(),
    }
}