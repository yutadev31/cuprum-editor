use std::{
    io::{Write, stdout},
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor::{self, MoveTo},
    execute, queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use utils::vec2::UVec2;

use crate::{buffer::Buffer, window::Window};

#[derive(Debug, Default)]
pub struct Renderer {}

impl Renderer {
    pub fn init_screen(&self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }

    pub fn clean_screen(&self) -> anyhow::Result<()> {
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn render(
        &self,
        active_window: Arc<Mutex<Window>>,
        active_buffer: Arc<Mutex<Buffer>>,
    ) -> anyhow::Result<()> {
        let win = active_window.lock().unwrap();

        let (_w, h) = terminal::size()?;

        queue!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let pos = win.get_render_cursor();
        let scroll = win.get_scroll();

        let buf = active_buffer.lock().unwrap();
        for (y, line) in buf
            .get_lines()
            .iter()
            .skip(scroll)
            .take(h as usize)
            .enumerate()
        {
            queue!(stdout(), MoveTo(0, y as u16), Print(line))?;
        }

        let pos = UVec2::new(pos.x, pos.y.saturating_sub(scroll));
        queue!(stdout(), cursor::MoveTo(pos.x as u16, pos.y as u16))?;

        stdout().flush()?;

        Ok(())
    }
}
