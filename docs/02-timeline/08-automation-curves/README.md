# 自动化关键帧属性包络线 (Automation Keyframe Envelope Visualizer on Timeline)

支持在时间线切片卡片上直接图形化绘制属性关键帧节点与包络折线（如音量淡入淡出包络、透明度渐变、缩放变换曲线等）。

## 核心设计
1. **关键帧轨道数据 (`ClipKeyframeTrack`)**：
   - 存储属性类型（`Volume` / `Opacity` / `Scale` / `Rotation`）。
   - 包含若干有序的关键帧点 `(offset_us, value)`。
2. **时间线切片叠加渲染**：
   - 将 `0.0 ~ 1.0` 的属性值映射至切片卡片的高度范围，使用平滑折线/曲线连接各个关键帧。
   - 关键帧处绘制菱形/圆形高亮控制节点。
3. **交互式控制与命令行**：
   - `:keyframe <prop> <value>` / `:kf <prop> <value>`：在当前播放头位置为当前切片打上属性关键帧。
   - `:clearkf` / `:clear_keyframes`：清除当前切片的关键帧包络。
   - 右键菜单提供 `📈 插入属性关键帧` 与 `🗑 清除属性关键帧`。
