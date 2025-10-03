pub mod action;
pub mod buffer;
mod ui;
pub mod window;

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use api::{ApiRequest, ApiResponse, BufferId, Mode, Position, WindowId};
use builtin::Builtin;
use crossterm::event::{self, Event};
use tokio::{sync::Mutex, time::sleep};
use utils::vec2::{IVec2, UVec2};

use crate::{
    action::Action,
    buffer::Buffer,
    ui::{
        commands::CommandMap,
        input::{InputManager, KeyCode},
        render::Renderer,
    },
    window::Window,
};

#[derive(Debug, Default)]
pub struct BufferManager {
    buffers: HashMap<BufferId, Arc<Mutex<Buffer>>>,
    next_index: usize,
}

impl BufferManager {
    pub fn open_buffer(&mut self, buf: Buffer) -> (BufferId, Arc<Mutex<Buffer>>) {
        let id = BufferId(self.next_index);
        let buf = Arc::new(Mutex::new(buf));
        self.buffers.insert(id, buf.clone());
        self.next_index += 1;
        (id, buf)
    }

    pub fn close_buffer(&mut self, id: BufferId) {
        self.buffers.remove(&id);
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<Arc<Mutex<Buffer>>> {
        self.buffers.get(&id).cloned()
    }
}

#[derive(Debug, Default)]
pub struct WindowManager {
    windows: HashMap<WindowId, Arc<Mutex<Window>>>,
    next_index: usize,
}

impl WindowManager {
    pub fn open_window(&mut self, win: Window) -> (WindowId, Arc<Mutex<Window>>) {
        let id = WindowId(self.next_index);
        let win = Arc::new(Mutex::new(win));
        self.windows.insert(id, win.clone());
        self.next_index += 1;
        (id, win)
    }

    pub fn close_buffer(&mut self, id: WindowId) {
        self.windows.remove(&id);
    }

    pub fn get_window(&self, id: WindowId) -> Option<Arc<Mutex<Window>>> {
        self.windows.get(&id).cloned()
    }
}

#[derive(Debug)]
pub struct EditorState {
    #[allow(dead_code)]
    buffer_manager: BufferManager,
    window_manager: WindowManager,
    active_window: WindowId,
    mode: Arc<Mutex<Mode>>,
    command_buf: String,
    command_map: CommandMap,
}

impl EditorState {
    pub fn new(files: Vec<String>) -> anyhow::Result<Self> {
        let mode = Arc::new(Mutex::new(Mode::Normal));
        let mut buffer_manager = BufferManager::default();
        let mut window_manager = WindowManager::default();
        if files.is_empty() {
            let (id, buf) = buffer_manager.open_buffer(Buffer::default());
            window_manager.open_window(Window::new(id, buf, mode.clone()));
        } else {
            for file in files {
                let buf = Buffer::open(PathBuf::from(file))?;
                let (id, buf) = buffer_manager.open_buffer(buf);
                window_manager.open_window(Window::new(id, buf, mode.clone()));
            }
        }

        Ok(Self {
            buffer_manager,
            window_manager,
            active_window: WindowId(0),
            mode,
            command_buf: String::new(),
            command_map: CommandMap::default(),
        })
    }

    pub fn get_active_window(&self) -> Option<Arc<Mutex<Window>>> {
        self.window_manager.get_window(self.active_window)
    }

    async fn set_mode(&mut self, mode: Mode) {
        if let Mode::Insert(true) = mode
            && let Some(win) = self.get_active_window()
        {
            let mut win = win.lock().await;
            win.move_by(IVec2::right()).await;
        }

        let mut mutex_mode = self.mode.lock().await;
        *mutex_mode = mode;
    }

    async fn set_command_to_normal_mode(&mut self) {
        self.command_buf = String::new();
        self.set_mode(Mode::Normal).await;
    }

    pub async fn process_command(&mut self, key_code: KeyCode) -> anyhow::Result<Option<Action>> {
        match key_code {
            KeyCode::Esc => {
                self.command_buf = String::new();
                self.set_command_to_normal_mode().await;
            }
            KeyCode::Backspace => {
                if self.command_buf.is_empty() {
                    self.set_command_to_normal_mode().await;
                } else {
                    self.command_buf.pop();
                }
            }
            KeyCode::Char('\n') => {
                if let Some(action) = self.command_map.get(&self.command_buf) {
                    let action = action.clone();
                    self.set_command_to_normal_mode().await;
                    return Ok(Some(action));
                } else {
                    self.set_command_to_normal_mode().await;
                }
            }
            KeyCode::Char(ch) => self.command_buf.push(ch),
            _ => {}
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub struct EditorApplication {
    state: Arc<Mutex<EditorState>>,
    input_manager: InputManager,
    builtin: Builtin,
    is_quit: bool,
}

impl EditorApplication {
    pub fn new(files: Vec<String>) -> anyhow::Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(EditorState::new(files)?)),
            input_manager: InputManager::default(),
            builtin: Builtin::default(),
            is_quit: false,
        })
    }

