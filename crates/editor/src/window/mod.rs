use std::sync::Arc;

use api::Mode;
use tokio::sync::Mutex;
use utils::{
    term::get_terminal_size,
    vec2::{IVec2, UVec2},
};

use crate::{BufferId, buffer::Buffer};

#[derive(Debug)]
pub struct Window {
    buffer_id: BufferId,
    buffer: Arc<Mutex<Buffer>>,
    mode: Arc<Mutex<Mode>>,
    cursor: UVec2,
    scroll: usize,
    position: UVec2,
    size: UVec2,
}

impl Window {
    pub fn new(buffer_id: BufferId, buffer: Arc<Mutex<Buffer>>, mode: Arc<Mutex<Mode>>) -> Self {
        let term_size = get_terminal_size().unwrap();

        Self {
            buffer_id,
            buffer,
            mode,
            cursor: UVec2::default(),
            scroll: 0,
            position: UVec2::default(),
            size: UVec2::new(term_size.x, term_size.y - 1),
        }
    }

    pub fn get_position(&self) -> UVec2 {
        self.position
    }

    #[allow(dead_code)] // TODO
    pub fn set_position(&mut self, position: UVec2) {
        self.position = position;
    }

    pub fn get_size(&self) -> UVec2 {
        self.size
    }

    pub fn set_size(&mut self, size: UVec2) {
        self.size = size;
    }

    pub fn get_buffer(&self) -> Arc<Mutex<Buffer>> {
        self.buffer.clone()
    }

    #[allow(dead_code)] // TODO
    pub fn get_buffer_id(&self) -> BufferId {
        self.buffer_id
    }

    #[allow(dead_code)] // TODO
    pub fn get_cursor(&self) -> UVec2 {
        self.cursor
    }

    pub(crate) async fn get_render_cursor(&self) -> UVec2 {
        if let Some(max_x) = self.get_cursor_max_x().await {
            if self.cursor.x > max_x {
                return UVec2::new(max_x, self.cursor.y);
            } else {
                return self.cursor;
            }
        }
        self.cursor
    }

    pub async fn get_cursor_max_x(&self) -> Option<usize> {
        let buffer = self.buffer.lock().await;
        if let Some(line_len) = buffer.get_line_length(self.cursor.y) {
            Some(if let Mode::Insert(_) = self.mode.lock().await.clone() {
                line_len
            } else {
                line_len.checked_sub(1).unwrap_or(line_len)
            })
        } else {
            None
        }
    }

    pub fn get_scroll(&self) -> usize {
        self.scroll
    }

    pub async fn move_by(&mut self, offset: IVec2) {
        {
            if let Some(max_x) = self.get_cursor_max_x().await
                && offset.x != 0
            {
                if self.cursor.x > max_x {
                    self.cursor.x = max_x;
                }
            }
        }

        if let Some(pos) = self.cursor.checked_add(offset) {
            {
                let buffer = self.buffer.lock().await;
                let line_count = buffer.get_line_count();
                if pos.y >= line_count {
                    return;
                }

                self.cursor = pos;
            }
            self.sync_scroll();
        }
    }

    pub async fn move_to_x(&mut self, x: usize) {
        {
            let buffer = self.buffer.lock().await;
            let line_count = buffer.get_line_count();
            if self.cursor.y >= line_count {
                return;
            }
        }

        if let Some(max_x) = self.get_cursor_max_x().await {
            if x > max_x {
                self.cursor.x = max_x;
            } else {
                self.cursor.x = x;
            }
        }
        self.sync_scroll();
    }

    pub async fn move_to_y(&mut self, y: usize) {
        {
            let buffer = self.buffer.lock().await;
            let line_count = buffer.get_line_count();
            if y >= line_count {
                return;
            }

            self.cursor.y = y;

            if let Some(line) = buffer.get_line(self.cursor.y)
                && self.cursor.x > line.len()
            {
                self.cursor.x = line.len();
            }
        }
        self.sync_scroll();
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor.x = 0;
        self.sync_scroll();
    }

    pub async fn move_to_line_end(&mut self) {
        {
            let buffer = self.buffer.lock().await;
            let line_count = buffer.get_line_count();
            if self.cursor.y >= line_count {
                return;
            }

            self.cursor.x = usize::MAX;
        }
        self.sync_scroll();
    }

    pub fn move_to_buffer_start(&mut self) {
        self.cursor = UVec2::new(0, 0);
        self.sync_scroll();
    }

    pub async fn move_to_buffer_end(&mut self) {
        {
            let buffer = self.buffer.lock().await;
            let line_count = buffer.get_line_count();
            if line_count > 0
                && let Some(line) = buffer.get_line(line_count - 1)
            {
                self.cursor = UVec2::new(line.len(), line_count - 1);
            }
        }
        self.sync_scroll();
    }

    pub fn sync_scroll(&mut self) {
        if self.cursor.y < self.scroll {
            self.scroll = self.cursor.y;
        } else if self.cursor.y >= self.scroll + self.size.y {
            if self.size.y > 0 {
                self.scroll = self.cursor.y - self.size.y + 1;
            } else {
                self.scroll = self.cursor.y;
            }
        }
    }
}
