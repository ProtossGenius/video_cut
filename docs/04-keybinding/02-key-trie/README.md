# 按键 Trie 树

为了支持像 `d d`，`v 5 l` 这种多键组合（Chord）和前缀计数操作，按键映射数据必须存储在字典树中。

## 结构设计
```rust
pub enum KeyTrieNode {
    Leaf(ActionID),
    Branch(HashMap<KeyEvent, KeyTrieNode>),
    // 允许通过闭包执行动态的带参解析
    Parameterized(fn(KeyEvent) -> Option<ActionID>), 
}
```

## 序列处理逻辑
1. 用户按下一个键，存入 `pending_keys` 缓冲区。
2. 在 Trie 树中查找路径。
   - 如果遇到 `Branch`，则等待下一次按键输入（此时若停留时间超过阈值，唤起**帮助面板**展示该前缀下的剩余合法键位）。
   - 如果解析到 `Leaf`，清空缓冲区，生成并派发 `Command`，执行对应逻辑。
   - 如果发生超时（如 500ms 内无动作且序列不合法），抛弃序列，退回初始态。
3. **数字前缀**处理：在识别命令序列之前，先剥离前导阿拉伯数字，存入一个 `count` 乘数器寄存器中。
