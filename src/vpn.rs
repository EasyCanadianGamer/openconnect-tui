use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum VpnStatus {
    Connected,
    Disconnected,
    Error(String),
}

pub fn log_path() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openconnect-tui")
        .join("gpclient.log")
}

fn open_log() -> Option<std::fs::File> {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()
}

pub async fn spawn_vpn(
    server: String,
    browser: String,
    csd_wrapper: String,
    status_tx: mpsc::Sender<VpnStatus>,
    mut kill_rx: mpsc::Receiver<()>,
) {
    let mut cmd = Command::new("sudo");
    // Pass only the env vars the browser needs for auth; avoid -E which can
    // interfere with vpnc-script's route/DNS setup.
    for var in &["DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "XAUTHORITY"] {
        if let Ok(val) = std::env::var(var) {
            cmd.arg(format!("{var}={val}"));
        }
    }
    cmd.args(["gpclient", "connect", &server, "--browser", &browser]);
    if !csd_wrapper.is_empty() {
        cmd.args(["--csd-wrapper", &csd_wrapper]);
    }
    let mut child = match cmd
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = status_tx.send(VpnStatus::Error(e.to_string())).await;
            return;
        }
    };

    // Read stderr line-by-line: write to log and detect "Connected to VPN"
    let stderr = child.stderr.take().expect("stderr piped");
    let tx_clone = status_tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Some(mut log) = open_log() {
                let _ = writeln!(log, "{line}");
            }
            if line.contains("Connected to VPN") {
                let _ = tx_clone.send(VpnStatus::Connected).await;
            }
        }
    });

    tokio::select! {
        status = child.wait() => {
            match status {
                Ok(_) => {
                    let _ = status_tx.send(VpnStatus::Disconnected).await;
                }
                Err(e) => {
                    let _ = status_tx.send(VpnStatus::Error(e.to_string())).await;
                }
            }
        }
        _ = kill_rx.recv() => {
            let _ = child.kill().await;
            disconnect_vpn(&status_tx).await;
        }
    }
}

pub async fn disconnect_vpn(status_tx: &mpsc::Sender<VpnStatus>) {
    let log_stdio = || {
        open_log()
            .map(std::process::Stdio::from)
            .unwrap_or_else(|| std::process::Stdio::null())
    };

    let result = Command::new("sudo")
        .args(["gpclient", "disconnect"])
        .stdout(log_stdio())
        .stderr(log_stdio())
        .status()
        .await;

    match result {
        Ok(s) if s.success() => {
            let _ = status_tx.send(VpnStatus::Disconnected).await;
        }
        Ok(s) => {
            let _ = status_tx
                .send(VpnStatus::Error(format!("disconnect failed with status {s}")))
                .await;
        }
        Err(e) => {
            let _ = status_tx.send(VpnStatus::Error(e.to_string())).await;
        }
    }
}
