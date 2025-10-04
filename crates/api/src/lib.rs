use std::fmt::Debug;

use anyhow::anyhow;
use utils::vec2::{IVec2, UVec2};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferId(pub usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WindowId(pub usize);

#[derive(Debug, Clone)]
pub enum Position {
    Number(usize),
    Start,
    End,
}

#[derive(Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    Insert(bool),
    Command,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::Insert(false) => "INSERT",
            Mode::Insert(true) => "INSERT (APPEND)",
            Mode::Command => "COMMAND",
        }
        .to_string()
    }
}

pub trait Api {}

#[derive(Debug, Clone)]
pub enum ApiRequest {
    ChangeMode(Mode),
    OpenFile(Option<String>),
    SaveBuffer(Option<BufferId>, Option<String>),
    GetLineCount(Option<BufferId>),
    GetLineLength(Option<BufferId>, usize),
    GetChar(Option<BufferId>, UVec2),
    GetLine(Option<BufferId>, usize),
    GetAllLines(Option<BufferId>),
    GetContent(Option<BufferId>),
    InsertChar(Option<BufferId>, UVec2, char),
    InsertLine(Option<BufferId>, usize, String),
    ReplaceChar(Option<BufferId>, UVec2, char),
    ReplaceLine(Option<BufferId>, usize, String),
    ReplaceAllLines(Option<BufferId>, Vec<String>),
    ReplaceContent(Option<BufferId>, String),
    RemoveChar(Option<BufferId>, UVec2),
    RemoveLine(Option<BufferId>, usize),
    SplitLine(Option<BufferId>, UVec2),
    JoinLines(Option<BufferId>, usize),
    GetPosition(Option<WindowId>),
    GetVisualStart(Option<WindowId>),
    MoveBy(Option<WindowId>, IVec2),
    MoveToX(Option<WindowId>, Position),
    MoveToY(Option<WindowId>, Position),
}

#[derive(Debug, Clone)]
pub enum ApiResponse {
    None,
    Number(usize),
    Vec2(UVec2),
    Char(char),
    String(String),
    VecString(Vec<String>),
    BufferId(BufferId),
    WindowId(BufferId),
}

#[derive(Debug, Default)]
pub struct CuprumApi<T: CuprumApiProvider> {
    provider: T,
}

impl<T: CuprumApiProvider> CuprumApi<T> {
    pub fn new(provider: T) -> Self {
        Self { provider }
    }

