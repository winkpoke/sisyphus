use anyhow::{Context, Result};
use client::Client;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Duration};
use url::Url;

pub struct ServerManager {
    process: Option<Child>,
    base_url: Url,
}

impl ServerManager {
    pub async fn start(
        port: u16,
        config_path: Option<String>,
        log_file: Option<String>,
    ) -> Result<Self> {
        let exe = std::env::current_exe()?;

        let mut cmd = Command::new(exe);
        cmd.arg("serve").arg("--port").arg(port.to_string());
        cmd.arg("--print-port");

        if let Some(path) = config_path {
            cmd.arg("--config").arg(path);
        }

        if let Some(log_path) = &log_file {
            cmd.arg("--log-file").arg(log_path);
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn server process")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;

        let mut reader = tokio::io::BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let actual_port: u16 = line
            .trim()
            .parse()
            .with_context(|| format!("Failed to parse port from server output: '{}'", line))?;

        let base_url = Url::parse(&format!("http://localhost:{}", actual_port))?;
        let mut manager = Self {
            process: Some(child),
            base_url,
        };

        manager.wait_for_ready().await?;

        Ok(manager)
    }

    pub async fn connect(url: Url) -> Result<Self> {
        let mut manager = Self {
            process: None,
            base_url: url,
        };
        manager.wait_for_ready().await?;
        Ok(manager)
    }

    async fn wait_for_ready(&mut self) -> Result<()> {
        let client = Client::new(self.base_url.clone());
        let mut attempts = 0;
        loop {
            if let Some(ref mut child) = &mut self.process {
                if let Ok(Some(exit_status)) = child.try_wait() {
                    anyhow::bail!(
                        "Server process exited unexpectedly with status: {:?}",
                        exit_status
                    );
                }
            }

            if client.health_check().await.is_ok() {
                return Ok(());
            }

            attempts += 1;
            if attempts > 20 {
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
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_manager_structure() {
        let url = url::Url::parse("http://localhost:8080").unwrap();
        let manager = ServerManager {
            process: None,
            base_url: url.clone(),
        };

        assert_eq!(manager.base_url, url);
        assert!(manager.process.is_none());
    }

    #[test]
    fn test_client_method() {
        let url = url::Url::parse("http://localhost:8080").unwrap();
        let manager = ServerManager {
            process: None,
            base_url: url.clone(),
        };

        let _client = manager.client();
    }

    #[test]
    fn test_url_parsing() {
        let port = 8080;
        let url_str = format!("http://localhost:{}", port);
        let url = url::Url::parse(&url_str);

        assert!(url.is_ok());

        let parsed = url.unwrap();
        assert_eq!(parsed.host_str(), Some("localhost"));
        assert_eq!(parsed.port().unwrap_or(80), 8080);
    }

    #[test]
    fn test_stop_with_no_process() {
        let mut manager = ServerManager {
            process: None,
            base_url: url::Url::parse("http://localhost:8080").unwrap(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(manager.stop());

        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_trait() {
        let url = url::Url::parse("http://localhost:8080").unwrap();
        let manager = ServerManager {
            process: None,
            base_url: url,
        };

        drop(manager);
    }
}
