use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::ui::app::{App, ToastSeverity};
use crate::ui::input::InputMode;

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn get_border_type(style: &str) -> BorderType {
    match style {
        "Plain" => BorderType::Plain,
        "Double" => BorderType::Double,
        "Thick" => BorderType::Thick,
        _ => BorderType::Rounded,
    }
}

fn get_spinner(spinner_type: &str) -> &'static [&'static str] {
    match spinner_type {
        "Bars" => &[" ", "▃", "▄", "▅", "▆", "▇", "▆", "▅", "▄", "▃"],

        "Pulse" => &["█", "▓", "▒", "░", " ", "░", "▒", "▓"],

        "Classic" => &["|", "/", "-", "\\"],

        "Arc" => &["◜", "◠", "◝", "◞", "◡", "◟"],

        "Braille" => &["⠁", "⠂", "⠄", "⠂"],

        _ => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    }
}

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let theme_colors = app.config.current_theme();
    let vertical_root = Layout::vertical([
        Constraint::Length(1), // Help header
        Constraint::Length(3), // Tabs
        Constraint::Min(1),    // Content
        Constraint::Length(1), // Status Bar
    ]);
    let [help_area_root, tabs_area, content_area, status_area] = vertical_root.areas(frame.area());

    draw_help_header(frame, app, help_area_root);
    draw_tabs(frame, app, tabs_area, &theme_colors);

    if let crate::ui::app::Tab::Settings = app.current_tab {
        draw_settings_tab(frame, app, content_area, &theme_colors);
    } else {
        // Responsive layout for content area
        let is_wide = content_area.width >= 100;
        let constraints = if is_wide {
            [Constraint::Percentage(50), Constraint::Percentage(50)]
        } else {
            [Constraint::Percentage(60), Constraint::Percentage(40)]
        };
        let direction = if is_wide { Direction::Horizontal } else { Direction::Vertical };

        let content_layout = Layout::new(direction, constraints).split(content_area);
        let search_area = content_layout[0];
        let details_area = content_layout[1];

        let (input_area, list_area) = if let crate::ui::app::Tab::Search = app.current_tab {
            let vertical_search = Layout::vertical([
                Constraint::Length(3), // Input
                Constraint::Min(1),    // List
            ]);
            let [i, l] = vertical_search.areas(search_area);
            (Some(i), l)
        } else {
            (None, search_area)
        };

        if let Some(i_area) = input_area {
            draw_search_input(frame, app, i_area, &theme_colors);
        }

        draw_package_list(frame, app, list_area, &theme_colors);
        draw_details(frame, app, details_area, &theme_colors);
    }

    draw_status_bar(frame, app, status_area, &theme_colors);

    if app.show_help {
        draw_help_overlay(frame, app, &theme_colors);
    }

    if app.update_prompt.is_some() {
        draw_update_prompt(frame, app, &theme_colors);
    }

    if let Some(toast) = app.toasts.first() {
        let color = match toast.severity {
            ToastSeverity::Success => Color::Green,
            ToastSeverity::Warning => Color::Yellow,
            ToastSeverity::Error => Color::Red,
            ToastSeverity::Info => Color::Cyan,
        };

        draw_popup(frame, &toast.message, color, &theme_colors, &app.config.settings.border_style);
    }
}

