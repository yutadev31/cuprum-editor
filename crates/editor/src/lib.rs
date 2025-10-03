mod action;
mod buffer;
mod ui;
mod window;

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use api::{ApiRequest, ApiResponse, BufferId, Mode, Position, WindowId};
use builtin::Builtin;
use crossterm::event::{self, Event};
use tokio::{
    sync::{Mutex, MutexGuard},
    time::sleep,
};
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
pub(crate) struct BufferManager {
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

    #[allow(dead_code)] // TODO
    pub fn close_buffer(&mut self, id: BufferId) {
        self.buffers.remove(&id);
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<Arc<Mutex<Buffer>>> {
        self.buffers.get(&id).cloned()
    }
}

#[derive(Debug, Default)]
pub(crate) struct WindowManager {
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

    #[allow(dead_code)] // TODO
    pub fn close_buffer(&mut self, id: WindowId) {
        self.windows.remove(&id);
    }

    pub fn get_window(&self, id: WindowId) -> Option<Arc<Mutex<Window>>> {
        self.windows.get(&id).cloned()
    }
}

#[derive(Debug)]
pub struct EditorState {
    #[allow(dead_code)] // TODO
    buffer_manager: BufferManager,
    window_manager: WindowManager,
    active_window: WindowId,
    mode: Arc<Mutex<Mode>>,
    command_buf: String,
    command_map: CommandMap,
}

impl EditorState {
    fn new(files: Vec<String>) -> anyhow::Result<Self> {
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

    fn get_active_window(&self) -> Option<Arc<Mutex<Window>>> {
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

    async fn process_command(&mut self, key_code: KeyCode) -> anyhow::Result<Option<Action>> {
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

pub struct EditorApiHandler {
    state: Arc<Mutex<EditorState>>,
}

impl EditorApiHandler {
    pub fn new(state: Arc<Mutex<EditorState>>) -> Self {
        Self { state }
    }

    async fn process(&mut self, request: ApiRequest) -> Option<ApiResponse> {
        let mut state = self.state.lock().await;

        async fn get_window(
            state: MutexGuard<'_, EditorState>,
            win: Option<WindowId>,
        ) -> Option<Arc<Mutex<Window>>> {
            if let Some(win) = win {
                state.window_manager.get_window(win)
            } else {
                state.get_active_window()
            }
        }

        async fn get_buffer(
            state: MutexGuard<'_, EditorState>,
            buf: Option<BufferId>,
        ) -> Option<Arc<Mutex<Buffer>>> {
            if let Some(buf) = buf {
                state.buffer_manager.get_buffer(buf)
            } else {
                if let Some(active) = state.get_active_window() {
                    let win = active.lock().await;
                    Some(win.get_buffer())
                } else {
                    None
                }
            }
        }

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
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.save().ok()?;
                }

                Some(ApiResponse::None)
            }
            ApiRequest::GetLineCount(buf) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    let count = buf.get_line_count();
                    Some(ApiResponse::Number(count))
                } else {
                    Some(ApiResponse::None)
                }
            }
            ApiRequest::GetLineLength(buf, y) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    if let Some(length) = buf.get_line_length(y) {
                        return Some(ApiResponse::Number(length));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::GetChar(buf, pos) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    if let Some(ch) = buf.get_char(pos) {
                        return Some(ApiResponse::Char(ch));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::GetLine(buf, y) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    if let Some(line) = buf.get_line(y) {
                        return Some(ApiResponse::String(line));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::GetAllLines(buf) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    let lines = buf.get_all_lines();
                    Some(ApiResponse::VecString(lines))
                } else {
                    Some(ApiResponse::None)
                }
            }
            ApiRequest::GetContent(buf) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    let lines = buf.get_content();
                    Some(ApiResponse::String(lines))
                } else {
                    Some(ApiResponse::None)
                }
            }
            ApiRequest::InsertChar(buf, pos, ch) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.insert_char(pos, ch);
                }

