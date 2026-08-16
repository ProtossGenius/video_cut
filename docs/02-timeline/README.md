# 时间线系统

本模块定义了视频编辑的核心领域模型，采用非破坏性编辑模式。
子模块包含：
- [轨道管理](./01-track/README.md)
- [切片管理](./02-clip/README.md)
- [锚点系统](./03-anchor/README.md)
- [选择模式](./04-selection/README.md)
- [垃圾轨道](./05-trash-track/README.md)

## 核心数据结构

```rust
// 伪代码表示
pub struct Timeline {
    pub duration: Duration,
    pub tracks: Vec<Track>,
    pub trash_track: Track,
    // 快速查找某个时间点有哪些切片重叠，使用区间树优化
    pub interval_index: IntervalTree<Frame, ClipId>, 
}
```
