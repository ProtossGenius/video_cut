use eframe::egui::Color32;

/// 语法高亮 Token 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Builtin,
    String,
    Number,
    Comment,
    Function,
    Operator,
    Punctuation,
    Identifier,
    Default,
}

impl TokenType {
    pub fn color(&self) -> Color32 {
        match self {
            TokenType::Keyword => Color32::from_rgb(197, 134, 192), // 紫色
            TokenType::Builtin => Color32::from_rgb(78, 201, 176),  // 青绿
            TokenType::String => Color32::from_rgb(206, 145, 120),  // 暖橙
            TokenType::Number => Color32::from_rgb(181, 206, 168),  // 嫩绿
            TokenType::Comment => Color32::from_rgb(106, 153, 85),  // 暗绿
            TokenType::Function => Color32::from_rgb(220, 220, 170), // 淡黄
            TokenType::Operator => Color32::from_rgb(212, 212, 212), // 浅灰
            TokenType::Punctuation => Color32::from_rgb(160, 160, 160), // 中灰
            TokenType::Identifier => Color32::from_rgb(156, 220, 254), // 天蓝
            TokenType::Default => Color32::from_rgb(220, 220, 220), // 默认白灰
        }
    }
}

/// 高亮文本片段
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSpan {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

/// Lua 语法高亮解析器
pub struct LuaSyntaxHighlighter {
    parser: tree_sitter::Parser,
}

impl Default for LuaSyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaSyntaxHighlighter {
    pub fn new() -> Self {
        let mut parser = tree_sitter::Parser::new();
        let language: tree_sitter::Language = tree_sitter_lua::LANGUAGE.into();
        let _ = parser.set_language(&language);
        Self { parser }
    }

    /// 对输入的 Lua 源代码进行语法解析并生成高亮 Token 列表
    pub fn highlight(&mut self, source_code: &str) -> Vec<HighlightedSpan> {
        let tree = match self.parser.parse(source_code, None) {
            Some(t) => t,
            None => {
                return vec![HighlightedSpan {
                    start: 0,
                    end: source_code.len(),
                    token_type: TokenType::Default,
                }];
            }
        };

        let mut spans = Vec::new();
        let cursor = tree.walk();

        Self::traverse_node(&cursor.node(), source_code, &mut spans);
        spans
    }

    fn traverse_node(node: &tree_sitter::Node, source: &str, spans: &mut Vec<HighlightedSpan>) {
        let kind = node.kind();
        let start = node.start_byte();
        let end = node.end_byte();

        if kind == "comment" || kind.contains("comment") {
            spans.push(HighlightedSpan {
                start,
                end,
                token_type: TokenType::Comment,
            });
            return;
        }

        if kind == "string" || kind.contains("string") || kind == "string_content" {
            spans.push(HighlightedSpan {
                start,
                end,
                token_type: TokenType::String,
            });
            return;
        }

        // 检查叶子节点或特殊语法节点
        if node.child_count() == 0 {
            let token_type = match kind {
                "comment" => TokenType::Comment,
                "string" => TokenType::String,
                "number" | "number_literal" => TokenType::Number,
                "nil" | "true" | "false" | "function" | "local" | "if" | "then" | "else"
                | "elseif" | "end" | "return" | "for" | "while" | "do" | "in" | "repeat"
                | "until" | "not" | "and" | "or" | "goto" | "break" => TokenType::Keyword,
                "identifier" | "property_identifier" => {
                    let text = &source[start..end];
                    if is_builtin_symbol(text) {
                        TokenType::Builtin
                    } else {
                        TokenType::Identifier
                    }
                }
                "+" | "-" | "*" | "/" | "%" | "^" | "#" | "==" | "~=" | "<=" | ">=" | "<" | ">"
                | "=" | ".." => TokenType::Operator,
                "(" | ")" | "{" | "}" | "[" | "]" | ";" | ":" | "," | "." => TokenType::Punctuation,
                _ => TokenType::Default,
            };

            spans.push(HighlightedSpan {
                start,
                end,
                token_type,
            });
        } else {
            // 遍历所有子节点
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                Self::traverse_node(&child, source, spans);
            }
        }
    }
}

fn is_builtin_symbol(name: &str) -> bool {
    matches!(
        name,
        "begin_animation"
            | "end_animation"
            | "begin_camera"
            | "end_camera"
            | "frame"
            | "actions"
            | "moveto"
            | "resize"
            | "rect"
            | "focuse"
            | "main"
            | "bind_key"
            | "bind_group_desc"
            | "print"
            | "math"
            | "string"
            | "table"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_syntax_highlighting() {
        let mut highlighter = LuaSyntaxHighlighter::new();
        let code = r#"
            -- This is a comment
            local speed = 1.5
            begin_animation()
            frame('1:00')
        "#;

        let spans = highlighter.highlight(code);
        assert!(!spans.is_empty());

        // 验证存在注释、local 关键字、数字、内置函数与字符串
        let has_comment = spans.iter().any(|s| s.token_type == TokenType::Comment);
        let has_keyword = spans.iter().any(|s| s.token_type == TokenType::Keyword);
        let has_number = spans.iter().any(|s| s.token_type == TokenType::Number);
        let has_builtin = spans.iter().any(|s| s.token_type == TokenType::Builtin);
        let has_string = spans.iter().any(|s| s.token_type == TokenType::String);

        assert!(has_comment, "Should highlight comments");
        assert!(has_keyword, "Should highlight keywords (local)");
        assert!(has_number, "Should highlight numbers (1.5)");
        assert!(
            has_builtin,
            "Should highlight builtins (begin_animation/frame)"
        );
        assert!(has_string, "Should highlight strings ('1:00')");
    }
}
