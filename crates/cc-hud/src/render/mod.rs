mod colors;
mod session;
mod tools;
mod agents;
mod todos;
mod terminal;

use crate::types::RenderContext;
pub use colors::*;
pub use terminal::*;

/// Render the HUD output lines.
pub fn render(ctx: &RenderContext) -> Vec<String> {
    let mut lines = Vec::new();

    match ctx.config.layout {
        crate::types::Layout::Compact => {
            // Single line with everything
            if let Some(line) = render_session_line(ctx) {
                lines.push(line);
            }
        }
        crate::types::Layout::Expanded => {
            // Multi-line layout
            if let Some(line) = render_session_line(ctx) {
                lines.push(line);
            }

            // Tools line
            if ctx.config.show_tools {
                if let Some(line) = render_tools_line(ctx) {
                    lines.push(line);
                }
            }

            // Agents line
            if ctx.config.show_agents {
                if let Some(line) = render_agents_line(ctx) {
                    lines.push(line);
                }
            }

            // Todos line
            if ctx.config.show_todos {
                if let Some(line) = render_todos_line(ctx) {
                    lines.push(line);
                }
            }
        }
    }

    // Wrap lines to terminal width
    let width = get_terminal_width();
    lines
        .into_iter()
        .flat_map(|line| wrap_line(&line, width))
        .collect()
}

fn render_session_line(ctx: &RenderContext) -> Option<String> {
    session::render(ctx)
}

fn render_tools_line(ctx: &RenderContext) -> Option<String> {
    tools::render(ctx)
}

fn render_agents_line(ctx: &RenderContext) -> Option<String> {
    agents::render(ctx)
}

fn render_todos_line(ctx: &RenderContext) -> Option<String> {
    todos::render(ctx)
}