use std::{collections::HashMap, sync::Arc};

use api::{BufferId, WindowId};
use tokio::sync::Mutex;

use crate::{buffer::Buffer, window::Window};

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
