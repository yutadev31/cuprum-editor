use std::{
    io::{Stdout, Write, stdout},
    sync::Arc,
};

use api::Mode;
use crossterm::{
    cursor::{self, MoveTo},
    execute, queue,
    style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
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

    fn render_move_cursor(&self, stdout: &mut Stdout, cursor: UVec2) -> anyhow::Result<()> {
        queue!(stdout, MoveTo(cursor.x as u16, cursor.y as u16))?;
        Ok(())
    }

    fn render_code_token(
        &self,
        stdout: &mut Stdout,
        token: &str,
        fg: Option<Color>,
        bg: Option<Color>,
    ) -> anyhow::Result<()> {
        queue!(stdout, ResetColor)?;

        if let Some(fg) = fg {
            queue!(stdout, SetForegroundColor(fg))?;
        }

        if let Some(bg) = bg {
            queue!(stdout, SetBackgroundColor(bg))?;
        }

        queue!(stdout, Print(token))?;

        Ok(())
    }

    fn render_code_line(
        &self,
        stdout: &mut Stdout,
        line: &str,
        line_y: usize,
        y: usize,
        mode: &Mode,
        visual_cursor: (UVec2, UVec2),
        position: UVec2,
    ) -> anyhow::Result<()> {
        self.render_move_cursor(stdout, UVec2::new(position.x, position.y + y))?;

        if let Mode::Visual = mode {
            let (left, right) = visual_cursor;

            if left.y == line_y && right.y == line_y {
                let (line_left, line_right) = line.split_at(left.x);
                let (line_center, line_right) = line_right.split_at(right.x - left.x);

                self.render_code_token(stdout, line_left, None, None)?;
                self.render_code_token(stdout, line_center, None, Some(Color::Blue))?;
                self.render_code_token(stdout, line_right, None, None)?;
            } else if left.y == line_y && !line.is_empty() {
                let (line_left, line_right) = line.split_at(left.x);

                self.render_code_token(stdout, line_left, None, None)?;
                self.render_code_token(stdout, line_right, None, Some(Color::Blue))?;
            } else if right.y == line_y {
                let (line_left, line_right) = line.split_at(right.x);

                self.render_code_token(stdout, line_left, None, Some(Color::Blue))?;
                self.render_code_token(stdout, line_right, None, None)?;
            } else if left.y < line_y && right.y > line_y {
                self.render_code_token(stdout, line, None, Some(Color::Blue))?;
            } else {
                self.render_code_token(stdout, line, None, None)?;
            }
        } else {
            self.render_code_token(stdout, line, None, None)?;
        }

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

        let visual_cursor = if cursor < visual_start {
            (cursor + UVec2::new(1, 0), visual_start + UVec2::new(1, 0))
        } else {
            (visual_start, cursor)
        };

        let position = win.get_position();
        let size = win.get_size();

        let mut stdout = stdout();

        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All)
        )?;

        let mode = mode.lock().await.clone();
        let buf = active_buffer.lock().await;
        for (y, line) in buf
            .get_all_lines()
            .iter()
            .skip(scroll)
            .take(size.y)
            .enumerate()
        {
            self.render_code_line(
                &mut stdout,
                line,
                y + scroll,
                y,
                &mode,
                visual_cursor,
                position,
            )?;
        }

        if let Mode::Command = mode {
            queue!(
                stdout,
                cursor::MoveTo(0, h - 1),
                Print(':'),
                Print(&command_buf),
            )?;
        } else {
            let status = format!(" {} ", mode);

            queue!(
                stdout,
                cursor::MoveTo(0, h - 1),
                style::SetBackgroundColor(style::Color::White),
                style::SetForegroundColor(style::Color::Black),
                Print(status.clone()),
                Print(" ".repeat(w as usize - status.len())),
                style::ResetColor
            )?;

            let cursor = UVec2::new(cursor.x, cursor.y.saturating_sub(scroll));
            queue!(
                stdout,
                cursor::MoveTo(
                    (position.x + cursor.x) as u16,
                    (position.y + cursor.y) as u16
                )
            )?;
        }

        if let Mode::Normal | Mode::Visual = mode {
            queue!(stdout, cursor::SetCursorStyle::SteadyBlock)?;
        } else {
            queue!(stdout, cursor::SetCursorStyle::SteadyBar)?;
        }

        stdout.flush()?;

        Ok(())
    }
}
