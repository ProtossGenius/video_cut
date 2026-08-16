use std::collections::HashMap;

use super::action::Action;

/// 表示一次单独的按键输入
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    // 简化：这里只用 String 代表按键名字，例如 "v", "Shift+Enter", "Space"
    // 实际应使用 egui::Key + Modifiers，为了保持纯净领域模型，这里做了抽象
    pub key: String,
}

impl KeyEvent {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

/// 字典树节点
#[derive(Debug, Clone)]
pub enum KeyTrieNode {
    /// 叶子节点，解析成功，触发特定 Action
    Leaf(Action),
    /// 分支节点，等待后续按键
    Branch(HashMap<KeyEvent, KeyTrieNode>),
    // TODO: Parameterized(fn(KeyEvent) -> Option<Action>), 处理如 m<char> 等带参逻辑
}

/// 快捷键映射树
#[derive(Debug, Clone)]
pub struct KeymapTrie {
    pub root: KeyTrieNode,
}

impl Default for KeymapTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl KeymapTrie {
    pub fn new() -> Self {
        Self {
            root: KeyTrieNode::Branch(HashMap::new()),
        }
    }

    /// 将按键序列绑定到具体的行为
    /// 比如将 `["v", "l"]` 绑定到某个 Action
    pub fn insert(&mut self, keys: &[KeyEvent], action: Action) {
        let mut current = &mut self.root;

        for (i, key) in keys.iter().enumerate() {
            let is_last = i == keys.len() - 1;

            if !matches!(current, KeyTrieNode::Branch(_)) {
                *current = KeyTrieNode::Branch(HashMap::new());
            }

            if let KeyTrieNode::Branch(children) = current {
                if is_last {
                    children.insert(key.clone(), KeyTrieNode::Leaf(action.clone()));
                    break;
                } else {
                    current = children
                        .entry(key.clone())
                        .or_insert_with(|| KeyTrieNode::Branch(HashMap::new()));
                }
            }
        }
    }
}

/// 解析器状态结果
#[derive(Debug, PartialEq, Eq)]
pub enum ParseResult {
    /// 成功命中，返回需要执行的动作和之前积累的前缀数字
    Matched(Action, usize),
    /// 前缀命中，还在等待用户进一步敲击
    Pending,
    /// 没有匹配
    NotFound,
}

/// 状态机中的按键解析器
pub struct KeyParser {
    trie: KeymapTrie,
    /// 缓冲区：存储当前还没解析完成的按键序列
    pending_keys: Vec<KeyEvent>,
    /// 记录按键序列前面的数字，例如 `v 5 l` 里的 `5`
    count: usize,
}

impl KeyParser {
    pub fn new(trie: KeymapTrie) -> Self {
        Self {
            trie,
            pending_keys: Vec::new(),
            count: 0,
        }
    }

    /// 输入一个新的按键
    pub fn handle_key(&mut self, key: KeyEvent) -> ParseResult {
        // 如果缓冲区为空，且输入的是数字 1-9，则作为 count 乘数累加
        if self.pending_keys.is_empty() && key.key.len() == 1 {
            let c = key.key.chars().next().unwrap();
            if c.is_ascii_digit() && (c != '0' || self.count > 0) {
                self.count = self.count * 10 + (c.to_digit(10).unwrap() as usize);
                return ParseResult::Pending;
            }
        }

        self.pending_keys.push(key);

        let mut current = &self.trie.root;
        for pk in &self.pending_keys {
            match current {
                KeyTrieNode::Branch(children) => {
                    if let Some(next) = children.get(pk) {
                        current = next;
                    } else {
                        // 序列断裂，清空
                        self.clear();
                        return ParseResult::NotFound;
                    }
                }
                KeyTrieNode::Leaf(_) => {
                    // 如果中途就碰到了Leaf，说明定义有冲突，通常不会走到这里
                    break;
                }
            }
        }

        match current {
            KeyTrieNode::Leaf(action) => {
                let cnt = if self.count == 0 { 1 } else { self.count };
                let res = ParseResult::Matched(action.clone(), cnt);
                self.clear();
                res
            }
            KeyTrieNode::Branch(_) => {
                // 还在树枝上，等待下一个按键
                ParseResult::Pending
            }
        }
    }

    pub fn clear(&mut self) {
        self.pending_keys.clear();
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parser_simple_and_count() {
        let mut trie = KeymapTrie::new();
        // 绑定 'l' 为 MoveRight
        trie.insert(&[KeyEvent::new("l")], Action::MoveRight);
        // 绑定 'v', 'l' 为特殊动作（随便举例，方便测试深层路径）
        trie.insert(
            &[KeyEvent::new("v"), KeyEvent::new("l")],
            Action::EnterVisualLine,
        );

        let mut parser = KeyParser::new(trie);

        // 1. 测试单键
        let res = parser.handle_key(KeyEvent::new("l"));
        assert_eq!(res, ParseResult::Matched(Action::MoveRight, 1));

        // 2. 测试带数字前缀的单键: "5", "l"
        assert_eq!(parser.handle_key(KeyEvent::new("5")), ParseResult::Pending);
        assert_eq!(
            parser.handle_key(KeyEvent::new("l")),
            ParseResult::Matched(Action::MoveRight, 5)
        );

        // 3. 测试多键组合: "v", "l"
        assert_eq!(parser.handle_key(KeyEvent::new("v")), ParseResult::Pending);
        assert_eq!(
            parser.handle_key(KeyEvent::new("l")),
            ParseResult::Matched(Action::EnterVisualLine, 1)
        );
    }
}
