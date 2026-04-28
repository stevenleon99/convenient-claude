use owo_colors::{AnsiColors, OwoColorize, Stream::Stdout};

/// ANSI reset sequence.
pub const RESET: &str = "\x1b[0m";

/// Color a string with a specific ANSI color.
pub fn colored(s: &str, color: AnsiColors) -> String {
    s.if_supports_color(Stdout, |s| s.color(color)).to_string()
}

/// Green text.
pub fn green(s: &str) -> String {
    colored(s, AnsiColors::Green)
}

/// Yellow text.
pub fn yellow(s: &str) -> String {
    colored(s, AnsiColors::Yellow)
}

/// Red text.
pub fn red(s: &str) -> String {
    colored(s, AnsiColors::Red)
}

/// Cyan text.
pub fn cyan(s: &str) -> String {
    colored(s, AnsiColors::Cyan)
}

/// Dim/faint text.
pub fn dim(s: &str) -> String {
    s.if_supports_color(Stdout, |s| s.dimmed()).to_string()
}

/// Bold text.
pub fn bold(s: &str) -> String {
    s.if_supports_color(Stdout, |s| s.bold()).to_string()
}

/// Magenta text.
pub fn magenta(s: &str) -> String {
    colored(s, AnsiColors::Magenta)
}

/// Blue text.
pub fn blue(s: &str) -> String {
    colored(s, AnsiColors::Blue)
}

/// White text.
pub fn white(s: &str) -> String {
    colored(s, AnsiColors::White)
}

/// Render a colored progress bar.
/// `percent` is 0–100, `width` is the number of characters.
pub fn colored_bar(percent: u8, width: usize) -> String {
    let filled = ((percent as usize) * width) / 100;
    let empty = width - filled;

    let filled_color = if percent >= 85 {
        AnsiColors::Red
    } else if percent >= 60 {
        AnsiColors::Yellow
    } else {
        AnsiColors::Green
    };

    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);

    format!(
        "{}{}{}",
        colored(&filled_str, filled_color),
        dim(&empty_str),
        RESET
    )
}

/// Get context color based on percentage threshold.
pub fn context_color(percent: u8) -> AnsiColors {
    if percent >= 85 {
        AnsiColors::Red
    } else if percent >= 60 {
        AnsiColors::Yellow
    } else {
        AnsiColors::Green
    }
}

/// Get quota color based on percentage threshold.
pub fn quota_color(percent: f64) -> AnsiColors {
    if percent >= 100.0 {
        AnsiColors::Red
    } else if percent >= 80.0 {
        AnsiColors::Yellow
    } else {
        AnsiColors::Green
    }
}

/// Label style (dim cyan).
pub fn label(s: &str) -> String {
    cyan(&dim(s))
}

/// Model badge style (cyan).
pub fn model(s: &str) -> String {
    cyan(s)
}

/// Project path style (magenta).
pub fn project(s: &str) -> String {
    magenta(s)
}

/// Git branch style (yellow).
pub fn git_branch(s: &str) -> String {
    yellow(s)
}

/// Git prefix/suffix style (dim).
pub fn git_style(s: &str) -> String {
    dim(s)
}

/// Critical/warning style (red bold).
pub fn critical(s: &str) -> String {
    red(&bold(s))
}