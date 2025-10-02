use std::sync::Arc;

use tokio::sync::Mutex;
use utils::{
    term::get_terminal_size,
    vec2::{IVec2, UVec2},
};

use crate::{
    BufferId,
    action::{CursorAction, WindowAction},
    buffer::Buffer,
};

#[derive(Debug)]
pub struct Window {
    buffer_id: BufferId,
    buffer: Arc<Mutex<Buffer>>,
    cursor: UVec2,
    scroll: usize,
    size: UVec2,
}

impl Window {
    pub fn new(buffer_id: BufferId, buffer: Arc<Mutex<Buffer>>) -> Self {
        let term_size = get_terminal_size().unwrap();

        Self {
            buffer_id,
            buffer,
            cursor: UVec2::default(),
            scroll: 0,
            size: UVec2::new(term_size.x, term_size.y - 1),
        }
    }

    pub fn get_buffer(&self) -> Arc<Mutex<Buffer>> {
        self.buffer.clone()
    }

    pub fn get_buf(&self) -> BufferId {
        self.buffer_id
    }

    pub fn get_cursor(&self) -> UVec2 {
        self.cursor
    }

    pub(crate) async fn get_render_cursor(&self) -> UVec2 {
        let buffer = self.buffer.lock().await;

        if let Some(line) = buffer.get_line(self.cursor.y) {
            if self.cursor.x > line.len() {
                return UVec2::new(line.len(), self.cursor.y);
            } else {
                return self.cursor;
            }
        }
        self.cursor
    }

    pub fn get_scroll(&self) -> usize {
        self.scroll
    }

    pub async fn move_by(&mut self, offset: IVec2) {
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

            if let Some(line) = buffer.get_line(self.cursor.y) {
                if x > line.len() {
                    self.cursor.x = line.len();
                } else {
                    self.cursor.x = x;
                }
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

            if let Some(line) = buffer.get_line(self.cursor.y) {
                self.cursor.x = line.len();
            }
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

    pub(crate) async fn on_action(&mut self, action: WindowAction) {
        match action {
            WindowAction::Cursor(action) => match action {
                CursorAction::MoveLeft => self.move_by(IVec2::left()).await,
                CursorAction::MoveDown => self.move_by(IVec2::down()).await,
                CursorAction::MoveUp => self.move_by(IVec2::up()).await,
                CursorAction::MoveRight => self.move_by(IVec2::right()).await,
                CursorAction::MoveToStartOfLine => self.move_to_line_start(),
                CursorAction::MoveToEndOfLine => self.move_to_line_end().await,
                CursorAction::MoveToStartOfBuffer => self.move_to_buffer_start(),
                CursorAction::MoveToEndOfBuffer => self.move_to_buffer_end().await,
            },
        }
    }
}
