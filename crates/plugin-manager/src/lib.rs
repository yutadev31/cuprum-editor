use std::{path::PathBuf, process::Stdio, sync::Arc};

use api::{CuprumApiRequest, CuprumApiResponse};
use tokio::{
    fs::read_dir,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{Mutex, Notify},
};

#[derive(Debug)]
pub struct Plugin {
    command: PathBuf,
    request_queue: Arc<Mutex<Vec<CuprumApiRequest>>>,
    request_notify: Arc<Notify>,
    response_queue: Arc<Mutex<Vec<Option<CuprumApiResponse>>>>,
    response_notify: Arc<Notify>,
}

type Arcs = (
    Arc<Mutex<Vec<CuprumApiRequest>>>,
    Arc<Notify>,
    Arc<Mutex<Vec<Option<CuprumApiResponse>>>>,
    Arc<Notify>,
);

impl Plugin {
    pub fn new(command: PathBuf) -> Self {
        Self {
            command,
            request_queue: Default::default(),
            request_notify: Default::default(),
            response_queue: Default::default(),
            response_notify: Default::default(),
        }
    }

    pub fn get(&self) -> Arcs {
        (
            self.request_queue.clone(),
            self.request_notify.clone(),
            self.response_queue.clone(),
            self.response_notify.clone(),
        )
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut child = Command::new(&self.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let response_queue = self.response_queue.clone();
        let response_notify = self.response_notify.clone();
        tokio::spawn(async move {
            response_notify.notified().await;
            let queue = response_queue.lock().await;
            for response in queue.clone() {
                let response = serde_json::to_string(&response).unwrap();
                stdin.write_all(response.as_bytes()).await.unwrap();
                stdin.write_all(b"\n").await.unwrap();
                stdin.flush().await.unwrap();
            }
        });

        let mut reader = BufReader::new(stdout);
        loop {
            let mut request = String::new();
            reader.read_line(&mut request).await?;
            let request = serde_json::from_str(&request)?;
            let mut queue = self.request_queue.lock().await;
            queue.push(request);
            self.request_notify.notify_one();
        }
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
                plugin.run().await.unwrap();
            });
        }

        Ok(())
    }
}
