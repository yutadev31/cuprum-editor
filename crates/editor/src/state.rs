use std::{path::PathBuf, sync::Arc};

use api::{Mode, WindowId};
use tokio::sync::Mutex;
use utils::vec2::IVec2;

use crate::{
    action::Action,
    buffer::Buffer,
    managers::{BufferManager, WindowManager},
    ui::{commands::CommandMap, input::KeyCode},
    window::Window,
};

#[derive(Debug)]
pub struct EditorState {
    pub buffer_manager: BufferManager,
    pub window_manager: WindowManager,
    active_window: WindowId,
    pub mode: Arc<Mutex<Mode>>,
    pub command_buf: String,
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

    pub async fn set_mode(&mut self, mode: Mode) {
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
