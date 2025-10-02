pub mod file;

use std::path::PathBuf;

use crate::{action::BufferAction, buffer::file::EditorFile};

#[derive(Debug)]
pub struct Buffer {
    file: Option<EditorFile>,
    content: Vec<String>,
    dirty: bool,
}

impl Buffer {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let mut file = EditorFile::open(path)?;

        let content = file.read()?;
        let content = content
            .split("\n")
            .map(|line| line.chars().collect())
            .collect();

        Ok(Self {
            file: Some(file),
            content,
            ..Default::default()
        })
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        let content = self.get_content();
        if let Some(file) = &mut self.file {
            file.write(content)?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn get_line_count(&self) -> usize {
        self.content.len()
    }

    pub fn get_line_length(&self, y: usize) -> Option<usize> {
        self.content.get(y).map(|line| line.chars().count())
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.content.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.join("\n")
    }

    pub fn get_line(&self, y: usize) -> Option<String> {
        self.content.get(y).cloned()
    }

    pub fn get_char(&self, x: usize, y: usize) -> Option<char> {
        self.content.get(y).and_then(|line| line.chars().nth(x))
    }

    pub fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.mark_dirty();
        if let Some(line) = self.content.get_mut(y) {
            line.insert(x, ch);
        }
    }

    pub fn remove_char(&mut self, x: usize, y: usize) -> Option<char> {
        self.mark_dirty();
        if let Some(line) = self.content.get_mut(y)
            && x < line.len()
        {
            Some(line.remove(x))
        } else {
            None
        }
    }

    pub fn insert_line(&mut self, y: usize, content: String) {
        self.mark_dirty();
        self.content.insert(y, content);
    }

    pub fn replace_line(&mut self, y: usize, content: String) -> Option<String> {
        if let Some(old_line) = self.get_line(y) {
            self.mark_dirty();
            self.content[y] = content;
            Some(old_line)
        } else {
            None
        }
    }

    pub fn remove_line(&mut self, y: usize) -> Option<String> {
        if y < self.get_line_count() {
            self.mark_dirty();
            Some(self.content.remove(y))
        } else {
            None
        }
    }

    pub fn split_line(&mut self, x: usize, y: usize) {
        self.mark_dirty();

        let original = self.content[y].clone();
        let (p0, p1) = original.split_at(x);
        self.content[y] = p0.to_string();
        self.content.insert(y + 1, p1.to_string());
    }

    pub fn join_lines(&mut self, y: usize) {
        if y + 1 < self.get_line_count() {
            self.mark_dirty();

            let combined = self.content[y].clone() + &self.content[y + 1];
            self.content[y] = combined;
            self.content.remove(y + 1);
        }
    }

    pub(crate) fn on_action(&mut self, action: BufferAction) -> anyhow::Result<()> {
        match action {
            BufferAction::Save => {
                self.save()?;
            }
        }
        Ok(())
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            file: None,
            content: vec![String::new()],
            dirty: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_remove_char() {
        let mut buf = Buffer::default();
        buf.insert_line(0, "Hello".to_string());
        buf.insert_line(1, "World".to_string());

        assert_eq!(buf.get_char(0, 0), Some('H'));
        assert_eq!(buf.get_char(4, 0), Some('o'));
        assert_eq!(buf.get_char(0, 1), Some('W'));
        assert_eq!(buf.get_char(4, 1), Some('d'));
        assert_eq!(buf.get_char(5, 1), None);

        assert_eq!(buf.remove_char(1, 0), Some('e'));
        assert_eq!(buf.get_line(0), Some("Hllo".to_string()));
        assert_eq!(buf.remove_char(10, 0), None);
    }

    #[test]
    fn test_insert_remove_line() {
        let mut buf = Buffer::default();
        assert_eq!(buf.remove_line(0), None);
        assert_eq!(buf.remove_line(1), None);

        buf.insert_line(0, "first line".to_string());
        buf.insert_line(1, "second line".to_string());

        assert_eq!(buf.get_line_count(), 2);
        assert_eq!(buf.get_line(0), Some("first line".to_string()));
        assert_eq!(buf.get_line(1), Some("second line".to_string()));

        assert_eq!(buf.remove_line(0), Some("first line".to_string()));
        assert_eq!(buf.remove_line(0), Some("second line".to_string()));
        assert_eq!(buf.remove_line(0), None);
        assert_eq!(buf.get_line_count(), 0);
    }

    #[test]
    fn test_split_join_line() {
        let mut buf = Buffer::default();
        buf.insert_line(0, "HelloWorld".to_string());

        buf.split_line(5, 0);
        assert_eq!(buf.get_line_count(), 2);
        assert_eq!(buf.get_line(0), Some("Hello".to_string()));
        assert_eq!(buf.get_line(1), Some("World".to_string()));

        buf.join_lines(0);
        assert_eq!(buf.get_line_count(), 1);
        assert_eq!(buf.get_line(0), Some("HelloWorld".to_string()));
    }
}
