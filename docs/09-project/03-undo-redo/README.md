# 撤销重做架构

建立在 `Command Pattern` 设计模式上。
每一个导致状态（包含剪切、移动、属性变化）发生变化的行为，必须要求开发者继承并实现 `EditorCommand` trait：
```rust
pub trait EditorCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()>;
    fn undo(&mut self, state: &mut ProjectState) -> Result<()>;
    // 对于连续相同的快速微调操作进行合并，不留废墟命令栈
    fn merge(&mut self, other: &dyn EditorCommand) -> bool { false }
}
```

内存中持久化维护两个栈数组 `undo_stack` 和 `redo_stack` 即可。
