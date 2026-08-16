# 变换系统

基础的 2D 几何变换效果封装。
底层在渲染合成器中，变换属性通过操作 `wgpu` 的矩阵堆栈实现（MVP 矩阵的 Model Matrix 操作）。支持：
- `Translate`（位移）
- `Scale`（百分比或绝对像素缩放，`:scale <factor>`）
- `Rotate`（绕特定质心点的旋转，`:rotate <deg>`）
- `Flip`（水平/垂直翻转，`:flip [h|v|both]`）
- `Reset`（恢复初始变换，`:reset_transform`）

## 视口实时交互控制器 (Viewport Transform Gizmo)
- **视觉反馈**：在预览播放器视口中选中切片时，在切片周围绘制半透明变换边框与 8 个缩放锚点手柄，以及顶部旋转控制圆环。
- **快捷指令与菜单**：
  - `:rotate <角度>`：如 `:rotate 90`（顺时针旋转90度）、`:rotate -45`。
  - `:scale <倍率>`：如 `:scale 1.5`（放大至150%）、`:scale 0.8`。
  - `:flip [h|v]`：水平或垂直翻转画面。
  - `:reset_transform`：重置所有变换参数。

