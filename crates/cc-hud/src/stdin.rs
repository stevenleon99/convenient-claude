use std::io::{self, IsTerminal, Read};
use std::time::{Duration, Instant};

use crate::types::StdinData;

const FIRST_BYTE_TIMEOUT: Duration = Duration::from_millis(250);
const IDLE_TIMEOUT: Duration = Duration::from_millis(30);
const MAX_BYTES: usize = 256 * 1024;

/// Read JSON from stdin with timeout. Returns `Ok(None)` if stdin is a TTY or
/// no data arrives within the timeout (normal when not invoked by Claude Code).
pub fn read_stdin() -> anyhow::Result<Option<StdinData>> {
    // Check if stdin is a TTY — if so, we're not being piped to by Claude Code.
    if io::stdin().is_terminal() {
        return Ok(None);
    }

    let mut raw = String::with_capacity(4096);
    let mut buf = [0u8; 4096];
    let mut stdin = io::stdin();
    let stdin_fd = &mut stdin;

    // Set stdin to non-blocking would require platform-specific code.
    // Instead, we use a thread-based approach: read in a spawned thread with
    // a timeout on the first byte, then read until idle.

    let start = Instant::now();

    // First byte timeout
    loop {
        match stdin_fd.read(&mut buf) {
            Ok(0) => {
                // EOF — we have everything
                return finish_parse(&raw);
            }
            Ok(n) => {
                raw.push_str(&String::from_utf8_lossy(&buf[..n]));
                break;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if start.elapsed() > FIRST_BYTE_TIMEOUT {
                    return Ok(None);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) => return Err(e.into()),
        }
    }

    // Idle timeout — keep reading until we get a gap
    let mut last_data = Instant::now();
    loop {
        match stdin_fd.read(&mut buf) {
            Ok(0) => {
                // EOF
                return finish_parse(&raw);
            }
            Ok(n) => {
                raw.push_str(&String::from_utf8_lossy(&buf[..n]));
                last_data = Instant::now();

                if raw.len() > MAX_BYTES {
                    return Ok(None);
                }

                // Try to parse what we have — if it's valid JSON, we're done
                if let Some(data) = try_parse(&raw) {
                    return Ok(Some(data));
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if last_data.elapsed() > IDLE_TIMEOUT {
                    return finish_parse(&raw);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(e) => return Err(e.into()),
        }
    }
}

fn finish_parse(raw: &str) -> anyhow::Result<Option<StdinData>> {
    Ok(try_parse(raw))
}

fn try_parse(raw: &str) -> Option<StdinData> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

// ---------------------------------------------------------------------------
// Helper functions for extracting data from StdinData (ported from claude-hud)
// ---------------------------------------------------------------------------

/// Get total tokens from context window usage.
pub fn get_total_tokens(stdin: &StdinData) -> u64 {
    let usage = match &stdin.context_window {
        Some(cw) => match &cw.current_usage {
            Some(u) => u,
            None => return 0,
        },
        None => return 0,
    };
    usage.input_tokens.unwrap_or(0)
        + usage.cache_creation_input_tokens.unwrap_or(0)
        + usage.cache_read_input_tokens.unwrap_or(0)
}

/// Get context usage as a percentage (0–100).
/// Prefers native percentage from Claude Code v2.1.6+ when available.
pub fn get_context_percent(stdin: &StdinData) -> u8 {
    // Prefer native percentage
    if let Some(cw) = &stdin.context_window {
        if let Some(pct) = cw.used_percentage {
            if pct > 0.0 && pct.is_finite() {
                return pct.clamp(0.0, 100.0) as u8;
            }
        }
    }

    // Fallback: manual calculation
    let size = match &stdin.context_window {
        Some(cw) => cw.context_window_size.unwrap_or(0),
        None => 0,
    };
    if size == 0 {
        return 0;
    }
    let total = get_total_tokens(stdin);
    ((total as f64 / size as f64) * 100.0).clamp(0.0, 100.0) as u8
}

/// Get the model display name.
pub fn get_model_name(stdin: &StdinData) -> String {
    if let Some(model) = &stdin.model {
        if let Some(name) = &model.display_name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return strip_context_suffix(trimmed.to_string());
            }
        }
        if let Some(id) = &model.id {
            if !id.trim().is_empty() {
                return id.clone();
            }
        }
    }
    "Unknown".to_string()
}

/// Get provider label (Bedrock / Vertex / Enterprise).
pub fn get_provider_label(stdin: &StdinData) -> Option<&str> {
    if std::env::var("CLAUDE_CODE_USE_BEDROCK").as_deref() == Ok("1") {
        return Some("Bedrock");
    }
    if std::env::var("CLAUDE_CODE_USE_VERTEX").as_deref() == Ok("1") {
        return Some("Vertex");
    }
    if let Some(model) = &stdin.model {
        if let Some(id) = &model.id {
            let lower = id.to_lowercase();
            if lower == "opusplan" || lower == "sonnetplan" || lower == "haikuplan" {
                return Some("Enterprise");
            }
        }
    }
    None
}

/// Strip redundant context-window size suffix from model name.
fn strip_context_suffix(name: String) -> String {
    // Remove parenthetical containing "context", e.g. "(1M context)"
    if let Some(idx) = name.rfind('(') {
        let paren = &name[idx..];
        if paren.to_lowercase().contains("context") {
            let before = &name[..idx];
            return before.trim().to_string();
        }
    }
    name
}

/// Extract usage (rate limit) data from stdin.
pub fn get_usage_from_stdin(stdin: &StdinData) -> Option<crate::types::UsageData> {
    let limits = stdin.rate_limits.as_ref()?;

    fn parse_pct(v: Option<f64>) -> Option<f64> {
        v.filter(|&p| p.is_finite()).map(|p| p.clamp(0.0, 100.0).round())
    }

    fn parse_reset(v: Option<f64>) -> Option<chrono::DateTime<chrono::Utc>> {
        v.filter(|&t| t.is_finite() && t > 0.0)
            .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
    }

    let five_hour = parse_pct(limits.five_hour.as_ref().and_then(|w| w.used_percentage));
    let seven_day = parse_pct(limits.seven_day.as_ref().and_then(|w| w.used_percentage));

    if five_hour.is_none() && seven_day.is_none() {
        return None;
    }

    Some(crate::types::UsageData {
        five_hour,
        seven_day,
        five_hour_reset_at: parse_reset(limits.five_hour.as_ref().and_then(|w| w.resets_at)),
        seven_day_reset_at: parse_reset(limits.seven_day.as_ref().and_then(|w| w.resets_at)),
    })
}

/// Format session duration from a start time.
pub fn format_session_duration(session_start: Option<&chrono::DateTime<chrono::Utc>>) -> String {
    let start = match session_start {
        Some(s) => *s,
        None => return String::new(),
    };

    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(start);
    let mins = diff.num_minutes();

    if mins < 1 {
        return "<1m".to_string();
    }
    if mins < 60 {
        return format!("{mins}m");
    }

    let hours = mins / 60;
    let remaining = mins % 60;
    format!("{hours}h {remaining}m")
}
