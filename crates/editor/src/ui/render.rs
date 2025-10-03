use std::{
    io::{Write, stdout},
    sync::Arc,
};

use api::Mode;
use crossterm::{
    cursor::{self, MoveTo},
    execute, queue,
    style::{self, Print},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use tokio::sync::Mutex;
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

    pub async fn render(
        &self,
        active_window: Arc<Mutex<Window>>,
        active_buffer: Arc<Mutex<Buffer>>,
        mode: Arc<Mutex<Mode>>,
        command_buf: String,
    ) -> anyhow::Result<()> {
        let mut win = active_window.lock().await;

        let (w, h) = terminal::size()?;
        win.set_size(UVec2::new(w.into(), (h - 1).into()));

        let cursor = win.get_render_cursor().await;
        let scroll = win.get_scroll();

        let position = win.get_position();
        let size = win.get_size();

        queue!(stdout(), cursor::MoveTo(0, 0))?;

        let buf = active_buffer.lock().await;
        for (y, line) in buf.get_lines().iter().skip(scroll).take(size.y).enumerate() {
            queue!(
                stdout(),
                MoveTo(position.x as u16, (position.y + y) as u16),
                Print(line),
                Print(" ".repeat(size.x - line.len()))
            )?;
        }

        let mode = mode.lock().await.clone();
        if let Mode::Command = mode {
            queue!(
                stdout(),
                cursor::MoveTo(0, h - 1),
                Print(':'),
                Print(&command_buf),
                Print(" ".repeat(w as usize - command_buf.len() - 1))
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

            let cursor = UVec2::new(cursor.x, cursor.y.saturating_sub(scroll));
            queue!(
                stdout(),
                cursor::MoveTo(
                    (position.x + cursor.x) as u16,
                    (position.y + cursor.y) as u16
                )
            )?;
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
