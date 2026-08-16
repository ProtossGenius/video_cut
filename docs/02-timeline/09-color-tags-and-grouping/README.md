# 轨道与切片色彩标签及多选编组系统 (Color Tags & Multi-Clip Grouping Pipeline)

支持为时间线轨道及切片赋予色彩标签分类管理，并支持将多个切片编组成组进行联动。

## 色彩标签预设 (ColorTagPreset)
提供 8 种高对比度色彩标签：
- `None` (默认主题色)
- `Rose` (绯红 / 玫红)
- `Orange` (橙黄)
- `Amber` (琥珀金)
- `Emerald` (翡翠绿)
- `Cyan` (青蓝)
- `Blue` (天蓝)
- `Purple` (紫罗兰)

## 切片多选编组 (Clip Grouping)
- **编组机制**：切片包含可选的 `group_id: Option<u64>`。同一组内的切片在时间线上具有统一的高亮编组徽章（`🔗 Group #X`）。
- **联动操作**：当选定或移动组内任一切片时，同组内的其他切片自动联动同步位移。
- **交互指令**：
  - `:group` / `:g`：将当前选区（Visual Line / Multi-selection）内的所有切片绑定为同一个新编组。
  - `:ungroup` / `:ug`：将当前切片或选区内的切片解除编组。
  - `:color <color_name>` / `:tag <color_name>`：为当前切片设置色彩标签。
- **右键上下文菜单**：
  - 增加 `🎨 标记色彩标签` 与 `🔗 编组/解除编组` 菜单项。
