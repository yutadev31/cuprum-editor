use std::{
    io::{Write, stdout},
    sync::Arc,
};

use crossterm::{
    cursor::{self, MoveTo},
    execute, queue,
    style::{self, Print},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use tokio::sync::Mutex;
use utils::vec2::UVec2;

use crate::{action::Mode, buffer::Buffer, window::Window};

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

    pub async fn render(
        &self,
        active_window: Arc<Mutex<Window>>,
        active_buffer: Arc<Mutex<Buffer>>,
        mode: Mode,
        command_buf: String,
    ) -> anyhow::Result<()> {
        let win = active_window.lock().await;

        let (w, h) = terminal::size()?;

        queue!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let pos = win.get_render_cursor().await;
        let scroll = win.get_scroll();

        let buf = active_buffer.lock().await;
        for (y, line) in buf
            .get_lines()
            .iter()
            .skip(scroll)
            .take((h - 1) as usize)
            .enumerate()
        {
            queue!(stdout(), MoveTo(0, y as u16), Print(line))?;
        }

        if let Mode::Command = mode {
            queue!(
                stdout(),
                cursor::MoveTo(0, h - 1),
                Print(':'),
                Print(command_buf)
            )?;
        } else {
            let status = format!(" {} ", mode.to_string());

            queue!(
                stdout(),
                cursor::MoveTo(0, h - 1),
                style::SetBackgroundColor(style::Color::White),
                style::SetForegroundColor(style::Color::Black),
                Print(status.clone()),
                Print(" ".repeat(w as usize - status.len())),
                style::ResetColor
            )?;

            let pos = UVec2::new(pos.x, pos.y.saturating_sub(scroll));
            queue!(stdout(), cursor::MoveTo(pos.x as u16, pos.y as u16))?;
        }

        if let Mode::Normal = mode {
            queue!(stdout(), cursor::SetCursorStyle::SteadyBlock)?;
        } else {
            queue!(stdout(), cursor::SetCursorStyle::SteadyBar)?;
        }

        stdout().flush()?;

        Ok(())
    }
}
