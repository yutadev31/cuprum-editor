use std::{path::PathBuf, process::Stdio, sync::Arc};

use anyhow::{anyhow, bail};
use api::{CuprumApiRequest, CuprumApiResponse};
use tokio::{
    fs::read_dir,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::{Mutex, Notify},
};

#[derive(Debug)]
pub struct Plugin {
    command: PathBuf,
    requests: Arc<Mutex<Vec<CuprumApiRequest>>>,
    request_notify: Arc<Notify>,
    responses: Arc<Mutex<Vec<CuprumApiResponse>>>,
    response_notify: Arc<Notify>,
}

type Arcs = (
    Arc<Mutex<Vec<CuprumApiRequest>>>,
    Arc<Notify>,
    Arc<Mutex<Vec<CuprumApiResponse>>>,
    Arc<Notify>,
);

impl Plugin {
    pub fn new(command: PathBuf) -> Self {
        Self {
            command,
            requests: Default::default(),
            request_notify: Default::default(),
            responses: Default::default(),
            response_notify: Default::default(),
        }
    }

    pub fn get(&self) -> Arcs {
        (
            self.requests.clone(),
            self.request_notify.clone(),
            self.responses.clone(),
            self.response_notify.clone(),
        )
    }

    async fn process_response(
        stdin: &mut ChildStdin,
        queue: &Arc<Mutex<Vec<CuprumApiResponse>>>,
        notify: &Arc<Notify>,
    ) -> anyhow::Result<()> {
        notify.notified().await;
        let queue = queue.lock().await;
        for response in queue.clone() {
            let response = serde_json::to_string(&response)?;
            stdin.write_all(response.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }
        Ok(())
    }

    async fn process_request(
        stdout: &mut BufReader<ChildStdout>,
        queue: &Arc<Mutex<Vec<CuprumApiRequest>>>,
        notify: &Arc<Notify>,
    ) -> anyhow::Result<()> {
        let mut request = String::new();
        stdout.read_line(&mut request).await?;

        if request.is_empty() {
            bail!("Error: Empty request")
        }

        let request = serde_json::from_str(&request)?;
        let mut queue = queue.lock().await;
        queue.push(request);
        notify.notify_one();

        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut child = Command::new(&self.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or(anyhow!("Failed to get stdin"))?;

        let response_queue = self.responses.clone();
        let response_notify = self.response_notify.clone();
        let response_task = tokio::spawn(async move {
            loop {
                match Self::process_response(&mut stdin, &response_queue, &response_notify).await {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("{}", err);
                        break;
                    }
                };
            }
        });

        let stdout = child.stdout.take().ok_or(anyhow!("Failed to get stdout"))?;
        let mut stdout = BufReader::new(stdout);
        let queue = self.requests.clone();
        let notify = self.request_notify.clone();
        let request_task = tokio::spawn(async move {
            loop {
                match Self::process_request(&mut stdout, &queue, &notify).await {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("{}", err);
                        break;
                    }
                }
            }
        });

        tokio::select! {
            _ = response_task => {
                child.kill().await?
            },
            _ = request_task => {
                child.kill().await?
            },
            _ = child.wait() => {
                log::error!("{} finished", self.command.to_string_lossy())
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<Arc<Mutex<Plugin>>>,
}

impl PluginManager {
    fn get_plugin_dir(&self) -> PathBuf {
        let home_dir = home::home_dir().unwrap();

        #[cfg(debug_assertions)]
        let plugin_dir = home_dir.join(".cuprum/debug/plugins");
        #[cfg(not(debug_assertions))]
        let plugin_dir = home_dir.join(".cuprum/plugins");

        plugin_dir
    }

    async fn get_plugin_paths(&self, plugin_dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
        let mut entries = read_dir(plugin_dir).await?;
        let mut paths = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            paths.push(entry.path());
        }

        Ok(paths)
    }

    async fn get_plugins(&self) -> anyhow::Result<Vec<PathBuf>> {
        let plugin_dir = self.get_plugin_dir();
        let plugin_paths = self.get_plugin_paths(plugin_dir).await?;

        Ok(plugin_paths)
    }

    pub async fn init(&mut self) -> anyhow::Result<Vec<Arcs>> {
        let plugins = self.get_plugins().await?;

        let mut arcs = Vec::new();
        for plugin in plugins {
            let plugin = Plugin::new(plugin);
            arcs.push(plugin.get());
            self.plugins.push(Arc::new(Mutex::new(plugin)));
        }

        log::info!("{} plugins loaded", self.plugins.len());
        Ok(arcs)
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        for plugin in &self.plugins {
            let plugin = plugin.clone();
            tokio::spawn(async move {
                let mut plugin = plugin.lock().await;
                match plugin.run().await {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("{}", err);
                    }
                }
            });
        }

        Ok(())
    }
}
