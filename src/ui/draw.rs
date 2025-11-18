use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph, Wrap},
};

use crate::ui::{app::App, input::InputMode};
use textwrap::wrap;

use crate::managers::details_package;
/// draw_ui updated to accept a mutable App reference so it can use App.list_state.
/// The important change: use render_stateful_widget with app.list_state so ratatui keeps the
/// selected item visible (scrolls) and can apply highlight styling.
pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [search_area, details_area] = horizontal.areas(frame.area());

    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ]);
    let [help_area, input_area, list_area] = vertical.areas(search_area);

    let (help_lines, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to quit, ".into(),
                "e".bold(),
                " to edit".into(),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to submit".into(),
            ],
            Style::default(),
        ),
    };

    let text = Text::from(Line::from(help_lines)).patch_style(style);
    frame.render_widget(Paragraph::new(text), help_area);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::bordered().title("Search"));
    frame.render_widget(input, input_area);

    // Build items (use packages if available; otherwise, fallback to messages)

    let items: Vec<ListItem> = if app.packages.is_empty() {
        app.messages
            .iter()
            .enumerate()
            .map(|(i, m)| ListItem::new(Line::from(Span::raw(format!("{i}: {m}")))))
            .collect()
    } else {
        app.packages
            .iter()
            .enumerate()
            .map(|(_i, p)| {
                // package name and provider
                let package: Vec<&str> = p.name.split('/').collect();
                let pkg_name = if package.get(1).unwrap_or(&"").len() > 24 {
                    format!("{}...", &package.get(1).unwrap_or(&"")[..22])
                } else {
                    package.get(1).unwrap_or(&"").to_string()
                };
                let provider = package.get(0).unwrap_or(&"");

                // version formatting
                let version = if p.version.len() > 12 {
                    format!("{}...", &p.version[..8])
                } else {
                    p.version.clone()
                };

                let checked_symbol = if app.selected_names.contains(&p.name) {
                    "[*]"
                } else {
                    "[ ]"
                };

                let content = Span::raw(format!(
                    "{} {: <28} {: <20} {}",
                    checked_symbol, pkg_name, version, provider
                ));

                ListItem::new(Line::from(content))
            })
            .collect()
    };

    // Create a List with a highlight style and symbol
    let list = List::new(items)
        .block(Block::bordered().title("Packages"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("» ");

    // Render it statefully so ratatui can scroll to the selected item automatically.
    frame.render_stateful_widget(list, list_area, &mut app.list_state);
    // details pane: show information for selected package
    let mut details_lines: Vec<Line> = Vec::new();

    // If no packages found
    if app.packages.is_empty() {
        details_lines.push(Line::from("No package selected"));
    } else {
        // Load details only if selection changed
        if app.selected != app.last_selected {
            let pkg_name = &app.packages[app.selected].name;
            app.details = details_package(pkg_name);
            app.last_selected = app.selected;
        }

        if let Some(ref info) = app.details {
            let mut sorted: Vec<_> = info.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);

            let key_width = 15; // fixed width for keys

            for (key, value) in sorted {
                let key_text = format!("{:<key_width$}: ", key, key_width = key_width);
                let indent = " ".repeat(key_text.len());

                let value_wrapped = wrap(value, 80 - key_text.len());

                if let Some(first) = value_wrapped.get(0) {
                    details_lines.push(Line::from(vec![
                        Span::styled(
                            key_text.clone(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(first.to_string()),
                    ]));
                }

                // Remaining lines => indent + rest of value
                for line in value_wrapped.iter().skip(1) {
                    details_lines.push(Line::from(format!("{}{}", indent, line)));
                }
            }
        } else {
            details_lines.push(Line::from("Loading details..."));
        }
    }

    // Now render
    frame.render_widget(
        Paragraph::new(details_lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title("Details")),
        details_area,
    );

    if let InputMode::Editing = app.input_mode {
        frame.set_cursor_position(Position::new(
            input_area.x + app.character_index as u16 + 1,
            input_area.y + 1,
        ));
    }
}
