use utils::UVec2;

#[derive(Debug, Clone, Copy)]
pub enum CursorPosition {
    Normal(UVec2),
    Selection(UVec2, UVec2),
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self::Normal(UVec2::default())
    }
}
