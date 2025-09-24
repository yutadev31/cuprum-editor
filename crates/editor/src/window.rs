use utils::UVec2;

use crate::{BufferId, cursor::CursorPosition};

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
}
