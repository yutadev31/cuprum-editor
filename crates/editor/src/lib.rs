pub mod buffer;
pub mod cursor;
pub mod file;
pub mod input;
pub mod render;
pub mod window;

use std::{collections::HashMap, path::PathBuf};

use crate::{
    buffer::Buffer,
    input::{Action, InputManager},
    render::Renderer,
    window::Window,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferId(pub usize);

#[derive(Debug, Default)]
pub struct BufferManager {
    buffers: HashMap<BufferId, Buffer>,
    next_index: usize,
}

impl BufferManager {
    pub fn open_buffer(&mut self, buf: Buffer) -> BufferId {
        let id = BufferId(self.next_index);
        self.buffers.insert(id, buf);
        self.next_index += 1;
        id
    }

    pub fn close_buffer(&mut self, id: BufferId) {
        self.buffers.remove(&id);
    }

    pub fn get_buffer(&self, id: BufferId) -> Option<&Buffer> {
        self.buffers.get(&id)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WindowId(pub usize);

#[derive(Debug, Default)]
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    next_index: usize,
}

impl WindowManager {
    pub fn open_window(&mut self, win: Window) -> WindowId {
        let id = WindowId(self.next_index);
        self.windows.insert(id, win);
        self.next_index += 1;
        id
    }

    pub fn close_buffer(&mut self, id: WindowId) {
        self.windows.remove(&id);
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }
}

#[derive(Debug)]
pub struct Editor {
    buffer_manager: BufferManager,
    window_manager: WindowManager,
    active_window: WindowId,
    renderer: Renderer,
    input_manager: InputManager,
}

impl Editor {
    pub fn new(files: Vec<String>) -> anyhow::Result<Self> {
        let mut buffer_manager = BufferManager::default();
        let mut window_manager = WindowManager::default();
        if files.is_empty() {
            let id = buffer_manager.open_buffer(Buffer::default());
            window_manager.open_window(Window::new(id));
        } else {
            for file in files {
                let buf = Buffer::open(PathBuf::from(file))?;
                let id = buffer_manager.open_buffer(buf);
                window_manager.open_window(Window::new(id));
            }
        }

        Ok(Self {
            buffer_manager,
            window_manager,
            active_window: WindowId(0),
            renderer: Renderer::default(),
            input_manager: InputManager::default(),
        })
    }

    pub fn run(&self) -> anyhow::Result<()> {
        self.renderer.init_screen()?;
        loop {
            self.renderer.render(
                &self.buffer_manager,
                &self.window_manager,
                self.active_window,
            )?;

            if let Some(Action::Quit) = self.input_manager.read_event()? {
                break;
            }
        }
        self.renderer.clean_screen()?;
        Ok(())
    }
}
