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
        mode: Arc<Mutex<Mode>>,
        command_buf: String,
    ) -> anyhow::Result<()> {
        let win = active_window.lock().await;
        let (term_w, term_h) = terminal::size()?;

        queue!(stdout(), cursor::MoveTo(0, 0))?;

        let pos = win.get_render_cursor().await;
        let scroll = win.get_scroll();

        let win_position = win.get_position();
        let win_size = win.get_size();

        let buf = active_buffer.lock().await;
        for (y, line) in buf
            .get_lines()
            .iter()
            .skip(scroll)
            .take(win_size.y)
            .enumerate()
        {
            queue!(
                stdout(),
                MoveTo(win_position.x as u16, (win_position.y + y) as u16),
                Print(line),
                Print(" ".repeat(win_size.x - line.len())),
            )?;
        }

        let mode = mode.lock().await.clone();
        if let Mode::Command = mode {
            queue!(
                stdout(),
                cursor::MoveTo(0, term_h - 1),
                terminal::Clear(terminal::ClearType::CurrentLine),
                Print(':'),
                Print(command_buf),
            )?;
        } else {
            let status = format!(" {} ", mode.to_string());

            queue!(
                stdout(),
                cursor::MoveTo(0, term_h - 1),
                style::SetBackgroundColor(style::Color::White),
                style::SetForegroundColor(style::Color::Black),
                Print(status.clone()),
                Print(" ".repeat(term_w as usize - status.len())),
                style::ResetColor
            )?;

            let pos = UVec2::new(pos.x, pos.y.saturating_sub(scroll));
            queue!(
                stdout(),
                cursor::MoveTo(
                    (win_position.x + pos.x) as u16,
                    (win_position.y + pos.y) as u16
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
