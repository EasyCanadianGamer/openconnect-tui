use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum VpnStatus {
    Connected,
    Disconnected,
    Error(String),
}

pub async fn spawn_vpn(
    server: String,
    browser: String,
    status_tx: mpsc::Sender<VpnStatus>,
    mut kill_rx: mpsc::Receiver<()>,
) {
    let mut child = match Command::new("sudo")
        .args(["-E", "gpclient", "connect", &server, "--browser", &browser])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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
                    // gpclient connect exits after establishing the tunnel — Connected now
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
            // Kill the connect process if still running, then run disconnect
            let _ = child.kill().await;
            disconnect_vpn(&status_tx).await;
        }
    }
}

pub async fn disconnect_vpn(status_tx: &mpsc::Sender<VpnStatus>) {
    let result = Command::new("sudo")
        .args(["-E", "gpclient", "disconnect"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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
