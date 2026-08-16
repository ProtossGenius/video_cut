/// 定义按键被解析后转化为的具体行为
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // ==== 播放控制 ====
    PlayPause,

    // ==== 移动控制 ====
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    JumpNextAudio,
    JumpPrevAudio,
    JumpNextClipEdge,
    JumpPrevClipEdge,

    // ==== 编辑操作 ====
    Split,
    Delete,
    Yank,
    Paste,

    // ==== 模式切换 ====
    EnterVisual,
    EnterVisualLine,
    EnterCommand,
    EnterSearch,
    EscapeToNormal,

    // ==== 动态执行 (Lua绑定) ====
    /// 触发一个 Lua 脚本绑定的命名命令
    LuaCommand(String),
}
