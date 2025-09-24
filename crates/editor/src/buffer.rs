use std::path::PathBuf;

use crate::file::EditorFile;

#[derive(Debug, Default)]
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
        self.content.get(y).map(|line| line.clone())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf() {
        let mut buf = Buffer::default();
        assert_eq!(buf.remove_line(0), None);
        assert_eq!(buf.remove_line(1), None);

        buf.content.insert(0, "test line".to_string());
        assert_eq!(buf.remove_line(0), Some("test line".to_string()));
        assert_eq!(buf.remove_line(0), None);
    }
}
