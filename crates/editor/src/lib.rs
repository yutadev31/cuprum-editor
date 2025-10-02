pub mod action;
pub mod buffer;
mod ui;
pub mod window;

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use crossterm::event::{self, Event};
use tokio::{sync::Mutex, time::sleep};
use utils::vec2::IVec2;

use crate::{
    action::{Action, CursorAction, EditorAction, Mode, WindowAction},
    buffer::Buffer,
    ui::{
        commands::CommandMap,
        input::{InputManager, KeyCode},
        render::Renderer,
    },
    window::Window,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferId(pub usize);

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WindowId(pub usize);

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
pub struct Editor {
    #[allow(dead_code)]
    buffer_manager: BufferManager,
    window_manager: WindowManager,
    active_window: WindowId,
    input_manager: InputManager,
    mode: Arc<Mutex<Mode>>,
    command_buf: String,
    command_map: CommandMap,
    pub is_quit: bool,
}

impl Editor {
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
            input_manager: InputManager::default(),
            mode,
            command_buf: String::new(),
            command_map: CommandMap::default(),
            is_quit: false,
        })
    }

    pub fn get_active_window(&self) -> Option<Arc<Mutex<Window>>> {
        self.window_manager.get_window(self.active_window)
    }

    async fn set_mode(&mut self, mode: Mode) {
        let mut mutex_mode = self.mode.lock().await;
        *mutex_mode = mode;
    }

    async fn on_action(&mut self, action: Action) -> anyhow::Result<bool> {
        if let Some(active_window) = self.get_active_window() {
            let mut active_window = active_window.lock().await;
            match action {
                Action::Editor(action) => match action {
                    EditorAction::Quit => return Ok(true),
                    EditorAction::Mode(mode) => {
                        self.set_mode(mode.clone()).await;

                        if let Mode::Insert(true) = mode {
                            active_window
                                .on_action(WindowAction::Cursor(CursorAction::MoveRight))
                                .await;
                        }
                    }
                    EditorAction::Buffer(action) => {
                        let active_buffer = active_window.get_buffer();
                        let mut active_buffer = active_buffer.lock().await;
                        active_buffer.on_action(action)?;
                    }
                    EditorAction::Window(action) => {
                        active_window.on_action(action).await;
                    }
                },
            }
        }
        Ok(false)
    }

    async fn process_normal(&mut self, evt: Event) -> anyhow::Result<bool> {
        if let Some(action) = self.input_manager.read_event_normal(evt)? {
            self.on_action(action).await
        } else {
            Ok(false)
        }
    }

    async fn process_insert(&mut self, evt: Event, is_append: bool) -> anyhow::Result<bool> {
        if let Some(key_code) = self.input_manager.event_to_key(evt)?
            && let Some(active_window) = self.get_active_window()
        {
            let mut active_window = active_window.lock().await;
            let cursor = active_window.get_render_cursor().await;
            match key_code {
                KeyCode::Char(ch) => {
                    {
                        let active_buffer = active_window.get_buffer();
                        let mut active_buffer = active_buffer.lock().await;

                        if ch == '\n' {
                            active_buffer.split_line(cursor.x, cursor.y);
                        } else {
                            active_buffer.insert_char(cursor.x, cursor.y, ch);
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
                    if cursor.x == 0 && cursor.y == 0 {
                        return Ok(false);
                    }

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
                            active_buffer.remove_char(cursor.x - 1, cursor.y);
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
                    active_buffer.remove_char(cursor.x, cursor.y);
                }
                KeyCode::Esc => {
                    if is_append {
                        active_window
                            .on_action(WindowAction::Cursor(CursorAction::MoveLeft))
                            .await;
                    }

                    self.set_mode(Mode::Normal).await;
                }
                _ => {}
            }
        }

        Ok(false)
    }

    async fn set_command_to_normal_mode(&mut self) {
        self.command_buf = String::new();
        self.set_mode(Mode::Normal).await;
    }

    async fn process_command(&mut self, evt: Event) -> anyhow::Result<bool> {
        if let Some(key_code) = self.input_manager.event_to_key(evt)? {
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
                        let is_quit = self.on_action(action.clone()).await?;
                        self.set_command_to_normal_mode().await;
                        return Ok(is_quit);
                    } else {
                        self.set_command_to_normal_mode().await;
                    }
                }
                KeyCode::Char(ch) => self.command_buf.push(ch),
                _ => {}
            }
        }

        Ok(false)
    }

    async fn process(&mut self, evt: Event) -> anyhow::Result<bool> {
        let mode = self.mode.lock().await.clone();
        match mode {
            Mode::Normal => self.process_normal(evt).await,
            Mode::Insert(is_append) => self.process_insert(evt, is_append).await,
            Mode::Command => self.process_command(evt).await,
        }
    }

    pub async fn run(&mut self, event: Event) {
        match self.process(event).await {
            Ok(is_quit) => {
                if is_quit {
                    self.is_quit = true;
                }
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
    }
}

pub async fn main(files: Vec<String>) -> anyhow::Result<()> {
    let editor = Arc::new(Mutex::new(Editor::new(files)?));

    let editor1 = editor.clone();
    let handle = tokio::spawn(async move {
        let renderer = Renderer::default();
        renderer.init_screen().ok();
        loop {
            let editor = editor1.lock().await;
            if editor.is_quit {
                break;
            }

            let active_window = editor.get_active_window();
            if let Some(win) = active_window {
                let buf = {
                    let win = win.lock().await;
                    win.get_buffer()
                };

                renderer
                    .render(win, buf, editor.mode.clone(), editor.command_buf.clone())
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
