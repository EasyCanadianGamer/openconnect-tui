mod app;
mod config;
mod ui;
mod vpn;

use app::{App, ConnectionState, Tab};
use config::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use vpn::VpnStatus;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::load();
    let mut app = App::new(config);

    // Channel for VPN status updates coming back from the subprocess
    let (status_tx, mut status_rx) = mpsc::channel::<VpnStatus>(16);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(16); // ~60fps
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.tab {
                    Tab::Connect => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::F(1) => app.tab = Tab::Connect,
                        KeyCode::F(2) => app.tab = Tab::Settings,
                        KeyCode::Enter => {
                            match &app.connection {
                                ConnectionState::Disconnected | ConnectionState::Error(_) => {
                                    let (kill_tx, kill_rx) = mpsc::channel::<()>(1);
                                    app.kill_tx = Some(kill_tx);
                                    app.connection = ConnectionState::Connecting;

                                    let server = app.config.vpn_server.clone();
                                    let browser = app.config.browser.clone();
                                    let csd_wrapper = app.config.csd_wrapper.clone();
                                    let tx = status_tx.clone();
                                    tokio::spawn(async move {
                                        vpn::spawn_vpn(server, browser, csd_wrapper, tx, kill_rx).await;
                                    });
                                }
                                ConnectionState::Connecting => {
                                    // Still connecting — kill the connect process
                                    if let Some(tx) = app.kill_tx.take() {
                                        let _ = tx.send(()).await;
                                    }
                                }
                                ConnectionState::Connected => {
                                    // Tunnel is up — run gpclient disconnect
                                    app.connection = ConnectionState::Connecting;
                                    let tx = status_tx.clone();
                                    tokio::spawn(async move {
                                        vpn::disconnect_vpn(&tx).await;
                                    });
                                }
                            }
                        }
                        _ => {}
                    },
                    Tab::Settings => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::F(1) => app.tab = Tab::Connect,
                        KeyCode::F(2) => app.tab = Tab::Settings,
                        KeyCode::Tab => {
                            app.settings_field = (app.settings_field + 1) % 3;
                        }
                        KeyCode::Enter => {
                            app.config.vpn_server = app.settings_server.clone();
                            app.config.browser = app.settings_browser.clone();
                            app.config.csd_wrapper = app.settings_csd_wrapper.clone();
                            let _ = app.config.save();
                        }
                        KeyCode::Backspace => {
                            match app.settings_field {
                                0 => { app.settings_server.pop(); }
                                1 => { app.settings_browser.pop(); }
                                _ => { app.settings_csd_wrapper.pop(); }
                            }
                        }
                        KeyCode::Char(c) => {
                            match app.settings_field {
                                0 => app.settings_server.push(c),
                                1 => app.settings_browser.push(c),
                                _ => app.settings_csd_wrapper.push(c),
                            }
                        }
                        _ => {}
                    },
                }
            }
        }

        // Drain VPN status updates from the subprocess
        while let Ok(status) = status_rx.try_recv() {
            match status {
                VpnStatus::Connected => app.connection = ConnectionState::Connected,
                VpnStatus::Disconnected => {
                    app.connection = ConnectionState::Disconnected;
                    app.kill_tx = None;
                }
                VpnStatus::Error(e) => {
                    app.connection = ConnectionState::Error(e);
                    app.kill_tx = None;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
