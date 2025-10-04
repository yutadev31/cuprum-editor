use std::{
    io::{Write, stdout},
    sync::Arc,
};

use api::Mode;
use crossterm::{
    cursor::{self, MoveTo},
    execute, queue,
    style::{self, Color, Print, ResetColor, SetBackgroundColor},
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
        let visual_start = win.get_visual_start().await;
        let scroll = win.get_scroll();

        let (left, right) = if cursor < visual_start {
            (cursor + UVec2::new(1, 0), visual_start + UVec2::new(1, 0))
        } else {
            (visual_start, cursor)
        };

        let position = win.get_position();
        let size = win.get_size();

        queue!(stdout(), cursor::MoveTo(0, 0))?;

        let mode = mode.lock().await.clone();
        let buf = active_buffer.lock().await;
        for (y, line) in buf
            .get_all_lines()
            .iter()
            .skip(scroll)
            .take(size.y)
            .enumerate()
        {
            let line_y = scroll + y;
            if let Mode::Visual = mode {
                if left.y == line_y && right.y == line_y {
                    let (line_left, line_right) = line.split_at(left.x);
                    let (line_center, line_right) = line_right.split_at(right.x - left.x);
                    queue!(
                        stdout(),
                        MoveTo(position.x as u16, (position.y + y) as u16),
                        Print(line_left),
                        SetBackgroundColor(Color::Blue),
                        Print(line_center),
                        ResetColor,
                        Print(line_right),
                        Print(" ".repeat(size.x - line.len()))
                    )?;
                } else if left.y == line_y && line.len() != 0 {
                    let (line_left, line_right) = line.split_at(left.x);
                    queue!(
                        stdout(),
                        MoveTo(position.x as u16, (position.y + y) as u16),
                        Print(line_left),
                        SetBackgroundColor(Color::Blue),
                        Print(line_right),
                        ResetColor,
                        Print(" ".repeat(size.x - line.len()))
                    )?;
                } else if right.y == line_y {
                    let (line_left, line_right) = line.split_at(right.x);
                    queue!(
                        stdout(),
                        MoveTo(position.x as u16, (position.y + y) as u16),
                        SetBackgroundColor(Color::Blue),
                        Print(line_left),
                        ResetColor,
                        Print(line_right),
                        Print(" ".repeat(size.x - line.len()))
                    )?;
                } else if left.y < line_y && right.y > line_y {
                    queue!(
                        stdout(),
                        MoveTo(position.x as u16, (position.y + y) as u16),
                        SetBackgroundColor(Color::Blue),
                        Print(line),
                        ResetColor,
                        Print(" ".repeat(size.x - line.len()))
                    )?;
                } else {
                    queue!(
                        stdout(),
                        MoveTo(position.x as u16, (position.y + y) as u16),
                        Print(line),
                        Print(" ".repeat(size.x - line.len()))
                    )?;
                }
            } else {
                queue!(
                    stdout(),
                    MoveTo(position.x as u16, (position.y + y) as u16),
                    Print(line),
                    Print(" ".repeat(size.x - line.len()))
                )?;
            }
        }

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

        if let Mode::Normal | Mode::Visual = mode {
            queue!(stdout(), cursor::SetCursorStyle::SteadyBlock)?;
        } else {
            queue!(stdout(), cursor::SetCursorStyle::SteadyBar)?;
        }

        stdout().flush()?;

        Ok(())
    }
}
