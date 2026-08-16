# 遮罩系统

基于颜色键控（Chroma Key）和 Alpha 通道遮盖的处理。

## WGSL 实现
在 GPU fragment shader 级别实现。
通过注入颜色的浮点判断阈值（threshold 与 smoothness），直接修改原始纹理采样的 `out_color.a`。例如：设置提取绿色背景使其透明，在 wgpu 管线内部直接剔除不需要绘制的像素，实现性能最大化。
