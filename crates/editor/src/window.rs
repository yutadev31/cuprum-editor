use utils::vec2::{IVec2, UVec2};

use crate::{cursor::CursorPosition, BufferId};

#[derive(Debug)]
pub struct Window {
    buffer_id: BufferId,
    cursor: CursorPosition,
    scroll: UVec2,
}

impl Window {
    pub fn new(buffer_id: BufferId) -> Self {
        Self {
            buffer_id,
            cursor: CursorPosition::default(),
            scroll: UVec2::default(),
        }
    }

    pub fn get_buf(&self) -> BufferId {
        self.buffer_id
    }

    pub fn get_cursor(&self) -> CursorPosition {
        self.cursor
    }

    pub fn get_scroll(&self) -> UVec2 {
        self.scroll
    }

    pub fn move_by(&mut self, offset: IVec2) {
        match self.cursor {
            CursorPosition::Normal(pos) => {
                if let Some(cursor) = pos.checked_add(offset) {
                    self.cursor = CursorPosition::Normal(cursor);
                }
            }
            _ => {}
        }
    }
}
