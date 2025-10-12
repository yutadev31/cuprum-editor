mod content;
mod file;

use std::path::PathBuf;

use utils::vec2::UVec2;

use crate::buffer::{content::BufferContent, file::EditorFile};

#[derive(Debug)]
pub struct Buffer {
    file: Option<EditorFile>,
    content: BufferContent,
    dirty: bool,
}

impl Buffer {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let mut file = EditorFile::open(path)?;

        let content = file.read()?;

        Ok(Self {
            file: Some(file),
            content: BufferContent::from_str(&content),
            ..Default::default()
        })
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        let content = self.content.get();
        if let Some(file) = &mut self.file {
            file.write(content)?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn vec2_to_index(&self, vec2: UVec2) -> usize {
        self.content.vec2_to_index(vec2)
    }

    pub fn index_to_vec2(&self, index: usize) -> UVec2 {
        self.content.index_to_vec2(index)
    }

    pub fn get_line_start(&self, line: usize) -> usize {
        self.content.get_line_start(line)
    }

    pub fn get_line_count(&self) -> usize {
        self.content.get_line_count()
    }

    pub fn get_line_length(&self, line: usize) -> usize {
        self.content.get_line_length(line)
    }

    pub fn get_all_lines(&self) -> Vec<String> {
        self.content
            .get()
            .split('\n')
            .map(|text| text.to_string())
            .collect()
    }

    pub fn get_content(&self) -> String {
        self.content.get()
    }

    pub fn insert(&mut self, position: usize, text: &str) {
        self.content.insert(position, text);
    }

    pub fn insert_char(&mut self, position: usize, ch: char) {
        self.content.insert(position, &ch.to_string());
    }

    pub fn remove(&mut self, start: usize, end: usize) -> Option<String> {
        self.content.remove(start, end)
    }

    pub fn remove_char(&mut self, position: usize) -> Option<char> {
        self.content.remove_char(position)
    }

    // pub fn get_line(&self, y: usize) -> Option<String> {
    //     self.content.get(y).cloned()
    // }

    // pub fn get_char(&self, pos: UVec2) -> Option<char> {
    //     self.content
    //         .get(pos.y)
    //         .and_then(|line| line.chars().nth(pos.x))
    // }

    // pub fn insert_char(&mut self, pos: UVec2, ch: char) {
    //     self.mark_dirty();
    //     if let Some(line) = self.content.get_mut(pos.y) {
    //         line.insert(pos.x, ch);
    //     }
    // }

    // pub fn replace_char(&mut self, pos: UVec2, ch: char) -> Option<char> {
    //     self.mark_dirty();
    //     if let Some(line) = self.content.get(pos.y) {
    //         let mut chars = line.chars().collect::<Vec<char>>();
    //         let old = chars[pos.x];
    //         chars[pos.x] = ch;
    //         self.content[pos.y] = chars.iter().collect();
    //         return Some(old);
    //     }
    //     None
    // }

    // pub fn replace_content(&mut self, content: String) -> String {
    //     let old = self.content.clone();
    //     self.content = content.split('\n').map(|line| line.to_string()).collect();
    //     old.join("\n")
    // }

    // pub fn remove_char(&mut self, pos: UVec2) -> Option<char> {
    //     self.mark_dirty();
    //     if let Some(line) = self.content.get_mut(pos.y) {
    //         if pos.x < line.len() {
    //             return Some(line.remove(pos.x));
    //         } else if pos.x == line.len() {
    //             self.join_lines(pos.y);
    //             return Some('\n');
    //         }
    //     }
    //     None
    // }

    // pub fn insert_line(&mut self, y: usize, line: String) {
    //     self.mark_dirty();
    //     self.content.insert(y, line);
    // }

    // pub fn replace_line(&mut self, y: usize, line: String) -> Option<String> {
    //     if let Some(old_line) = self.get_line(y) {
    //         self.mark_dirty();
    //         self.content[y] = line;
    //         Some(old_line)
    //     } else {
    //         None
    //     }
    // }

    // pub fn replace_all_lines(&mut self, lines: Vec<String>) -> Vec<String> {
    //     let old = self.content.clone();
    //     self.content = lines;
    //     old
    // }

    // pub fn remove_line(&mut self, y: usize) -> Option<String> {
    //     let line_count = self.get_line_count();
    //     if line_count != 0 && y < line_count {
    //         self.mark_dirty();
    //         Some(self.content.remove(y))
    //     } else {
    //         None
    //     }
    // }

    // pub fn split_line(&mut self, pos: UVec2) {
    //     self.mark_dirty();

    //     let original = self.content[pos.y].clone();
    //     let (p0, p1) = original.split_at(pos.x);
    //     self.content[pos.y] = p0.to_string();
    //     self.content.insert(pos.y + 1, p1.to_string());
    // }

    // pub fn join_lines(&mut self, y: usize) {
    //     if y + 1 < self.get_line_count() {
    //         self.mark_dirty();

    //         let combined = self.content[y].clone() + &self.content[y + 1];
    //         self.content[y] = combined;
    //         self.content.remove(y + 1);
    //     }
    // }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            file: None,
            content: BufferContent::new(),
            dirty: false,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_insert_remove_char() {
//         let mut buf = Buffer::default();
//         buf.insert_line(0, "Hello".to_string());
//         buf.insert_line(1, "World".to_string());

//         assert_eq!(buf.get_char(UVec2::new(0, 0)), Some('H'));
//         assert_eq!(buf.get_char(UVec2::new(4, 0)), Some('o'));
//         assert_eq!(buf.get_char(UVec2::new(0, 1)), Some('W'));
//         assert_eq!(buf.get_char(UVec2::new(4, 1)), Some('d'));
//         assert_eq!(buf.get_char(UVec2::new(5, 1)), None);

//         assert_eq!(buf.remove_char(UVec2::new(1, 0)), Some('e'));
//         assert_eq!(buf.get_line(0), Some("Hllo".to_string()));
//         assert_eq!(buf.remove_char(UVec2::new(10, 0)), None);
//     }

//     #[test]
//     fn test_insert_remove_line() {
//         let mut buf = Buffer::default();
//         assert_eq!(buf.remove_line(0), Some("".to_string()));
//         assert_eq!(buf.remove_line(1), None);

//         buf.insert_line(0, "first line".to_string());
//         buf.insert_line(1, "second line".to_string());

//         assert_eq!(buf.get_line_count(), 2);
//         assert_eq!(buf.get_line(0), Some("first line".to_string()));
//         assert_eq!(buf.get_line(1), Some("second line".to_string()));

//         assert_eq!(buf.remove_line(0), Some("first line".to_string()));
//         assert_eq!(buf.remove_line(0), Some("second line".to_string()));
//         assert_eq!(buf.remove_line(0), None);
//         assert_eq!(buf.get_line_count(), 0);
//     }

//     #[test]
//     fn test_split_join_line() {
//         let mut buf = Buffer::default();
//         buf.replace_line(0, "HelloWorld".to_string());

//         buf.split_line(UVec2::new(5, 0));
//         assert_eq!(buf.get_line_count(), 2);
//         assert_eq!(buf.get_line(0), Some("Hello".to_string()));
//         assert_eq!(buf.get_line(1), Some("World".to_string()));

//         buf.join_lines(0);
//         assert_eq!(buf.get_line_count(), 1);
//         assert_eq!(buf.get_line(0), Some("HelloWorld".to_string()));
//     }
// }
