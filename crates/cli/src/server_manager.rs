use anyhow::{Context, Result};
use client::Client;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Duration};
use url::Url;

pub struct ServerManager {
    process: Option<Child>,
    port: u16,
    base_url: Url,
}

impl ServerManager {
    pub async fn start(port: u16, config_path: Option<String>) -> Result<Self> {
        let exe = std::env::current_exe()?;

        let mut cmd = Command::new(exe);
        cmd.arg("serve").arg("--port").arg(port.to_string());

        if let Some(path) = config_path {
            cmd.arg("--config").arg(path);
        }

        let child = cmd
            .stdout(Stdio::null()) // Or pipe if we want to log
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn server process")?;

        let base_url = Url::parse(&format!("http://localhost:{}", port))?;
        let manager = Self {
            process: Some(child),
            port,
            base_url,
        };

        // Wait for health check
        manager.wait_for_ready().await?;

        Ok(manager)
    }

    // Connect to existing server
    pub async fn connect(url: Url) -> Result<Self> {
        let manager = Self {
            process: None,
            port: url.port().unwrap_or(80),
            base_url: url,
        };
        manager.wait_for_ready().await?;
        Ok(manager)
    }

    async fn wait_for_ready(&self) -> Result<()> {
        let client = Client::new(self.base_url.clone());
        let mut attempts = 0;
        loop {
            if client.health_check().await.is_ok() {
                return Ok(());
            }

            attempts += 1;
            if attempts > 20 {
                // 10 seconds
                anyhow::bail!("Server failed to start");
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    pub fn client(&self) -> Client {
        Client::new(self.base_url.clone())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(child) = &mut self.process {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        if let Some(child) = &mut self.process {
            // We can't await in drop, so we try to kill it.
            // If it's already awaited, this might fail, but that's fine.
            let _ = child.start_kill();
        }
    }
}
