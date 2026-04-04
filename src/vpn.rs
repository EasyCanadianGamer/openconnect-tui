use std::fs;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum VpnStatus {
    Connected,
    Disconnected,
    Error(String),
}

fn log_file() -> std::process::Stdio {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map(std::process::Stdio::from)
        .unwrap_or_else(|_| std::process::Stdio::null())
}

pub fn log_path() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openconnect-tui")
        .join("gpclient.log")
}

pub async fn spawn_vpn(
    server: String,
    browser: String,
    status_tx: mpsc::Sender<VpnStatus>,
    mut kill_rx: mpsc::Receiver<()>,
) {
    let mut child = match Command::new("sudo")
        .args(["-E", "gpclient", "connect", &server, "--browser", &browser])
        .stdout(log_file())
        .stderr(log_file())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = status_tx.send(VpnStatus::Error(e.to_string())).await;
            return;
        }
    };

    tokio::select! {
        status = child.wait() => {
            match status {
                Ok(s) if s.success() => {
                    let _ = status_tx.send(VpnStatus::Connected).await;
                }
                Ok(s) => {
                    let _ = status_tx
                        .send(VpnStatus::Error(format!("exited with status {s}")))
                        .await;
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
    let result = Command::new("sudo")
        .args(["-E", "gpclient", "disconnect"])
        .stdout(log_file())
        .stderr(log_file())
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