                Some(ApiResponse::None)
            }
            ApiRequest::InsertLine(buf, y, line) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.insert_line(y, line);
                }

                Some(ApiResponse::None)
            }
            ApiRequest::ReplaceChar(buf, pos, ch) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    if let Some(ch) = buf.replace_char(pos, ch) {
                        return Some(ApiResponse::Char(ch));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::ReplaceLine(buf, y, line) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    if let Some(line) = buf.replace_line(y, line) {
                        return Some(ApiResponse::String(line));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::ReplaceAllLines(buf, lines) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    let lines = buf.replace_all_lines(lines);
                    Some(ApiResponse::VecString(lines))
                } else {
                    Some(ApiResponse::None)
                }
            }
            ApiRequest::ReplaceContent(buf, content) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    let content = buf.replace_content(content);
                    Some(ApiResponse::String(content))
                } else {
                    Some(ApiResponse::None)
                }
            }
            ApiRequest::RemoveChar(buf, pos) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    if let Some(ch) = buf.remove_char(pos) {
                        return Some(ApiResponse::Char(ch));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::RemoveLine(buf, y) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    if let Some(line) = buf.remove_line(y) {
                        return Some(ApiResponse::String(line));
                    }
                }

                Some(ApiResponse::None)
            }
            ApiRequest::SplitLine(buf, pos) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.split_line(pos);
                }

                Some(ApiResponse::None)
            }
            ApiRequest::JoinLines(buf, y) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.join_lines(y);
                }

                Some(ApiResponse::None)
            }
            ApiRequest::GetPosition(win) => {
                if let Some(win) = get_window(state, win).await {
                    let win = win.lock().await;
                    let cursor = win.get_render_cursor().await;
                    Some(ApiResponse::Vec2(cursor))
                } else {
                    Some(ApiResponse::None)
                }
            }
            ApiRequest::MoveBy(win, offset) => {
                if let Some(win) = get_window(state, win).await {
                    let mut win = win.lock().await;
                    win.move_by(offset).await;
                }

                Some(ApiResponse::None)
            }
            ApiRequest::MoveToX(win, pos) => {
                if let Some(win) = get_window(state, win).await {
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
                if let Some(win) = get_window(state, win).await {
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
}

#[derive(Debug)]
pub struct EditorApplication {
    state: Arc<Mutex<EditorState>>,
    input_manager: InputManager,
    builtin: Arc<Mutex<Builtin>>,
    is_quit: bool,
}

impl EditorApplication {
    pub fn new(files: Vec<String>) -> anyhow::Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(EditorState::new(files)?)),
            input_manager: InputManager::default(),
            builtin: Arc::new(Mutex::new(Builtin::default())),
            is_quit: false,
        })
    }

    fn quit(&mut self) {
        self.is_quit = true;
    }

    fn get_quit(&self) -> bool {
        self.is_quit
    }

    async fn on_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Quit => self.quit(),
            Action::Builtin(action) => {
                let mut builtin = self.builtin.lock().await;
                builtin.on_action(action).await?;
            }
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

    async fn run(&mut self, event: Event) {
        match self.process(event).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
    }

    pub async fn main(files: Vec<String>) -> anyhow::Result<()> {
        let editor = Arc::new(Mutex::new(EditorApplication::new(files)?));

        let (messages, notify, state) = {
            let editor = editor.lock().await;
            let builtin = editor.builtin.lock().await;
            (
                builtin.messages.clone(),
                builtin.get_notify(),
                editor.state.clone(),
            )
        };
        tokio::spawn(async move {
            let mut handler = EditorApiHandler::new(state);

            loop {
                notify.notified().await;
                let messages = Builtin::get_messages(&messages).await;
                for (notify, response, request) in messages {
                    let res = handler.process(request).await;

                    let mut responses = response.lock().await;
                    *responses = res;

                    notify.notify_one();
                }
            }
        });

        let editor_render = editor.clone();
        let handle_render = tokio::spawn(async move {
            let renderer = Renderer::default();
            renderer.init_screen().ok();
            loop {
                let editor = editor_render.lock().await;
                if editor.get_quit() {
                    break;
                }

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

        handle_render.await?;

        Ok(())
    }
}