    pub async fn change_mode(&mut self, mode: Mode) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::ChangeMode(mode))
            .await?;
        Ok(())
    }

    pub async fn open_file(&mut self, path: Option<String>) -> anyhow::Result<BufferId> {
        if let ApiResponse::BufferId(buf) = self
            .provider
            .send_message(ApiRequest::OpenFile(path))
            .await?
        {
            Ok(buf)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn save_buffer(
        &mut self,
        buf: Option<BufferId>,
        path: Option<String>,
    ) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::SaveBuffer(buf, path))
            .await?;
        Ok(())
    }

    pub async fn get_line_count(&mut self, buf: Option<BufferId>) -> anyhow::Result<usize> {
        if let ApiResponse::Number(count) = self
            .provider
            .send_message(ApiRequest::GetLineCount(buf))
            .await?
        {
            Ok(count)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn get_line_length(
        &mut self,
        buf: Option<BufferId>,
        y: usize,
    ) -> anyhow::Result<usize> {
        if let ApiResponse::Number(length) = self
            .provider
            .send_message(ApiRequest::GetLineLength(buf, y))
            .await?
        {
            Ok(length)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn get_char(&mut self, buf: Option<BufferId>, pos: UVec2) -> anyhow::Result<char> {
        if let ApiResponse::Char(ch) = self
            .provider
            .send_message(ApiRequest::GetChar(buf, pos))
            .await?
        {
            Ok(ch)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn get_line(&mut self, buf: Option<BufferId>, y: usize) -> anyhow::Result<String> {
        if let ApiResponse::String(line) = self
            .provider
            .send_message(ApiRequest::GetLine(buf, y))
            .await?
        {
            Ok(line)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn get_all_lines(&mut self, buf: Option<BufferId>) -> anyhow::Result<Vec<String>> {
        if let ApiResponse::VecString(lines) = self
            .provider
            .send_message(ApiRequest::GetAllLines(buf))
            .await?
        {
            Ok(lines)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn get_content(&mut self, buf: Option<BufferId>) -> anyhow::Result<String> {
        if let ApiResponse::String(content) = self
            .provider
            .send_message(ApiRequest::GetContent(buf))
            .await?
        {
            Ok(content)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn insert_char(
        &mut self,
        buf: Option<BufferId>,
        pos: UVec2,
        ch: char,
    ) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::InsertChar(buf, pos, ch))
            .await?;
        Ok(())
    }

    pub async fn insert_line(
        &mut self,
        buf: Option<BufferId>,
        y: usize,
        line: String,
    ) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::InsertLine(buf, y, line))
            .await?;
        Ok(())
    }

    pub async fn replace_char(
        &mut self,
        buf: Option<BufferId>,
        pos: UVec2,
        ch: char,
    ) -> anyhow::Result<char> {
        if let ApiResponse::Char(ch) = self
            .provider
            .send_message(ApiRequest::ReplaceChar(buf, pos, ch))
            .await?
        {
            Ok(ch)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn replace_line(
        &mut self,
        buf: Option<BufferId>,
        y: usize,
        line: String,
    ) -> anyhow::Result<String> {
        if let ApiResponse::String(line) = self
            .provider
            .send_message(ApiRequest::ReplaceLine(buf, y, line))
            .await?
        {
            Ok(line)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn replace_all_lines(
        &mut self,
        buf: Option<BufferId>,
        lines: Vec<String>,
    ) -> anyhow::Result<Vec<String>> {
        if let ApiResponse::VecString(lines) = self
            .provider
            .send_message(ApiRequest::ReplaceAllLines(buf, lines))
            .await?
        {
            Ok(lines)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn replace_content(
        &mut self,
        buf: Option<BufferId>,
        content: String,
    ) -> anyhow::Result<String> {
        if let ApiResponse::String(content) = self
            .provider
            .send_message(ApiRequest::ReplaceContent(buf, content))
            .await?
        {
            Ok(content)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn remove_char(&mut self, buf: Option<BufferId>, pos: UVec2) -> anyhow::Result<char> {
        if let ApiResponse::Char(ch) = self
            .provider
            .send_message(ApiRequest::RemoveChar(buf, pos))
            .await?
        {
            Ok(ch)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn remove_line(&mut self, buf: Option<BufferId>, y: usize) -> anyhow::Result<String> {
        if let ApiResponse::String(line) = self
            .provider
            .send_message(ApiRequest::RemoveLine(buf, y))
            .await?
        {
            Ok(line)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn split_line(&mut self, buf: Option<BufferId>, pos: UVec2) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::SplitLine(buf, pos))
            .await?;
        Ok(())
    }

    pub async fn join_lines(&mut self, buf: Option<BufferId>, y: usize) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::JoinLines(buf, y))
            .await?;
        Ok(())
    }

    pub async fn get_position(&mut self, win: Option<WindowId>) -> anyhow::Result<UVec2> {
        if let ApiResponse::Vec2(pos) = self
            .provider
            .send_message(ApiRequest::GetPosition(win))
            .await?
        {
            Ok(pos)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn get_visual_start(&mut self, win: Option<WindowId>) -> anyhow::Result<UVec2> {
        if let ApiResponse::Vec2(pos) = self
            .provider
            .send_message(ApiRequest::GetVisualStart(win))
            .await?
        {
            Ok(pos)
        } else {
            Err(anyhow!("mismatched types"))
        }
    }

    pub async fn move_by(&mut self, win: Option<WindowId>, offset: IVec2) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::MoveBy(win, offset))
            .await?;
        Ok(())
    }

    pub async fn move_to_x(&mut self, win: Option<WindowId>, pos: Position) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::MoveToX(win, pos))
            .await?;
        Ok(())
    }

    pub async fn move_to_y(&mut self, win: Option<WindowId>, pos: Position) -> anyhow::Result<()> {
        self.provider
            .send_message(ApiRequest::MoveToY(win, pos))
            .await?;
        Ok(())
    }
}

pub trait CuprumApiProvider: Default {
    #[allow(async_fn_in_trait)]
    async fn send_message(&mut self, msg: ApiRequest) -> anyhow::Result<ApiResponse>;
}
