use crate::config::Config;
use tokio::sync::mpsc;

#[derive(Clone, PartialEq)]
pub enum Tab {
    Connect,
    Settings,
}

#[derive(Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

pub struct App {
    pub tab: Tab,
    pub connection: ConnectionState,
    pub config: Config,
    pub spinner_frame: usize,
    pub settings_field: usize,
    pub settings_server: String,
    pub settings_browser: String,
    pub kill_tx: Option<mpsc::Sender<()>>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let settings_server = config.vpn_server.clone();
        let settings_browser = config.browser.clone();
        Self {
            tab: Tab::Connect,
            connection: ConnectionState::Disconnected,
            config,
            spinner_frame: 0,
            settings_field: 0,
            settings_server,
            settings_browser,
            kill_tx: None,
        }
    }

    pub fn tick(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 10;
    }
}
