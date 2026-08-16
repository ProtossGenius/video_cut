use anyhow::Result;
use std::collections::VecDeque;

use super::state::ProjectState;

/// 编辑器命令 trait，所有的剪辑操作必须实现此 trait
pub trait EditorCommand: Send + Sync {
    /// 执行命令，改变项目状态
    fn execute(&mut self, state: &mut ProjectState) -> Result<()>;
    
    /// 撤销命令，恢复状态
    fn undo(&mut self, state: &mut ProjectState) -> Result<()>;
    
    /// 尝试与上一个命令合并（对于连续相同且可压缩的微小操作）
    /// 返回 true 表示合并成功，该命令可以被丢弃
    fn merge(&mut self, _other: &dyn EditorCommand) -> bool {
        false
    }
}

/// 历史栈管理
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
    /// WAL (Write-Ahead Log) 队列，等待刷入磁盘的序列化操作日志
    /// 这里先留一个占位符，实际会是一个序列化结构
    pub pending_wal: VecDeque<String>, 
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            pending_wal: VecDeque::new(),
        }
    }
    
    /// 执行并记录新命令
    pub fn execute(&mut self, mut cmd: Box<dyn EditorCommand>, state: &mut ProjectState) -> Result<()> {
        cmd.execute(state)?;
        self.redo_stack.clear(); // 新操作打断了redo链
        self.undo_stack.push(cmd);
        // TODO: append to WAL
        Ok(())
    }
    
    pub fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state)?;
            self.redo_stack.push(cmd);
            // TODO: write undo log to WAL
        }
        Ok(())
    }
    
    pub fn redo(&mut self, state: &mut ProjectState) -> Result<()> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(state)?;
            self.undo_stack.push(cmd);
            // TODO: write redo log to WAL
        }
        Ok(())
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
