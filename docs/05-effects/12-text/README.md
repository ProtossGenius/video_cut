# 字幕与多行富文本气泡渲染器 (Subtitles & Text Overlay Engine)

在视频剪辑中，文本字幕、标题条与贴纸是信息呈现的核心组成部分。

## 文本图层属性 (`TextOverlayParams`)
切片可携带文本覆盖参数：
- **文本内容 (`content`)**：支持单行或多行文本（如 `"Hello World\n欢迎使用 Video Cut"`）。
- **字体字号 (`font_size`)**：范围 `8.0 ~ 120.0` 像素（默认 `24.0`）。
- **文本主色 (`color`)**：RGBA 格式（默认纯白 `(255, 255, 255, 255)`）。
- **描边效果 (`outline_width` / `outline_color`)**：防止文字与背景背景色混淆的黑色高对比度描边。
- **气泡底框 (`has_background` / `bg_color` / `bg_corner_radius` / `bg_padding`)**：电影级半透明圆角气泡字幕底框。
- **排版对齐 (`alignment`)**：底部居中 (`BottomCenter`)、顶部居中 (`TopCenter`)、屏幕居中 (`Center`)、左下 (`BottomLeft`) 等。

## 交互命令与上下文菜单
- `:text <文本内容>`：为当前切片设置或更新显示文本。
- `:fontsize <px>`：设置文本字号（如 `:fontsize 32`）。
- `:textcolor <hex/color>`：设置文字颜色（如 `:textcolor #ffff00`、`:textcolor white`、`:textcolor yellow`）。
- `:bgbox [on|off]`：开启或关闭字幕气泡半透明底框（`:bgbox on` / `:bgbox off`）。
- `:clear_text`：清除当前切片的文本覆盖。
- 在切片右键菜单中提供 `💬 文本与字幕 (Subtitles)` 子菜单，支持快捷设置气泡框与常用字号。
