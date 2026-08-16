# 轨道管理

轨道（Track）是横向排布时间内容的容器。

## 结构定义
```rust
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub clips: Vec<Clip>,      // 按 timeline_start 排序
    pub is_pinned: bool,       // 是否置顶
    pub pin_order: Option<u32>,// 置顶顺序
}
```

## 视图呈现
一条标准的轨道使用 `egui::Painter` 进行定制化绘制：
- 上半部分：视频帧的等间隔缩略图连屏（Strip）。
- 下半部分：音频波形图渲染（Waveform）。
- 顶部边界：1 px 宽的特殊行，用于绘制锚点标记（小倒三角或字符）。

## 操作指令
- `Shift + Enter`：在当前光标所在的轨道**下方**新建一条空白轨道。
- `Alt + j/k`：将当前轨道向下/向上移动一个层级。
- `^`：将当前轨道设为**置顶**（若已有置顶轨道，则排列在其最下方）。再按一次变为绝对最顶。光标焦点随轨道一起移动。
- `$`：取消当前轨道的置顶状态。置顶轨道依然可以用 `Alt + j/k` 互相改变顺序。
