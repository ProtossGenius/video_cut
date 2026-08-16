# 视频画中画与分屏排布模板引擎 (Picture-in-Picture & Split Screen Presets)

提供将多条视频轨道的切片一键排布为专业多机位画中画或分屏视口变换参数的管线引擎。

## 预设分屏布局 (PipLayoutPreset)
1. **画中画角落浮窗 (`CornerBR`, `CornerTR`, `CornerBL`, `CornerTL`)**：
   - 默认缩放 `0.35` (35% 画幅)，置于屏幕指定角落并带有安全边距。
2. **左右双分屏 (`SplitHorizontal` / `SideBySide`)**：
   - 双画面水平并排，各自缩放 `0.5`，X 轴偏移 `-25%` 与 `+25%`。
3. **上下双分屏 (`SplitVertical` / `TopBottom`)**：
   - 双画面垂直堆叠，各自缩放 `0.5`，Y 轴偏移 `-25%` 与 `+25%`。
4. **2x2 四宫格分屏 (`Grid2x2` / `Quad`)**：
   - 四画面 2x2 矩阵网格排布，各画面缩放 `0.5` 并分布在四个象限。
5. **画中画居中浮窗 (`CenterFloating`)**：
   - 缩放 `0.5`，居中悬浮于背景画面之上。

## 交互命令与上下文菜单
- `:pip corner [br|tr|bl|tl] [scale]`：将当前切片设置为角落画中画（默认右下角 0.35 缩放）。
- `:pip split_h` 或 `:pip side_by_side`：水平双分屏布局。
- `:pip split_v` 或 `:pip top_bottom`：垂直双分屏布局。
- `:pip grid2x2` 或 `:pip quad`：四宫格分屏。
- `:pip reset` 或 `:reset_pip`：重置当前切片变换为全屏充满。
- 切片右键菜单提供 `🖼 画中画与分屏布局 (PIP & Split Screen)` 独立子菜单。
