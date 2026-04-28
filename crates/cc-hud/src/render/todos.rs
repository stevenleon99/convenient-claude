use crate::render::colors::*;
use crate::types::{RenderContext, TodoStatus};

/// Render the todos/progress line.
pub fn render(ctx: &RenderContext) -> Option<String> {
    if ctx.transcript.todos.is_empty() {
        return None;
    }

    let todos = &ctx.transcript.todos;
    let completed = todos.iter().filter(|t| t.status == TodoStatus::Completed).count();
    let _in_progress = todos.iter().filter(|t| t.status == TodoStatus::InProgress).count();
    let total = todos.len();

    // Find the first in-progress or pending task
    let current = todos
        .iter()
        .find(|t| t.status == TodoStatus::InProgress)
        .or_else(|| todos.iter().find(|t| t.status == TodoStatus::Pending));

    let progress = label(&format!("▸ {}/{}", completed, total));

    if let Some(task) = current {
        let desc = if task.content.len() > 60 {
            format!("{}...", &task.content[..57])
        } else {
            task.content.clone()
        };
        Some(format!("{} {}", progress, dim(&desc)))
    } else {
        Some(progress)
    }
}