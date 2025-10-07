use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    sync::Arc,
};

use anyhow::anyhow;
use api_macro::define_api;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout},
    sync::{Mutex, Notify},
};
use utils::vec2::{IVec2, UVec2};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferId(pub usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Position {
    Number(usize),
    Start,
    End,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    Insert(bool),
    Command,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::Normal => "NORMAL",
                Mode::Visual => "VISUAL",
                Mode::Insert(false) => "INSERT",
                Mode::Insert(true) => "INSERT (APPEND)",
                Mode::Command => "COMMAND",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestId(pub usize);

define_api!(
    fn change_mode(mode: Mode)
    fn open_file(path: Option<String>) -> BufferId
    fn save_buffer(buf: Option<BufferId>, path: Option<String>)
    fn get_line_count(buf: Option<BufferId>) -> usize
    fn get_line_length(buf: Option<BufferId>, y: usize) -> usize
    fn get_char(buf: Option<BufferId>, pos: UVec2) -> char
    fn get_line(buf: Option<BufferId>, y: usize) -> String
    fn get_all_lines(buf: Option<BufferId>) -> Vec<String>
    fn get_content(buf: Option<BufferId>) -> String
    fn insert_char(buf: Option<BufferId>, pos: UVec2, ch: char)
    fn insert_line(buf: Option<BufferId>, y: usize, line: String)
    fn replace_char(buf: Option<BufferId>, pos: UVec2, ch: char) -> char
    fn replace_line(buf: Option<BufferId>, y: usize, line: String) -> String
    fn replace_all_lines(buf: Option<BufferId>, lines: Vec<String>) -> Vec<String>
    fn replace_content(buf: Option<BufferId>, content: String) -> String
    fn remove_char(buf: Option<BufferId>, pos: UVec2) -> char
    fn remove_line(buf: Option<BufferId>, y: usize) -> String
    fn split_line(buf: Option<BufferId>, pos: UVec2)
    fn join_lines(buf: Option<BufferId>, y: usize)
    fn get_cursor(win: Option<WindowId>) -> UVec2
    fn get_visual_start(win: Option<WindowId>) -> UVec2
    fn move_by(win: Option<WindowId>, offset: IVec2)
    fn move_to_x(win: Option<WindowId>, pos: Position)
    fn move_to_y(win: Option<WindowId>, pos: Position)
);

pub trait CuprumApiProvider {
    #[allow(async_fn_in_trait)]
    async fn send_message(
        &mut self,
        kind: CuprumApiRequestKind,
    ) -> anyhow::Result<Option<CuprumApiResponseKind>>;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultCuprumApiProvider {
    requests: Arc<Mutex<Vec<CuprumApiRequest>>>,
    request_notify: Arc<Notify>,
    responses: Arc<Mutex<HashMap<RequestId, Option<CuprumApiResponseKind>>>>,
    response_notify: Arc<Notify>,
    next_index: Arc<Mutex<usize>>,
}

impl DefaultCuprumApiProvider {
    async fn process_request(
        requests: &Arc<Mutex<Vec<CuprumApiRequest>>>,
        request_notify: &Arc<Notify>,
    ) -> anyhow::Result<()> {
        request_notify.notified().await;
        let requests = {
            let mut requests = requests.lock().await;
            let cloned_requests = requests.clone();
            requests.clear();
            cloned_requests
        };

        for request in requests {
            let request = serde_json::to_string(&request)?;

            let mut stdout = stdout();
            stdout.write_all(request.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    async fn process_response(
        responses: &Arc<Mutex<HashMap<RequestId, Option<CuprumApiResponseKind>>>>,
        response_notify: &Arc<Notify>,
    ) -> anyhow::Result<()> {
        let mut reader = BufReader::new(stdin());
        let mut response = String::new();
        reader.read_line(&mut response).await?;

        let response: CuprumApiResponse = serde_json::from_str(&response)?;

        let mut responses = responses.lock().await;
        responses.insert(response.id, response.kind);
        response_notify.notify_one();

        Ok(())
    }

    pub fn new() -> Self {
        let provider = Self::default();

        let requests = provider.requests.clone();
        let request_notify = provider.request_notify.clone();
        let responses = provider.responses.clone();
        let response_notify = provider.response_notify.clone();

        tokio::spawn(async move {
            loop {
                match Self::process_request(&requests, &request_notify).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                match Self::process_response(&responses, &response_notify).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        provider
    }
}

impl CuprumApiProvider for DefaultCuprumApiProvider {
    async fn send_message(
        &mut self,
        kind: CuprumApiRequestKind,
    ) -> anyhow::Result<Option<CuprumApiResponseKind>> {
        let id = {
            let mut next_index = self.next_index.lock().await;

            let id = RequestId(next_index.clone());
            let mut requests = self.requests.lock().await;
            requests.push(CuprumApiRequest {
                id: id.clone(),
                kind,
            });

            *next_index += 1;
            id
        };

        self.request_notify.notify_one();
        self.response_notify.notified().await;

        let responses = self.responses.lock().await;
        let response = responses
            .get(&id)
            .ok_or(anyhow!("Failed to get response"))?;

        Ok(response.clone())
    }
}

pub struct CuprumApiManager {}
