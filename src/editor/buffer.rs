use ropey::Rope;

/// 编辑器文本操作命令（用于文本内 Undo/Redo）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEditAction {
    Insert { char_pos: usize, text: String },
    Delete { char_pos: usize, text: String },
}

/// 基于 Ropey 的高性能文本缓冲区
#[derive(Debug, Clone)]
pub struct TextBuffer {
    rope: Rope,
    undo_stack: Vec<TextEditAction>,
    redo_stack: Vec<TextEditAction>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rope)
    }
}

impl From<&str> for TextBuffer {
    fn from(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        Self::from(text)
    }

    /// 总字符数
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// 总行数
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// 在指定字符位置插入文本
    pub fn insert(&mut self, char_pos: usize, text: &str) {
        let pos = char_pos.min(self.rope.len_chars());
        self.rope.insert(pos, text);
        self.undo_stack.push(TextEditAction::Insert {
            char_pos: pos,
            text: text.to_string(),
        });
        self.redo_stack.clear();
    }

    /// 删除字符区间 [start_pos, end_pos)
    pub fn delete(&mut self, start_pos: usize, end_pos: usize) {
        let start = start_pos.min(self.rope.len_chars());
        let end = end_pos.min(self.rope.len_chars());
        if start >= end {
            return;
        }

        let deleted_text = self.rope.slice(start..end).to_string();
        self.rope.remove(start..end);
        self.undo_stack.push(TextEditAction::Delete {
            char_pos: start,
            text: deleted_text,
        });
        self.redo_stack.clear();
    }

    /// 获取特定行内容
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx).to_string())
        } else {
            None
        }
    }

    /// 行列坐标转为一维字符索引
    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        if line >= self.rope.len_lines() {
            return self.rope.len_chars();
        }
        let line_start_char = self.rope.line_to_char(line);
        let line_len = self.rope.line(line).len_chars();
        line_start_char + col.min(line_len)
    }

    /// 一维字符索引转为 (行号, 列号)
    pub fn char_to_line_col(&self, char_pos: usize) -> (usize, usize) {
        let pos = char_pos.min(self.rope.len_chars());
        let line = self.rope.char_to_line(pos);
        let line_start_char = self.rope.line_to_char(line);
        let col = pos - line_start_char;
        (line, col)
    }

    /// 撤销上一步文本编辑
    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            match &action {
                TextEditAction::Insert { char_pos, text } => {
                    let end = char_pos + text.chars().count();
                    self.rope.remove(*char_pos..end);
                }
                TextEditAction::Delete { char_pos, text } => {
                    self.rope.insert(*char_pos, text);
                }
            }
            self.redo_stack.push(action);
            true
        } else {
            false
        }
    }

    /// 重做上一步文本编辑
    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            match &action {
                TextEditAction::Insert { char_pos, text } => {
                    self.rope.insert(*char_pos, text);
                }
                TextEditAction::Delete { char_pos, text } => {
                    let end = char_pos + text.chars().count();
                    self.rope.remove(*char_pos..end);
                }
            }
            self.undo_stack.push(action);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_insert_delete_undo_redo() {
        let mut buf = TextBuffer::from_text("Hello World");
        assert_eq!(buf.len_chars(), 11);

        // 插入 ", Rust"
        buf.insert(5, ", Rust");
        assert_eq!(format!("{}", buf), "Hello, Rust World");

        // 删除 " World"
        buf.delete(11, 17);
        assert_eq!(format!("{}", buf), "Hello, Rust");

        // 撤销删除
        assert!(buf.undo());
        assert_eq!(format!("{}", buf), "Hello, Rust World");

        // 撤销插入
        assert!(buf.undo());
        assert_eq!(format!("{}", buf), "Hello World");

        // 重做插入
        assert!(buf.redo());
        assert_eq!(format!("{}", buf), "Hello, Rust World");

        // 重做删除
        assert!(buf.redo());
        assert_eq!(format!("{}", buf), "Hello, Rust");
    }

    #[test]
    fn test_text_buffer_line_col_conversion() {
        let text = "Line 1\nLine 2\nLine 3";
        let buf = TextBuffer::from_text(text);

        assert_eq!(buf.len_lines(), 3);
        assert_eq!(buf.line(0), Some("Line 1\n".into()));
        assert_eq!(buf.line(1), Some("Line 2\n".into()));
        assert_eq!(buf.line(2), Some("Line 3".into()));

        // (line 1, col 2) -> "Line 1\nLi" -> 7 + 2 = 9
        let char_idx = buf.line_col_to_char(1, 2);
        assert_eq!(char_idx, 9);

        let (l, c) = buf.char_to_line_col(9);
        assert_eq!((l, c), (1, 2));
    }
}
