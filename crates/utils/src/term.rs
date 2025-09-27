use crossterm::terminal;

use crate::vec2::UVec2;

pub fn get_terminal_size() -> anyhow::Result<UVec2> {
    let (w, h) = terminal::size()?;
    Ok(UVec2::new(w as usize, h as usize))
}
