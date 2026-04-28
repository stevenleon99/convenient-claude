use crate::render::colors::*;
use crate::render::terminal::format_tokens;
use crate::stdin::{format_session_duration, get_context_percent, get_model_name, get_provider_label};
use crate::types::RenderContext;

/// Render the main session line: model, context bar, project, git, usage, duration.
pub fn render(ctx: &RenderContext) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let cfg = &ctx.config;

    // Model + context bar
    let model_name = get_model_name(&ctx.stdin);
    let percent = get_context_percent(&ctx.stdin);
    let bar_width = cfg.context_bar_width;
    let bar = colored_bar(percent, bar_width);
    let pct_color = context_color(percent);
    let pct_str = format!("{}%{}", colored(&percent.to_string(), pct_color), RESET);

    let mut model_display = model_name.clone();
    if let Some(provider) = get_provider_label(&ctx.stdin) {
        model_display = format!("{} | {}", model_name, provider);
    }

    if cfg.show_model && cfg.show_context_bar {
        parts.push(format!(
            "{} {} {}",
            model(&format!("[{}]", model_display)),
            bar,
            pct_str
        ));
    } else if cfg.show_model {
        parts.push(format!(
            "{} {}",
            model(&format!("[{}]", model_display)),
            pct_str
        ));
    } else if cfg.show_context_bar {
        parts.push(format!("{} {}", bar, pct_str));
    }

    // Project + git
    let mut project_git_parts: Vec<String> = Vec::new();

    if cfg.show_project {
        if let Some(cwd) = &ctx.stdin.cwd {
            let segments: Vec<&str> = cwd.split(&['/', '\\'][..]).filter(|s| !s.is_empty()).collect();
            let path_levels = cfg.path_levels;
            let project_path = if segments.len() >= path_levels {
                segments[segments.len() - path_levels..].join("/")
            } else {
                segments.join("/")
            };
            project_git_parts.push(project(&project_path));
        }
    }

    if cfg.show_git {
        if let Some(gs) = &ctx.git_status {
            let mut git_parts = vec![gs.branch.clone()];
            if gs.is_dirty {
                git_parts.push("*".to_string());
            }
            if gs.ahead > 0 {
                git_parts.push(format!(" ↑{}", gs.ahead));
            }
            if gs.behind > 0 {
                git_parts.push(format!(" ↓{}", gs.behind));
            }
            let git_str = format!(
                "{}{}{}",
                git_style("git:("),
                git_branch(&git_parts.join("")),
                git_style(")")
            );
            project_git_parts.push(git_str);
        }
    }

    if !project_git_parts.is_empty() {
        parts.push(project_git_parts.join(" "));
    }

    // Config counts
    if cfg.show_config_counts {
        let total = ctx.claude_md_count + ctx.rules_count + ctx.mcp_count + ctx.hooks_count;
        if total > 0 {
            let mut count_parts: Vec<String> = Vec::new();
            if ctx.claude_md_count > 0 {
                count_parts.push(label(&format!("{} CLAUDE.md", ctx.claude_md_count)));
            }
            if ctx.rules_count > 0 {
                count_parts.push(label(&format!("{} rules", ctx.rules_count)));
            }
            if ctx.mcp_count > 0 {
                count_parts.push(label(&format!("{} MCPs", ctx.mcp_count)));
            }
            if ctx.hooks_count > 0 {
                count_parts.push(label(&format!("{} hooks", ctx.hooks_count)));
            }
            if !count_parts.is_empty() {
                parts.push(count_parts.join(" "));
            }
        }
    }

    // Usage rate limits
    if cfg.show_usage {
        let usage = &ctx.usage_data;
        let five_hour = usage.five_hour.unwrap_or(0.0);
        let seven_day = usage.seven_day.unwrap_or(0.0);
        let effective = five_hour.max(seven_day);

        if effective > 0.0 {
            if usage.is_limit_reached() {
                parts.push(critical("⚠ Limit reached"));
            } else {
                let five_hour_pct = usage.five_hour.map(|p| p as u8);
                let seven_day_pct = usage.seven_day.map(|p| p as u8);

                let mut usage_parts: Vec<String> = Vec::new();

                if let Some(five) = five_hour_pct {
                    let color = quota_color(five as f64);
                    let bar = colored_bar(five, bar_width);
                    usage_parts.push(format!(
                        "{} {}{}{} (5h)",
                        label("Usage"),
                        bar,
                        colored(&format!(" {}%", five), color),
                        RESET
                    ));
                }

                if let Some(seven) = seven_day_pct {
                    if seven >= 80 {
                        let color = quota_color(seven as f64);
                        let bar = colored_bar(seven, bar_width);
                        usage_parts.push(format!(
                            "{} {}{}{} (7d)",
                            label("Weekly"),
                            bar,
                            colored(&format!(" {}%", seven), color),
                            RESET
                        ));
                    }
                }

                if !usage_parts.is_empty() {
                    parts.push(usage_parts.join(" | "));
                }
            }
        }
    }

    // Session duration
    if cfg.show_duration {
        let duration = format_session_duration(ctx.transcript.session_start.as_ref());
        if !duration.is_empty() {
            parts.push(label(&format!("⏱ {}", duration)));
        }
    }

    // Cost
    if cfg.show_cost {
        if let Some(cost) = &ctx.stdin.cost {
            if let Some(usd) = cost.total_cost_usd {
                if usd > 0.0 {
                    parts.push(label(&format!("${:.2}", usd)));
                }
            }
        }
    }

    // Session tokens
    if cfg.show_session_tokens {
        if let Some(tokens) = &ctx.transcript.session_tokens {
            let total = tokens.total();
            if total > 0 {
                parts.push(label(&format!(
                    "tok: {} (in: {}, out: {})",
                    format_tokens(total),
                    format_tokens(tokens.input_tokens),
                    format_tokens(tokens.output_tokens)
                )));
            }
        }
    }

    // Token breakdown at high context
    if percent >= 85 {
        if let Some(cw) = &ctx.stdin.context_window {
            if let Some(usage) = &cw.current_usage {
                let input = format_tokens(usage.input_tokens.unwrap_or(0));
                let cache = format_tokens(
                    usage.cache_creation_input_tokens.unwrap_or(0)
                        + usage.cache_read_input_tokens.unwrap_or(0),
                );
                parts.push(label(&format!("(in: {}, cache: {})", input, cache)));
            }
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(" │ "))
}