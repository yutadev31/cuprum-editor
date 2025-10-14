use ropey::Rope;
use utils::vec2::UVec2;

#[derive(Debug)]
pub struct BufferContent {
    buf: Rope,
}

impl BufferContent {
    pub fn new() -> Self {
        Self { buf: Rope::new() }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            buf: Rope::from_str(text),
        }
    }

    pub fn vec2_to_index(&self, vec2: UVec2) -> usize {
        self.buf.line_to_char(vec2.y) + vec2.x
    }

    pub fn index_to_vec2(&self, index: usize) -> UVec2 {
        let line = self.buf.char_to_line(index);
        let line_start = self.buf.line_to_char(line);
        let x = index - line_start;
        UVec2::new(x, line)
    }

    pub fn get(&self) -> String {
        self.buf.to_string()
    }

    pub fn get_line_start(&self, line: usize) -> usize {
        self.buf.line_to_char(line)
    }

    pub fn get_line_count(&self) -> usize {
        self.buf.len_lines()
    }

    pub fn get_line_length(&self, line: usize) -> usize {
        self.buf.line(line).len_chars()
    }

    pub fn get_char(&self, position: usize) -> char {
        self.buf.char(position)
    }

    pub fn get_line(&self, line: usize) -> String {
        self.buf.line(line).to_string()
    }

    pub fn insert(&mut self, position: usize, text: &str) {
        self.buf.insert(position, text);
    }

    pub fn remove(&mut self, start: usize, end: usize) -> Option<String> {
        let text = self
            .buf
            .get_slice(start..end)
            .map(|slice| slice.to_string());

        self.buf.remove(start..end);

        text
    }

    pub fn remove_char(&mut self, position: usize) -> Option<char> {
        let ch = self.buf.get_char(position);
        self.buf.remove(position..position);
        ch
    }
}
