mod action;
mod buffer;
mod managers;
mod state;
mod ui;
mod window;

use std::{sync::Arc, time::Duration};

use api::{
    BufferId, CuprumApiRequestKind, CuprumApiResponse, CuprumApiResponseKind, Mode, Position,
    WindowId,
};
use builtin::{Builtin, BuiltinApiProvider};
use crossterm::event::{self, Event};
use plugin_manager::PluginManager;
use tokio::{
    sync::{Mutex, MutexGuard},
    time::sleep,
};
use utils::vec2::IVec2;

use crate::{
    action::Action,
    buffer::Buffer,
    state::EditorState,
    ui::{
        input::{InputManager, KeyCode},
        render::Renderer,
    },
    window::Window,
};

pub struct EditorApiHandler {
    state: Arc<Mutex<EditorState>>,
}

impl EditorApiHandler {
    /// Create a new editor API handler
    pub fn new(state: Arc<Mutex<EditorState>>) -> Self {
        Self { state }
    }

    /// Process a Cuprum API request
    async fn process(&mut self, request: CuprumApiRequestKind) -> Option<CuprumApiResponseKind> {
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
            } else if let Some(active) = state.get_active_window() {
                let win = active.lock().await;
                Some(win.get_buffer())
            } else {
                None
            }
        }

        match request {
            CuprumApiRequestKind::ChangeMode(mode) => {
                state.set_mode(mode).await;
                None
            }
            CuprumApiRequestKind::OpenFile(_path) => {
                todo!()
            }
            // TODO: Pathを使った処理の実装
            CuprumApiRequestKind::SaveBuffer(buf, _path) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.save().ok();
                }

                None
            }
            CuprumApiRequestKind::GetLineCount(buf) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    let count = buf.get_line_count();
                    Some(CuprumApiResponseKind::GetLineCount(count))
                } else {
                    None
                }
            }
            CuprumApiRequestKind::GetLineLength(buf, y) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    let length = buf.get_line_length(y);
                    return Some(CuprumApiResponseKind::GetLineLength(length));
                }

                None
            }
            CuprumApiRequestKind::GetContent(buf) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let buf = buf.lock().await;
                    let content = buf.get_content();
                    Some(CuprumApiResponseKind::GetContent(content))
                } else {
                    None
                }
            }
            CuprumApiRequestKind::Insert(buf, position, text) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.insert(position, &text);
                }

                None
            }
            CuprumApiRequestKind::InsertChar(buf, position, ch) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    buf.insert_char(position, ch);
                }

                None
            }
            CuprumApiRequestKind::InsertLine(buf, line, text) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    let start = buf.get_line_start(line);
                    buf.insert_char(start, '\n');
                    buf.insert(start, &text);
                }

                None
            }
            CuprumApiRequestKind::Remove(buf, start, end) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    if let Some(text) = buf.remove(start, end) {
                        return Some(CuprumApiResponseKind::Remove(text));
                    }
                }

                None
            }
            CuprumApiRequestKind::RemoveChar(buf, pos) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    if let Some(text) = buf.remove_char(pos) {
                        return Some(CuprumApiResponseKind::RemoveChar(text));
                    }
                }

                None
            }
            CuprumApiRequestKind::RemoveLine(buf, line) => {
                if let Some(buf) = get_buffer(state, buf).await {
                    let mut buf = buf.lock().await;
                    let start = buf.get_line_start(line);
                    let length = buf.get_line_length(line);

                    if let Some(line) = buf.remove(start, start + length) {
                        return Some(CuprumApiResponseKind::RemoveLine(line));
                    }
                }

                None
            }
            CuprumApiRequestKind::GetCursor(win) => {
                if let Some(win) = get_window(state, win).await {
                    let win = win.lock().await;
                    let cursor = win.get_cursor_usize().await;
                    Some(CuprumApiResponseKind::GetCursor(cursor))
                } else {
                    None
                }
            }
            CuprumApiRequestKind::GetCursorVec2(win) => {
                if let Some(win) = get_window(state, win).await {
                    let win = win.lock().await;
                    let cursor = win.get_cursor().await;
                    Some(CuprumApiResponseKind::GetCursorVec2(cursor))
                } else {
                    None
                }
            }
            CuprumApiRequestKind::GetVisualStart(win) => {
                if let Some(win) = get_window(state, win).await {
                    let win = win.lock().await;
                    let cursor = win.get_visual_start_usize().await;
                    Some(CuprumApiResponseKind::GetVisualStart(cursor))
                } else {
                    None
                }
            }
            CuprumApiRequestKind::GetVisualStartVec2(win) => {
                if let Some(win) = get_window(state, win).await {
                    let win = win.lock().await;
                    let cursor = win.get_visual_start();
                    Some(CuprumApiResponseKind::GetVisualStartVec2(cursor))
                } else {
                    None
                }
            }
            CuprumApiRequestKind::MoveBy(win, offset) => {
                if let Some(win) = get_window(state, win).await {
                    let mut win = win.lock().await;
                    win.move_by(offset).await;
                }

                None
            }
            CuprumApiRequestKind::MoveTo(win, pos) => {
                if let Some(win) = get_window(state, win).await {
                    let mut win = win.lock().await;
                    win.move_to(pos).await;
                }

                None
            }
            CuprumApiRequestKind::MoveToX(win, pos) => {
                if let Some(win) = get_window(state, win).await {
                    let mut win = win.lock().await;

                    match pos {
                        Position::Number(x) => win.move_to_x(x).await,
                        Position::Start => win.move_to_line_start(),
                        Position::End => win.move_to_line_end().await,
                    }
                }

                None
            }
            CuprumApiRequestKind::MoveToY(win, pos) => {
                if let Some(win) = get_window(state, win).await {
                    let mut win = win.lock().await;

                    match pos {
                        Position::Number(y) => win.move_to_y(y).await,
                        Position::Start => win.move_to_buffer_start(),
                        Position::End => win.move_to_buffer_end().await,
                    }
                }

                None
            }
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
    /// Create a new editor application
    pub fn new(files: Vec<String>) -> anyhow::Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(EditorState::new(files)?)),
            input_manager: InputManager::default(),
            builtin: Arc::new(Mutex::new(Builtin::default())),
            is_quit: false,
        })
    }

    /// Quit the application
    fn quit(&mut self) {
        self.is_quit = true;
    }

    /// Get the quit state
    fn get_quit(&self) -> bool {
        self.is_quit
    }

    /// Run an action
    async fn run_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Quit => self.quit(),
            Action::Builtin(action) => {
                let mut builtin = self.builtin.lock().await;
                builtin.on_action(action).await?;
            }
        }
        Ok(())
    }

    /// Process a single terminal when in normal mode
    async fn process_normal(&mut self, evt: Event) -> anyhow::Result<()> {
        if let Some(action) = self.input_manager.read_event_normal(evt)? {
            self.run_action(action).await?;
        }
        Ok(())
    }

    /// Process a single terminal when in visual mode
    async fn process_visual(&mut self, evt: Event) -> anyhow::Result<()> {
        if let Some(action) = self.input_manager.read_event_visual(evt)? {
            self.run_action(action).await?;
        }
        Ok(())
    }

    /// Process a single terminal when in insert mode
    async fn process_insert(&mut self, evt: Event, is_append: bool) -> anyhow::Result<()> {
        if let Some(key_code) = self.input_manager.event_to_key(evt)? {
            let mut state = self.state.lock().await;
            if let Some(active_window) = state.get_active_window() {
                let mut active_window = active_window.lock().await;
                let cursor = active_window.get_cursor().await;
                let cursor_usize = active_window.get_cursor_usize().await;
                match key_code {
                    KeyCode::Char(ch) => {
                        {
                            let active_buffer = active_window.get_buffer();
                            let mut active_buffer = active_buffer.lock().await;

                            active_buffer.insert_char(cursor_usize, ch);
                        }

                        if ch == '\n' {
                            active_window.move_by(IVec2::new(0, 1)).await;
                            active_window.move_to_x(0).await;
                        } else {
                            active_window.move_by(IVec2::right()).await;
                        }
                    }
                    KeyCode::Backspace => {
                        let x = cursor.x;

                        let line_len = {
                            let active_buffer = active_window.get_buffer();
                            let mut active_buffer = active_buffer.lock().await;

                            let line_len = if cursor.x == 0 && cursor.y != 0 {
                                Some(active_buffer.get_line_length(cursor.y - 1))
                            } else {
                                None
                            };

                            active_buffer.remove_char(cursor_usize - 1);
                            line_len
                        };

                        if cursor.x != 0 {
                            active_window.move_to_x(x - 1).await;
                        } else if let Some(line_len) = line_len
                            && cursor.y != 0
                        {
                            // Go to the beginning of the previous line
                            active_window.move_by(IVec2::new(0, -1)).await;
                            active_window.move_to_x(line_len).await;
                        }
                    }
                    KeyCode::Delete => {
                        let active_buffer = active_window.get_buffer();
                        let mut active_buffer = active_buffer.lock().await;
                        active_buffer.remove_char(cursor_usize);
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

    /// Process a single terminal when in command mode
    async fn process_command(&mut self, evt: Event) -> anyhow::Result<()> {
        if let Some(key_code) = self.input_manager.event_to_key(evt)? {
            let action = {
                let mut state = self.state.lock().await;
                state.process_command(key_code).await?
            };

            if let Some(action) = action {
                self.run_action(action).await?;
            }
        }
        Ok(())
    }

    /// Process a single terminal event
    async fn process(&mut self, evt: Event) -> anyhow::Result<()> {
        let mode = {
            let state = self.state.lock().await;
            let mode = state.mode.lock().await;
            mode.clone()
        };

        match mode {
            Mode::Normal => self.process_normal(evt).await,
            Mode::Visual => self.process_visual(evt).await,
            Mode::Insert(is_append) => self.process_insert(evt, is_append).await,
            Mode::Command => self.process_command(evt).await,
        }
    }

    /// Editor Application main entry point
    pub async fn main(files: Vec<String>) -> anyhow::Result<()> {
        let editor = Arc::new(Mutex::new(EditorApplication::new(files)?));

        // Run builtin features
        let (messages, notify, builtin_state, plugin_state) = {
            let editor = editor.lock().await;
            let builtin = editor.builtin.lock().await;
            (
                builtin.get_messages(),
                builtin.get_notify(),
                editor.state.clone(),
                editor.state.clone(),
            )
        };
        tokio::spawn(async move {
            let mut handler = EditorApiHandler::new(builtin_state);

            loop {
                notify.notified().await;
                let messages = BuiltinApiProvider::get_messages(&messages).await;
                for (notify, response, request) in messages {
                    let res = handler.process(request).await;

                    let mut responses = response.lock().await;
                    *responses = res;

                    notify.notify_one();
                }
            }
        });

        // Run plugin manager
        tokio::spawn(async move {
            let mut plugin_manager = PluginManager::default();
            let result = plugin_manager.init().await.unwrap();
            for (requests, request_notify, responses, response_notify) in result {
                let state = plugin_state.clone();
                tokio::spawn(async move {
                    let mut handler = EditorApiHandler::new(state);
                    loop {
                        request_notify.notified().await;
                        let requests = requests.lock().await;
                        let mut responses = responses.lock().await;

                        for request in requests.clone() {
                            let response = handler.process(request.kind).await;
                            responses.push(CuprumApiResponse {
                                id: request.id,
                                kind: response,
                            });
                        }

                        response_notify.notify_one();
                    }
                });
            }

            plugin_manager.run().await.unwrap();
        });

        // Render in terminal
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

        // Handle terminal events
        loop {
            let event = event::read()?;
            let mut editor = editor.lock().await;

            match editor.process(event).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error: {:?}", e);
                }
            }

            if editor.is_quit {
                break;
            }
        }

        handle_render.await?;

        Ok(())
    }
}
