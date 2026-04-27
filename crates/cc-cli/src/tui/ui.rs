use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Tabs, Wrap,
    },
    Frame,
};

use cc_schema::{Origin, ResourceType};

use super::app::App;

/// Render the full TUI frame.
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header + tabs
            Constraint::Min(5),   // main table
            Constraint::Length(if app.show_detail { 10 } else { 0 }), // detail panel
            Constraint::Length(2), // status bar
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_table(f, app, chunks[1]);

    if app.show_detail {
        render_detail(f, app, chunks[2]);
    }

    render_status(f, app, chunks[3]);

    // Confirmation dialog overlay
    if app.confirm_quit {
        render_confirm_dialog(f, app);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let tab_names: Vec<Line> = App::tab_names()
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == app.current_tab_index() {
                Line::from(Span::styled(
                    format!(" {name} "),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {name} "),
                    Style::default().fg(Color::DarkGray),
                ))
            }
        })
        .collect();

    let title = Line::from(vec![
        Span::styled(
            " cc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ]);

    let tabs = Tabs::new(tab_names)
        .block(Block::default().borders(Borders::BOTTOM).title(title))
        .select(app.current_tab_index())
        .highlight_style(Style::default().fg(Color::Yellow).bold());

    f.render_widget(tabs, area);
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let is_plugin_tab = app.resource_type == ResourceType::Plugin;

    let header = if is_plugin_tab {
        Row::new(vec![
            Cell::from(Span::styled("Sel", Style::default().bold())),
            Cell::from(Span::styled("Name", Style::default().bold())),
            Cell::from(Span::styled("Source", Style::default().bold())),
            Cell::from(Span::styled("Description", Style::default().bold())),
        ])
        .style(Style::default().fg(Color::White))
        .bottom_margin(1)
    } else {
        Row::new(vec![
            Cell::from(Span::styled("Sel", Style::default().bold())),
            Cell::from(Span::styled("Name", Style::default().bold())),
            Cell::from(Span::styled("Source", Style::default().bold())),
            Cell::from(Span::styled("In Project", Style::default().bold())),
        ])
        .style(Style::default().fg(Color::White))
        .bottom_margin(1)
    };

    let rows: Vec<Row> = app
        .resources
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected_marker = if app.is_selected(i) { "[*]" } else { "[ ]" };
            let selected_style = if app.is_selected(i) {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Source: for plugins use registry label, otherwise derive from origin
            let source = if is_plugin_tab {
                entry.registry.as_deref().unwrap_or("plugin").to_string()
            } else {
                match &entry.origin {
                    Origin::External { library } => library.clone(),
                    Origin::User => "~/.claude".to_string(),
                    Origin::Project => "project".to_string(),
                    Origin::Session => "session".to_string(),
                }
            };

            if is_plugin_tab {
                let desc = entry.description.as_deref().unwrap_or("");
                Row::new(vec![
                    Cell::from(Span::styled(selected_marker, selected_style)),
                    Cell::from(entry.name.as_str()),
                    Cell::from(Span::styled(source, Style::default().fg(Color::Blue))),
                    Cell::from(Span::styled(desc.to_string(), Style::default().fg(Color::DarkGray))),
                ])
            } else {
                let (in_proj_marker, in_proj_style) = if app.is_in_project(&entry.name) {
                    ("yes", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                } else {
                    ("no", Style::default().fg(Color::DarkGray))
                };

                Row::new(vec![
                    Cell::from(Span::styled(selected_marker, selected_style)),
                    Cell::from(entry.name.as_str()),
                    Cell::from(Span::styled(source, Style::default().fg(Color::Blue))),
                    Cell::from(Span::styled(in_proj_marker, in_proj_style)),
                ])
            }
        })
        .collect();

    let resource_label = app.resource_type.to_string();
    let count = app.resources.len();

    let constraints = if is_plugin_tab {
        vec![
            Constraint::Length(4),       // Sel
            Constraint::Length(20),      // Name
            Constraint::Length(20),      // Source
            Constraint::Min(10),         // Description
        ]
    } else {
        vec![
            Constraint::Length(4),       // Sel
            Constraint::Percentage(35),  // Name
            Constraint::Percentage(35),  // Source
            Constraint::Length(12),      // In Project
        ]
    };

    let table = ratatui::widgets::Table::new(rows, constraints)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(format!(" {resource_label}s ({count}) "))
                .title_style(Style::default().fg(Color::White)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(Clear, area);

    let detail_text = app
        .snapshot
        .as_deref()
        .unwrap_or("No resource selected");

    let paragraph = Paragraph::new(detail_text)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Resource Detail ")
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let selected_info = if app.total_selected_count() > 0 {
        format!(" [{} selected]", app.total_selected_count())
    } else {
        String::new()
    };

    let help = Line::from(vec![
        Span::styled(" ↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Nav "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Detail "),
        Span::styled("Spc", Style::default().fg(Color::Yellow)),
        Span::raw(" Select "),
        Span::styled("a", Style::default().fg(Color::Yellow)),
        Span::raw(" All "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(" Clear "),
        Span::styled("i", Style::default().fg(Color::Yellow)),
        Span::raw(" Install "),
        Span::styled("I", Style::default().fg(Color::Yellow)),
        Span::raw(" InstallAll "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(" Refresh "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
        Span::styled(selected_info, Style::default().fg(Color::Cyan)),
    ]);

    let status = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP).border_style(
            Style::default().fg(Color::DarkGray),
        ));

    f.render_widget(status, area);

    // Render status message in the right portion
    if !app.status_message.is_empty() {
        let msg_area = area.inner(Margin {
            vertical: 0,
            horizontal: area.width.saturating_sub(40) / 2,
        });
        let msg = Paragraph::new(app.status_message.as_str())
            .style(Style::default().fg(Color::Green));
        f.render_widget(msg, msg_area);
    }
}

fn render_confirm_dialog(f: &mut Frame, app: &App) {
    let total = app.total_selected_count();
    let dialog_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(" You have {total} resource(s) selected.",),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw(" Install them before quitting?")),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Install & quit   "),
            Span::styled(" [n]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit without install   "),
            Span::styled(" [c]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]),
    ];

    let dialog_height = 7u16;
    let dialog_width = 52u16;
    let area = centered_rect(dialog_width, dialog_height, f.area());

    f.render_widget(Clear, area);

    let dialog = Paragraph::new(dialog_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Confirm ")
                .title_style(Style::default().fg(Color::Cyan).bold()),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(dialog, area);
}

/// Create a centered rectangle of the given width/height within `r`.
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.width.saturating_sub(width) / 2;
    let y = r.height.saturating_sub(height) / 2;
    Rect::new(
        r.x + x,
        r.y + y,
        width.min(r.width),
        height.min(r.height),
    )
}
