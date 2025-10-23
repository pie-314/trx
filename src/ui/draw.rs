use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::ui::{app::App, input::InputMode};

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
                let title = format!("{} {}", p.name, p.version);
                let content = Span::raw(format!("{: <30} {}", title, p.description));
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
    let details_text = if app.packages.is_empty() {
        if app.loading {
            "Loading...".to_string()
        } else {
            "No package selected".to_string()
        }
    } else {
        let idx = app.selected.min(app.packages.len().saturating_sub(1));
        let p = &app.packages[idx];
        format!(
            "Name: {}\nVersion: {}\n\n{}",
            p.name, p.version, p.description
        )
    };
    frame.render_widget(
        Paragraph::new(details_text).block(Block::bordered().title("Details")),
        details_area,
    );

    if let InputMode::Editing = app.input_mode {
        frame.set_cursor_position(Position::new(
            input_area.x + app.character_index as u16 + 1,
            input_area.y + 1,
        ));
    }
}
