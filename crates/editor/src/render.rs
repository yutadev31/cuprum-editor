use std::io::{stdout, Write};

use crossterm::{
    cursor::{self, MoveTo},
    execute, queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::{cursor::CursorPosition, BufferManager, WindowId, WindowManager};

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
        buffers: &BufferManager,
        windows: &WindowManager,
        active_window: WindowId,
    ) -> anyhow::Result<()> {
        let (_w, h) = terminal::size()?;

        queue!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let active_window = windows.get_window(active_window);
        if let Some(win) = active_window {
            let scroll = win.get_scroll();
            let pos = win.get_cursor();
            let id = win.get_buf();
            let active_buf = buffers.get_buffer(id);
            if let Some(buf) = active_buf {
                for (y, line) in buf
                    .get_lines()
                    .iter()
                    .skip(scroll.y)
                    .take(h as usize)
                    .enumerate()
                {
                    queue!(stdout(), MoveTo(0, y as u16), Print(line))?;
                }
            }
            match pos {
                CursorPosition::Normal(pos) => {
                    queue!(stdout(), cursor::MoveTo(pos.x as u16, pos.y as u16))?;
                }
                _ => {}
            }
        }

        stdout().flush()?;

        Ok(())
    }
}