    fn quit(&mut self) {
        self.is_quit = true;
    }

    pub fn get_quit(&self) -> bool {
        self.is_quit
    }

    async fn on_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Quit => self.quit(),
            Action::Builtin(action) => self.builtin.on_action(action).await?,
        }
        Ok(())
    }

    async fn process_normal(&mut self, evt: Event) -> anyhow::Result<()> {
        if let Some(action) = self.input_manager.read_event_normal(evt)? {
            self.on_action(action).await?;
        }
        Ok(())
    }

    async fn process_insert(&mut self, evt: Event, is_append: bool) -> anyhow::Result<()> {
        if let Some(key_code) = self.input_manager.event_to_key(evt)? {
            let mut state = self.state.lock().await;
            if let Some(active_window) = state.get_active_window() {
                let mut active_window = active_window.lock().await;
                let cursor = active_window.get_render_cursor().await;
                match key_code {
                    KeyCode::Char(ch) => {
                        {
                            let active_buffer = active_window.get_buffer();
                            let mut active_buffer = active_buffer.lock().await;

                            if ch == '\n' {
                                active_buffer.split_line(cursor);
                            } else {
                                active_buffer.insert_char(cursor, ch);
                            }
                        }

                        if ch == '\n' {
                            active_window.move_by(IVec2::new(0, 1)).await;
                            active_window.move_to_x(0).await;
                        } else {
                            active_window.move_by(IVec2::right()).await;
                        }
                    }
                    KeyCode::Backspace => {
                        if cursor.x == 0 && cursor.y == 0 {}

                        let x = cursor.x;

                        let line_len = {
                            let active_buffer = active_window.get_buffer();
                            let mut active_buffer = active_buffer.lock().await;

                            let line_len = if cursor.x == 0 && cursor.y != 0 {
                                active_buffer.get_line_length(cursor.y - 1)
                            } else {
                                None
                            };

                            if cursor.x == 0 {
                                active_buffer.join_lines(cursor.y - 1);
                            } else {
                                active_buffer.remove_char(UVec2::new(cursor.x - 1, cursor.y));
                            }
                            line_len
                        };

                        if cursor.x != 0 {
                            active_window.move_to_x(x - 1).await;
                        } else if let Some(line_len) = line_len
                            && cursor.y != 0
                        {
                            // 行頭に戻る
                            active_window.move_by(IVec2::new(0, -1)).await;
                            active_window.move_to_x(line_len).await;
                        }
                    }
                    KeyCode::Delete => {
                        let active_buffer = active_window.get_buffer();
                        let mut active_buffer = active_buffer.lock().await;
                        active_buffer.remove_char(cursor);
                    }
                    KeyCode::Esc => {
                        if is_append {
                            active_window.move_by(IVec2::left()).await;
                        }

                        state.set_mode(Mode::Normal).await;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn process_command(&mut self, evt: Event) -> anyhow::Result<()> {
        if let Some(key_code) = self.input_manager.event_to_key(evt)? {
            let action = {
                let mut state = self.state.lock().await;
                state.process_command(key_code).await?
            };

            if let Some(action) = action {
                self.on_action(action).await?;
            }
        }
        Ok(())
    }

    async fn process(&mut self, evt: Event) -> anyhow::Result<()> {
        let mode = {
            let state = self.state.lock().await;
            let mode = state.mode.lock().await;
            mode.clone()
        };

        match mode {
            Mode::Normal => self.process_normal(evt).await,
            Mode::Insert(is_append) => self.process_insert(evt, is_append).await,
            Mode::Command => self.process_command(evt).await,
        }
    }

    async fn process_request(&mut self, request: ApiRequest) -> Option<ApiResponse> {
        let mut state = self.state.lock().await;

        match request {
            ApiRequest::ChangeMode(mode) => {
                state.set_mode(mode).await;
                Some(ApiResponse::None)
            }
            // ApiRequest::OpenFile(path) => {
            //     todo!()
            // }
            // TODO: Pathを使った処理の実装
            ApiRequest::SaveBuffer(buf, _path) => {
                let id = if let Some(buf) = buf {
                    Some(buf)
                } else {
                    if let Some(active) = state.get_active_window() {
                        let win = active.lock().await;
                        Some(win.get_buf())
                    } else {
                        None
                    }
                };

                if let Some(id) = id
                    && let Some(buf) = state.buffer_manager.get_buffer(id)
                {
                    let mut buf = buf.lock().await;
                    buf.save().ok()?;
                }

                Some(ApiResponse::None)
            }
            // ApiRequest::GetLineCount(buf) => {
            //     todo!()
            // }
            // ApiRequest::GetLineLength(buf, y) => {
            //     todo!()
            // }
            // ApiRequest::GetChar(buf, pos) => {
            //     todo!()
            // }
            // ApiRequest::GetLine(buf, y) => {
            //     todo!()
            // }
            // ApiRequest::GetAllLines(buf) => {
            //     todo!()
            // }
            // ApiRequest::GetContent(buf) => {
            //     todo!()
            // }
            // ApiRequest::InsertChar(buf, pos, ch) => {
            //     todo!()
            // }
            // ApiRequest::InsertLine(buf, y, line) => {
            //     todo!()
            // }
            // ApiRequest::ReplaceChar(buf, pos, ch) => {
            //     todo!()
            // }
            // ApiRequest::ReplaceLine(buf, y, line) => {
            //     todo!()
            // }
            // ApiRequest::ReplaceAllLines(buf, lines) => {
            //     todo!()
            // }
            // ApiRequest::ReplaceContent(buf, content) => {
            //     todo!()
            // }
            // ApiRequest::RemoveChar(buf, pos) => {
            //     todo!()
            // }
            // ApiRequest::RemoveLine(buf, y) => {
            //     todo!()
            // }
            // ApiRequest::SplitLine(buf, pos) => {
            //     todo!()
            // }
            // ApiRequest::JoinLines(buf, y) => {
            //     todo!()
            // }
            ApiRequest::MoveBy(win, offset) => {
                let win = if let Some(win) = win {
                    state.window_manager.get_window(win)
                } else {
                    state.get_active_window()
                };

                if let Some(win) = win {
                    let mut win = win.lock().await;
                    win.move_by(offset).await;
                }

                Some(ApiResponse::None)
            }
            ApiRequest::MoveToX(win, pos) => {
                let win = if let Some(win) = win {
                    state.window_manager.get_window(win)
                } else {
                    state.get_active_window()
                };

                if let Some(win) = win {
                    let mut win = win.lock().await;

                    match pos {
                        Position::Number(x) => win.move_to_x(x).await,
                        Position::Start => win.move_to_line_start(),
                        Position::End => win.move_to_line_end().await,
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::MoveToY(win, pos) => {
                let win = if let Some(win) = win {
                    state.window_manager.get_window(win)
                } else {
                    state.get_active_window()
                };

                if let Some(win) = win {
                    let mut win = win.lock().await;

                    match pos {
                        Position::Number(y) => win.move_to_y(y).await,
                        Position::Start => win.move_to_buffer_start(),
                        Position::End => win.move_to_buffer_end().await,
                    }
                }

                Some(ApiResponse::None)
            }
            _ => None,
        }
    }

    pub async fn run_request(&mut self) {
        let messages = self.builtin.api.get_messages();
        for (notify, response, request) in messages {
            log::debug!("run_request: {:?}", request);
            let res = self.process_request(request).await;

            let mut responses = response.lock().await;
            *responses = res;

            notify.notify_one();
        }
    }

    pub async fn run(&mut self, event: Event) {
        match self.process(event).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
    }
}

pub async fn main(files: Vec<String>) -> anyhow::Result<()> {
    let editor = Arc::new(Mutex::new(EditorApplication::new(files)?));

    let editor1 = editor.clone();
    let handle = tokio::spawn(async move {
        let renderer = Renderer::default();
        renderer.init_screen().ok();
        loop {
            let mut editor = editor1.lock().await;
            if editor.get_quit() {
                break;
            }

            editor.run_request().await;

            let state = editor.state.lock().await;
            let active_window = state.get_active_window();
            if let Some(win) = active_window {
                let buf = {
                    let win = win.lock().await;
                    win.get_buffer()
                };

                renderer
                    .render(win, buf, state.mode.clone(), state.command_buf.clone())
                    .await
                    .unwrap();
            }
            sleep(Duration::from_millis(32)).await;
        }
        renderer.clean_screen().ok();
    });

    loop {
        let event = event::read()?;
        let mut editor = editor.lock().await;
        editor.run(event).await;

        if editor.is_quit {
            break;
        }
    }

    handle.await?;

    Ok(())
}
