use crate::app::{App, ConnectionState, Tab};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    draw_tabs(frame, chunks[0], app);

    match app.tab {
        Tab::Connect => draw_connect(frame, chunks[1], app),
        Tab::Settings => draw_settings(frame, chunks[1], app),
        Tab::About => draw_about(frame, chunks[1]),
    }
}

fn tab_style(active: bool) -> Style {
    if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn draw_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let tabs_line = Line::from(vec![
        Span::raw(" "),
        Span::styled(" F1 Connect ", tab_style(app.tab == Tab::Connect)),
        Span::raw("  "),
        Span::styled(" F2 Settings ", tab_style(app.tab == Tab::Settings)),
        Span::raw("  "),
        Span::styled(" F3 About ", tab_style(app.tab == Tab::About)),
    ]);

    let tabs_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " openconnect-tui ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

    let tabs = Paragraph::new(tabs_line).block(tabs_block);
    frame.render_widget(tabs, area);
}

fn draw_connect(frame: &mut Frame, area: Rect, app: &App) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = outer.inner(area);
    frame.render_widget(outer, area);

    // Center content vertically
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(1), // server
            Constraint::Length(1),
            Constraint::Length(1), // status
            Constraint::Length(1),
            Constraint::Length(1), // button hint
            Constraint::Percentage(25),
        ])
        .split(inner_area);

    // Server name
    let server_text = Paragraph::new(app.config.vpn_server.as_str())
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(server_text, vert[1]);

    // Status line
    let (status_str, status_color) = match &app.connection {
        ConnectionState::Disconnected => ("○  Disconnected".to_string(), Color::DarkGray),
        ConnectionState::Connecting => (
            format!("{}  Connecting...", SPINNER[app.spinner_frame]),
            Color::Yellow,
        ),
        ConnectionState::Connected => ("●  Connected".to_string(), Color::Green),
        ConnectionState::Error(e) => (format!("✗  Error: {e}"), Color::Red),
    };

    let status = Paragraph::new(status_str)
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Center);
    frame.render_widget(status, vert[3]);

    // Action hint
    let hint = match &app.connection {
        ConnectionState::Disconnected | ConnectionState::Error(_) => {
            Span::styled("[ Enter: Connect ]", Style::default().fg(Color::Cyan))
        }
        ConnectionState::Connecting => {
            Span::styled("[ Enter: Cancel ]  [ q: Quit ]", Style::default().fg(Color::Gray))
        }
        ConnectionState::Connected => {
            Span::styled("[ Enter: Disconnect ]", Style::default().fg(Color::Red))
        }
    };

    let hint_para = Paragraph::new(Line::from(hint)).alignment(Alignment::Center);
    frame.render_widget(hint_para, vert[5]);
}

fn draw_settings(frame: &mut Frame, area: Rect, app: &App) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = outer.inner(area);
    frame.render_widget(outer, area);

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Length(3), // server field
            Constraint::Length(1),
            Constraint::Length(3), // browser field
            Constraint::Length(1),
            Constraint::Length(3), // csd wrapper field
            Constraint::Length(1),
            Constraint::Length(1), // hint
            Constraint::Min(0),
            Constraint::Length(1), // log path hint
        ])
        .split(inner_area);

    // Centered horizontal layout for fields
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ]);

    let server_area = horiz.split(vert[1])[1];
    let browser_area = horiz.split(vert[3])[1];
    let csd_area = horiz.split(vert[5])[1];
    let hint_area = horiz.split(vert[7])[1];

    let server_focused = app.settings_field == 0;
    let browser_focused = app.settings_field == 1;
    let csd_focused = app.settings_field == 2;

    let field_color = |focused: bool| if focused { Color::Cyan } else { Color::DarkGray };

    let server_block = Block::default()
        .title(Span::styled(" VPN Server ", Style::default().fg(Color::White)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(field_color(server_focused)));

    let browser_block = Block::default()
        .title(Span::styled(" Browser ", Style::default().fg(Color::White)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(field_color(browser_focused)));

    let csd_block = Block::default()
        .title(Span::styled(" HIP Report Script (--csd-wrapper) ", Style::default().fg(Color::White)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(field_color(csd_focused)));

    frame.render_widget(
        Paragraph::new(app.settings_server.as_str())
            .style(Style::default().fg(Color::White))
            .block(server_block),
        server_area,
    );
    frame.render_widget(
        Paragraph::new(app.settings_browser.as_str())
            .style(Style::default().fg(Color::White))
            .block(browser_block),
        browser_area,
    );
    frame.render_widget(
        Paragraph::new(app.settings_csd_wrapper.as_str())
            .style(Style::default().fg(Color::White))
            .block(csd_block),
        csd_area,
    );

    let hint = Paragraph::new("Tab: switch field  |  Enter: save  |  Backspace: delete")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, hint_area);

    // Log path hint at the very bottom
    let log_path = crate::vpn::log_path();
    let home = dirs::home_dir().unwrap_or_default();
    let display = log_path
        .strip_prefix(&home)
        .map(|p| format!("~/{}", p.display()))
        .unwrap_or_else(|_| log_path.display().to_string());
    let log_hint = Paragraph::new(format!("logs: tail -f {display}"))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(log_hint, vert[8]);
}

fn draw_about(frame: &mut Frame, area: Rect) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = outer.inner(area);
    frame.render_widget(outer, area);

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Length(1), // title
            Constraint::Length(1), // version
            Constraint::Length(1),
            Constraint::Length(1), // description
            Constraint::Length(1),
            Constraint::Length(1), // github
            Constraint::Length(1),
            Constraint::Length(1), // section header
            Constraint::Length(1),
            Constraint::Length(1), // sudo step 1
            Constraint::Length(1), // sudo step 2
            Constraint::Length(1), // sudo step 3
            Constraint::Length(1),
            Constraint::Length(1), // sudo rule
            Constraint::Min(0),
        ])
        .split(inner_area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ]);

    let col = |row: usize| horiz.split(vert[row])[1];

    frame.render_widget(
        Paragraph::new(format!("openconnect-tui  v{}", env!("CARGO_PKG_VERSION")))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        col(1),
    );
    frame.render_widget(
        Paragraph::new("TUI for GlobalProtect VPN via openconnect-gp")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center),
        col(3),
    );
    frame.render_widget(
        Paragraph::new("https://github.com/EasyCanadianGamer/openconnect-tui")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center),
        col(5),
    );

    frame.render_widget(
        Paragraph::new("── Passwordless sudo (prevents disconnect prompt) ──")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        col(7),
    );

    for (i, line) in [
        "1. sudo visudo -f /etc/sudoers.d/zzz-gpclient",
        "2. Add:  YOUR_USERNAME ALL=(ALL) NOPASSWD: /usr/bin/gpclient",
        "3. Save and exit",
    ]
    .iter()
    .enumerate()
    {
        frame.render_widget(
            Paragraph::new(*line)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center),
            col(9 + i),
        );
    }

    frame.render_widget(
        Paragraph::new("Note: rename to zzz-gpclient so it sorts after other sudoers rules")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        col(13),
    );
}
