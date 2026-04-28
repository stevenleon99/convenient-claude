/// Get terminal width from environment or fallback.
pub fn get_terminal_width() -> usize {
    // Try COLUMNS env var first
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(n) = cols.parse::<usize>() {
            if n > 4 {
                return n - 4; // Leave some margin
            }
        }
    }

    // Default fallback
    120
}

/// Calculate visual length of a string (strips ANSI codes).
pub fn visual_length(s: &str) -> usize {
    // Strip ANSI escape sequences
    let stripped = strip_ansi(s);

    // Count graphemes (handle wide chars)
    // Simplified: just count chars, assuming most are single-width
    // For proper wide char handling, use unicode-width crate
    stripped.chars().count()
}

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    // ANSI escape sequence: ESC [ ... m or ESC ] ... (BEL or ESC \)
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Start of escape sequence
            match chars.peek() {
                Some('[') => {
                    chars.next(); // consume '['
                    // CSI sequence: read until final byte (0x40-0x7E)
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if (next as u8) >= 0x40 && (next as u8) <= 0x7E {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next(); // consume ']'
                    // OSC sequence: read until BEL (0x07) or ST (ESC \)
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\x07' {
                            break;
                        }
                        if next == '\x1b'
                            && chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                    }
                }
                _ => {}
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Wrap a line to fit within a maximum width, splitting at ` | ` separators.
pub fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 || visual_length(line) <= max_width {
        return vec![line.to_string()];
    }

    // Split by separators
    let parts: Vec<&str> = line.split(" | ").collect();
    if parts.len() <= 1 {
        return vec![truncate_to_width(line, max_width)];
    }

    let mut wrapped: Vec<String> = Vec::new();
    let mut current = String::new();

    for part in parts {
        let candidate = if current.is_empty() {
            part.to_string()
        } else {
            format!("{} | {}", current, part)
        };

        if visual_length(&candidate) <= max_width {
            current = candidate;
        } else {
            if !current.is_empty() {
                wrapped.push(current);
            }
            current = part.to_string();
        }
    }

    if !current.is_empty() {
        wrapped.push(current);
    }

    wrapped
}

/// Truncate a string to fit within a maximum width, adding "..." if truncated.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width == 0 || visual_length(s) <= max_width {
        return s.to_string();
    }

    let suffix = if max_width >= 3 { "..." } else { "." };
    let keep = max_width - suffix.len();

    let stripped = strip_ansi(s);
    let truncated = stripped.chars().take(keep).collect::<String>();
    format!("{}{}", truncated, suffix)
}

/// Format token count (e.g., 1234 → "1.2k", 1234567 → "1.2M").
pub fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}