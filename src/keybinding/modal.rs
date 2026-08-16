use serde::{Deserialize, Serialize};

/// 代表系统的各个输入状态模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Mode {
    #[default]
    Normal, // 默认编辑模式
    Visual,     // (v 小写) 范围选择模式
    VisualLine, // (V 大写) 切片选择模式
    Command,    // (:) 命令行输入模式
    Insert,     // (i) 文本编辑器或标题输入模式
    Editor,     // 纯 Lua 代码编辑面板模式
    Search,     // (/) 项目或资源搜索模式
    Mark,       // (m) 正在等待输入锚点名称状态
    Goto,       // (') 正在等待跳转锚点按键状态
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
            Mode::Command => "COMMAND",
            Mode::Insert => "INSERT",
            Mode::Editor => "EDITOR",
            Mode::Search => "SEARCH",
            Mode::Mark => "MARK",
            Mode::Goto => "GOTO",
        }
    }
}