fn draw_update_prompt(frame: &mut Frame, app: &App, theme: &crate::config::Theme) {
    let area = centered_rect(40, 20, frame.area());
    frame.render_widget(Clear, area);

    let (version, _) = app.update_prompt.as_ref().unwrap();
    let current_version = env!("CARGO_PKG_VERSION");
    let success_color = app.config.get_color(&theme.success_color);
    let error_color = app.config.get_color(&theme.error_color);
    let border_type = get_border_type(&app.config.settings.border_style);

    let text = vec![
        Line::from(vec![Span::raw(format!("Version {} -> {}", current_version, version))]),
        Line::from(""),
        Line::from("Do you want to update?"),
        Line::from(""),
        Line::from(vec![
            if app.update_selected_yes {
                Span::styled(
                    " [ Yes ] ",
                    Style::default()
                        .bg(success_color)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw(" [ Yes ] ")
            },
            Span::raw("    "),
            if !app.update_selected_yes {
                Span::styled(
                    " [ No ] ",
                    Style::default().bg(error_color).fg(Color::Black).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw(" [ No ] ")
            },
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::bordered()
                .title("Update Available")
                .border_type(border_type)
                .border_style(Style::default().fg(app.config.get_color(&theme.border_color))),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn draw_help_header(frame: &mut Frame, app: &App, area: Rect) {
    let keys = &app.config.keys;
    let (help_lines, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                keys.quit.clone().bold(),
                " to quit, ".into(),
                keys.search_edit.clone().bold(),
                " to edit, ".into(),
                keys.tab_next.clone().bold(),
                "/".into(),
                keys.tab_prev.clone().bold(),
                " to switch tabs, ".into(),
                keys.help.clone().bold(),
                " for help".into(),
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
    frame.render_widget(Paragraph::new(text), area);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect, theme: &crate::config::Theme) {
    let highlight_color = app.config.get_color(&theme.highlight_color);
    let border_type = get_border_type(&app.config.settings.border_style);

    let tab_titles = vec!["Search", "Installed", "Updates", "Settings"];
    let tabs = ratatui::widgets::Tabs::new(tab_titles)
        .block(
            Block::bordered()
                .title("Views")
                .border_style(Style::default().fg(app.config.get_color(&theme.border_color)))
                .border_type(border_type),
        )
        .select(match app.current_tab {
            crate::ui::app::Tab::Search => 0,
            crate::ui::app::Tab::Installed => 1,
            crate::ui::app::Tab::Updates => 2,
            crate::ui::app::Tab::Settings => 3,
        })
        .highlight_style(Style::default().fg(highlight_color).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, area);
}

fn draw_popup(
    frame: &mut Frame,
    msg: &str,
    color: Color,
    _theme: &crate::config::Theme,
    border_style: &str,
) {
    let area = Rect { x: frame.area().width.saturating_sub(35), y: 1, width: 30, height: 3 };
    frame.render_widget(Clear, area);
    let border_type = get_border_type(border_style);

    let block = Block::bordered().border_style(Style::default().fg(color)).border_type(border_type);

    let paragraph =
        Paragraph::new(Span::styled(msg, Style::default().fg(color).add_modifier(Modifier::BOLD)))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn draw_settings_tab(frame: &mut Frame, app: &App, area: Rect, theme: &crate::config::Theme) {
    let highlight_color = app.config.get_color(&theme.highlight_color);
    let border_color = app.config.get_color(&theme.border_color);
    let primary_color = app.config.get_color(&theme.text_primary);
    let secondary_color = app.config.get_color(&theme.text_secondary);
    let border_type = get_border_type(&app.config.settings.border_style);

    let mut settings_lines = Vec::new();

    // Config Path (Not interactive)
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "trx") {
        let config_path = proj_dirs.config_dir().join("config.toml");
        settings_lines.push(Line::from(vec![
            Span::styled(
                "  Config Path: ",
                Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(config_path.to_string_lossy().to_string()),
        ]));
        settings_lines.push(Line::from(""));
    }

    // Helper macro for settings
    macro_rules! draw_setting {
        ($idx:expr, $label:expr, $value:expr, $is_toggle:expr) => {
            let style = if app.settings_index == $idx {
                Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(primary_color)
            };

            let indicator = if app.settings_index == $idx { "> " } else { "  " };
            let val_span = if $is_toggle {
                if $value == "true" || $value == "enabled" {
                    Span::styled(
                        format!("[{}]", $value),
                        Style::default().fg(app.config.get_color(&theme.success_color)),
                    )
                } else {
                    Span::styled(
                        format!("[{}]", $value),
                        Style::default().fg(app.config.get_color(&theme.error_color)),
                    )
                }
            } else {
                let mut val_text = $value.to_string();
                if app.settings_index == $idx && matches!(app.input_mode, InputMode::Editing) {
                    val_text = format!("{}█", app.input);
                }
                Span::styled(val_text, Style::default().fg(secondary_color))
            };

            settings_lines.push(Line::from(vec![
                Span::styled(indicator, style),
                Span::styled(format!("{:<20}: ", $label), style),
                val_span,
            ]));
        };
    }

    // Settings
    settings_lines.push(Line::from(Span::styled(
        "--- General ---",
        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
    )));
    draw_setting!(0, "AUR Helper", &app.config.aur_helper, false);
    draw_setting!(
        1,
        "Auto Update Check",
        if app.config.settings.auto_update_check { "true" } else { "false" },
        true
    );
    draw_setting!(
        2,
        "Auto Cleanup",
        if app.config.settings.auto_cleanup { "true" } else { "false" },
        true
    );
    draw_setting!(
        3,
        "Search Debounce",
        &format!("{}ms", app.config.settings.search_debounce_ms),
        false
    );
    draw_setting!(4, "Default Tab", &app.config.settings.default_tab, false);
    draw_setting!(5, "Max Results", &app.config.settings.max_search_results.to_string(), false);
    draw_setting!(6, "Clear Cache", "Press Enter/Space", false);
    settings_lines.push(Line::from(""));

    let mut current_idx = 7;

    // Managers
    settings_lines.push(Line::from(Span::styled(
        "--- Managers ---",
        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
    )));
    for m in &app.available_managers {
        let enabled = app.config.settings.enabled_managers.contains(m);
        draw_setting!(
            current_idx,
            &format!("Enable {}", m),
            if enabled { "enabled" } else { "disabled" },
            true
        );
        current_idx += 1;
    }
    settings_lines.push(Line::from(""));

    // Theme
    settings_lines.push(Line::from(Span::styled(
        "--- Aesthetics ---",
        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
    )));
    draw_setting!(current_idx, "Theme Preset", &app.config.theme_name, false);
    draw_setting!(current_idx + 1, "Border Style", &app.config.settings.border_style, false);
    draw_setting!(current_idx + 2, "Spinner Type", &app.config.settings.spinner_type, false);
    current_idx += 3;

    if app.config.theme_name == "Custom" {
        settings_lines.push(Line::from(""));
        settings_lines.push(Line::from(Span::styled(
            "--- Custom Colors ---",
            Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
        )));
        if let Some(ref ct) = app.config.custom_theme {
            draw_setting!(current_idx, "Border Color", &ct.border_color, false);
            draw_setting!(current_idx + 1, "Highlight Color", &ct.highlight_color, false);
            draw_setting!(current_idx + 2, "Success Color", &ct.success_color, false);
            draw_setting!(current_idx + 3, "Error Color", &ct.error_color, false);
            draw_setting!(current_idx + 4, "Text Primary", &ct.text_primary, false);
            draw_setting!(current_idx + 5, "Text Secondary", &ct.text_secondary, false);
        }
    }

    let paragraph = Paragraph::new(settings_lines)
        .block(
            Block::bordered()
                .title("Settings (Enter/Space to Toggle or Edit, Arrows for Cycles)")
                .border_type(border_type)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect, theme: &crate::config::Theme) {
    let highlight_color = app.config.get_color(&theme.highlight_color);
    let secondary_color = app.config.get_color(&theme.text_secondary);
    let primary_color = app.config.get_color(&theme.text_primary);

    let mode_str = match app.input_mode {
        InputMode::Normal => " NORMAL ",
        InputMode::Editing => " EDITING ",
    };

    let mode_style = match app.input_mode {
        InputMode::Normal => {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        }
        InputMode::Editing => {
            Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
        }
    };

    let status_line = Line::from(vec![
        Span::styled(mode_str, mode_style),
        Span::raw(" | "),
        Span::styled(
            format!("Selected: {} ", app.selected_names.len()),
            Style::default().fg(highlight_color),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Manager: {} ", app.manager.name()),
            Style::default().fg(secondary_color),
        ),
        Span::raw(" | "),
        Span::styled(
            "Press '?' for help ",
            Style::default().fg(primary_color).add_modifier(Modifier::ITALIC),
        ),
    ]);

    let paragraph =
        Paragraph::new(status_line).style(Style::default().bg(app.config.get_color("black")));
    frame.render_widget(paragraph, area);
}

fn draw_search_input(frame: &mut Frame, app: &App, area: Rect, theme: &crate::config::Theme) {
    let highlight_color = app.config.get_color(&theme.highlight_color);
    let border_color = app.config.get_color(&theme.border_color);
    let border_type = get_border_type(&app.config.settings.border_style);
    let spinners = get_spinner(&app.config.settings.spinner_type);

    let spinner =
        if app.loading { spinners[(app.spinner_tick as usize / 5) % spinners.len()] } else { "" };

    let search_title = format!(" Search {} ", spinner);
    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Editing => Style::default().fg(highlight_color),
            _ => Style::default(),
        })
        .block(
            Block::bordered()
                .title(search_title)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color)),
        );
    frame.render_widget(input, area);

    if let InputMode::Editing = app.input_mode {
        frame.set_cursor_position(Position {
            x: area.x + app.character_index as u16 + 1,
            y: area.y + 1,
        });
    }
}

fn draw_package_list(frame: &mut Frame, app: &mut App, area: Rect, theme: &crate::config::Theme) {
    let border_color = app.config.get_color(&theme.border_color);
    let success_color = app.config.get_color(&theme.success_color);
    let secondary_color = app.config.get_color(&theme.text_secondary);
    let primary_color = app.config.get_color(&theme.text_primary);
    let border_type = get_border_type(&app.config.settings.border_style);
    let spinners = get_spinner(&app.config.settings.spinner_type);

    let items: Vec<ListItem> = if app.packages.is_empty() {
        if app.loading {
            vec![ListItem::new(Line::from(Span::styled(
                "  Searching...",
                Style::default().fg(secondary_color),
            )))]
        } else if !app.input.trim().is_empty()
            || !matches!(app.current_tab, crate::ui::app::Tab::Search)
        {
            vec![ListItem::new(Line::from(Span::styled(
                "  No results found.",
                Style::default().fg(secondary_color).add_modifier(Modifier::ITALIC),
            )))]
        } else {
            vec![ListItem::new(Line::from(Span::styled(
                "  Start typing to search...",
                Style::default().fg(secondary_color).add_modifier(Modifier::ITALIC),
            )))]
        }
    } else {
        app.packages
            .iter()
            .enumerate()
            .map(|(i, pkg)| {
                let is_selected = i == app.selected;
                let is_checked = app.checked[i];
                let is_installed = app.installed_packages.contains(&pkg.name);

                let checkbox = if is_checked { "[x] " } else { "[ ] " };
                let status = if is_installed { " (installed)" } else { "" };

                let name_style = if is_installed {
                    Style::default().fg(success_color)
                } else {
                    Style::default().fg(primary_color)
                };
                // Bold + underline the focused row for extra visibility without
                // duplicating the row-level highlight background set below.
                let name_style = if is_selected {
                    name_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    name_style
                };

                let line = Line::from(vec![
                    Span::styled(checkbox, Style::default().fg(secondary_color)),
                    Span::styled(&pkg.name, name_style),
                    Span::styled(
                        format!(" v{}", pkg.version),
                        Style::default().fg(secondary_color),
                    ),
                    Span::styled(
                        status,
                        Style::default().fg(success_color).add_modifier(Modifier::ITALIC),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let spinner =
        if app.loading { spinners[(app.spinner_tick as usize / 5) % spinners.len()] } else { "" };

    let list_title = if app.loading && !matches!(app.current_tab, crate::ui::app::Tab::Search) {
        format!(" Packages {} ", spinner)
    } else {
        format!(" Packages ({}) ", app.packages.len())
    };

    let highlight_bg = app.config.get_color(&theme.highlight_color);
    let highlight_fg = crate::config::Config::contrast_fg_for(highlight_bg);
    let list = List::new(items)
        .block(
            Block::bordered()
                .title(list_title)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default().bg(highlight_bg).fg(highlight_fg).add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_details(frame: &mut Frame, app: &App, area: Rect, theme: &crate::config::Theme) {
    let highlight_color = app.config.get_color(&theme.highlight_color);
    let border_color = app.config.get_color(&theme.border_color);
    let error_color = app.config.get_color(&theme.error_color);
    let primary_color = app.config.get_color(&theme.text_primary);
    let border_type = get_border_type(&app.config.settings.border_style);
    let spinners = get_spinner(&app.config.settings.spinner_type);

    let mut details_lines: Vec<Line> = Vec::new();
    let spinner = spinners[(app.spinner_tick as usize / 5) % spinners.len()];

    match &app.details_state {
        crate::ui::app::DetailsState::Empty => {
            details_lines.push(Line::from("Select a package to see details"));
        }
        crate::ui::app::DetailsState::Loading => {
            details_lines.push(Line::from(format!("Loading details {}...", spinner)));
        }
        crate::ui::app::DetailsState::Error(err) => {
            details_lines.push(Line::from(Span::styled(err, Style::default().fg(error_color))));
        }
        crate::ui::app::DetailsState::Success(details) => {
            for (key, value) in details {
                details_lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}: ", key),
                        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(value),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(details_lines)
        .scroll((app.details_scroll, 0))
        .style(Style::default().fg(primary_color))
        .block(
            Block::bordered()
                .title("Details")
                .border_type(border_type)
                .border_style(Style::default().fg(border_color)),
        );

    frame.render_widget(paragraph, area);
}

fn draw_help_overlay(frame: &mut Frame, app: &App, theme: &crate::config::Theme) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);
    let keys = &app.config.keys;
    let highlight_color = app.config.get_color(&theme.highlight_color);
    let secondary_color = app.config.get_color(&theme.text_secondary);
    let border_type = get_border_type(&app.config.settings.border_style);
    let version = env!("CARGO_PKG_VERSION");

    let key_style = Style::default().fg(highlight_color).add_modifier(Modifier::BOLD);
    let section_style = Style::default().fg(secondary_color).add_modifier(Modifier::ITALIC);

    // Display Space key as a readable label
    let toggle_display =
        if keys.toggle_select == " " { "Space".to_string() } else { keys.toggle_select.clone() };

    let help_text = vec![
        Line::from(vec![
            Span::styled("trx ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("v{}", version)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Navigation", section_style)),
        Line::from(vec![
            Span::styled(format!("{:<18}", "↑/k  ↓/j"), key_style),
            Span::raw("Move up / down"),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Home / End"), key_style),
            Span::raw("Jump to first / last"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:<18}", format!("{} / {}", keys.tab_next, keys.tab_prev)),
                key_style,
            ),
            Span::raw("Next / previous tab"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Search", section_style)),
        Line::from(vec![
            Span::styled(format!("{:<18}", &keys.search_edit), key_style),
            Span::raw("Enter search mode"),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Esc"), key_style),
            Span::raw("Exit search / close overlay"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Packages", section_style)),
        Line::from(vec![
            Span::styled(format!("{:<18}", toggle_display), key_style),
            Span::raw("Toggle package selection (also toggles settings on the Settings tab)"),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", &keys.install), key_style),
            Span::raw("Install selected packages"),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", &keys.remove), key_style),
            Span::raw("Remove selected packages"),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", &keys.system_upgrade), key_style),
            Span::raw("Full system upgrade"),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", &keys.refresh_db), key_style),
            Span::raw("Refresh package databases"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Details Panel", section_style)),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Mouse scroll"), key_style),
            Span::raw("Scroll package details (right pane)"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Other", section_style)),
        Line::from(vec![
            Span::styled(format!("{:<18}", &keys.help), key_style),
            Span::raw("Toggle this help overlay"),
        ]),
        Line::from(vec![Span::styled(format!("{:<18}", &keys.quit), key_style), Span::raw("Quit")]),
        Line::from(""),
        Line::from(Span::styled(
            "Mouse: click tabs · scroll list/details · click checkboxes",
            section_style,
        )),
    ];

    frame.render_widget(
        Paragraph::new(help_text)
            .block(
                Block::bordered()
                    .title(" Help — Keybindings ")
                    .border_type(border_type)
                    .border_style(Style::default().fg(app.config.get_color(&theme.border_color))),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

