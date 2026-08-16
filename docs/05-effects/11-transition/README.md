# 视频转场特效引擎 (Video Transitions Engine)

视频转场用于在时间线相邻的两个切片之间创建平滑自然的画面过渡。

## 转场类型 (`TransitionType`)
- **交叉溶解 (`CrossDissolve`)**：前后两段素材透明度平滑线性/余弦渐变插值交叠。
- **左划像 (`WipeLeft`)**：后一段素材自右向左推动划入覆盖。
- **右划像 (`WipeRight`)**：后一段素材自左向右推动划入覆盖。
- **闪黑/淡入淡出黑场 (`DipToBlack`)**：前一段画面淡出至纯黑，后一段画面自黑场淡入。
- **闪白 (`DipToWhite`)**：前一段画面过曝冲白，后一段画面自白场恢复。

## 转场对齐模式 (`TransitionAlignment`)
- **居中对齐 (`Center`)**：转场区域对称跨越前后两个切片的接缝点（默认）。
- **起点对齐 (`StartOnCut`)**：转场自接缝点向后延伸。
- **终点对齐 (`EndOnCut`)**：转场自接缝点向前延伸。

## 交互命令与上下文菜单
- `:transition <type> [duration]`：在当前播放头所在切片与下一相邻切片之间创建转场。
  - 例如：`:transition dissolve 1.0`（创建 1 秒交叉溶解）。
  - 例如：`:transition wipe_left 0.8`（创建 0.8 秒左划像）。
  - 例如：`:transition dip_black 0.5`（创建 0.5 秒黑场闪烁）。
  - 例如：`:transition remove` / `:transition none`（移除当前切片尾部转场）。
- 在切片右键菜单中提供 `✨ 视频转场特效` 独立子菜单，支持快速一键添加与清除转场。
- 在时间线接缝处高亮渲染转场梯形连接带与转场类型图标 Badge。
