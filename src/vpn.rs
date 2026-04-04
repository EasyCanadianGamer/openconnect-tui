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
            let _ = status_tx.send(VpnStatus::Disconnected).await;
        }
    }
}
