pub mod action;
pub mod buffer;
mod ui;
pub mod window;

use std::{collections::HashMap, path::PathBuf, sync::Arc, thread, time::Duration};

use crossterm::event::{self, Event};
use tokio::sync::Mutex;
use utils::vec2::IVec2;

use crate::{
    action::{Action, EditorAction, Mode},
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
    buffer_manager: BufferManager,
    window_manager: WindowManager,
    active_window: WindowId,
    input_manager: InputManager,
    mode: Mode,
    command_buf: String,
    command_map: CommandMap,
    pub is_quit: bool,
}

impl Editor {
    pub fn new(files: Vec<String>) -> anyhow::Result<Self> {
        let mut buffer_manager = BufferManager::default();
        let mut window_manager = WindowManager::default();
        if files.is_empty() {
            let (id, buf) = buffer_manager.open_buffer(Buffer::default());
            window_manager.open_window(Window::new(id, buf));
        } else {
            for file in files {
                let buf = Buffer::open(PathBuf::from(file))?;
                let (id, buf) = buffer_manager.open_buffer(buf);
                window_manager.open_window(Window::new(id, buf));
            }
        }

        Ok(Self {
            buffer_manager: buffer_manager,
            window_manager,
            active_window: WindowId(0),
            input_manager: InputManager::default(),
            mode: Mode::Normal,
            command_buf: String::new(),
            command_map: CommandMap::default(),
            is_quit: false,
        })
    }

    pub fn get_active_window(&self) -> Option<Arc<Mutex<Window>>> {
        self.window_manager.get_window(self.active_window)
    }

    async fn on_action(&mut self, action: Action) -> anyhow::Result<bool> {
        if let Some(active_window) = self.get_active_window() {
            let mut active_window = active_window.lock().await;
            match action {
                Action::Editor(action) => match action {
                    EditorAction::Quit => return Ok(true),
                    EditorAction::Mode(mode) => {
                        self.mode = mode;
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

    async fn process_normal(&mut self, event: Event) -> anyhow::Result<bool> {
        let evt = self.input_manager.read_event_normal(event)?;

        if let Some(action) = evt {
            self.on_action(action).await?;
        }

        Ok(false)
    }

    async fn process_insert(&mut self, event: Event) -> anyhow::Result<bool> {
        if let Some(key_code) = self.input_manager.read_event_raw(event)? {
            let active_window = self.get_active_window();

            if let Some(active_window) = active_window {
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

                        let line_len = {
                            let active_buffer = active_window.get_buffer();
                            let mut active_buffer = active_buffer.lock().await;

                            let line_len = active_buffer.get_line_length(cursor.y - 1).unwrap();
                            if cursor.x == 0 {
                                active_buffer.join_lines(cursor.y - 1);
                            } else {
                                active_buffer.remove_char(cursor.x - 1, cursor.y);
                            }
                            line_len
                        };

                        if cursor.x == 0 {
                            // 行頭に戻る
                            active_window.move_by(IVec2::new(0, -1)).await;
                            active_window.move_to_x(line_len).await;
                        } else {
                            active_window.move_by(IVec2::left()).await;
                        }
                    }
                    KeyCode::Delete => {
                        let active_buffer = active_window.get_buffer();
                        let mut active_buffer = active_buffer.lock().await;
                        active_buffer.remove_char(cursor.x, cursor.y);
                    }
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                    }
                    _ => {}
                }
            }
        }

        Ok(false)
    }

    async fn process_command(&mut self, event: Event) -> anyhow::Result<bool> {
        if let Some(key_code) = self.input_manager.read_event_raw(event)? {
            match key_code {
                KeyCode::Esc => {
                    self.command_buf = String::new();
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    if self.command_buf.is_empty() {
                        self.command_buf = String::new();
                        self.mode = Mode::Normal;
                    } else {
                        self.command_buf.pop();
                    }
                }
                KeyCode::Char('\n') => {
                    if let Some(action) = self.command_map.get(&self.command_buf) {
                        let is_quit = self.on_action(action.clone()).await?;
                        self.command_buf = String::new();
                        self.mode = Mode::Normal;
                        return Ok(is_quit);
                    } else {
                        self.command_buf = String::new();
                        self.mode = Mode::Normal;
                    }
                }
                KeyCode::Char(ch) => self.command_buf.push(ch),
                _ => {}
            }
        }

        Ok(false)
    }

    async fn process(&mut self, event: Event) -> anyhow::Result<bool> {
        match self.mode {
            Mode::Normal => self.process_normal(event).await,
            Mode::Insert => self.process_insert(event).await,
            Mode::Command => self.process_command(event).await,
        }
    }

    pub async fn run(&mut self, event: Event) -> anyhow::Result<()> {
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
        Ok(())
    }
}

pub async fn main(files: Vec<String>) -> anyhow::Result<()> {
    let editor = Arc::new(Mutex::new(Editor::new(files)?));

    let editor1 = editor.clone();
    tokio::spawn(async move {
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
            thread::sleep(Duration::from_millis(16));
        }
        renderer.clean_screen().ok();
    });

    loop {
        let event = event::read()?;
        let mut editor = editor.lock().await;
        editor.run(event).await?;

        if editor.is_quit {
            break;
        }
    }

    Ok(())
}
