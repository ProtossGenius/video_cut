# 颜色分级与 LUT 滤镜预设管线 (Color Grading & 3D LUT Presets)

在视频剪辑和合成渲染管线中，画面色彩调整通过 WGSL Shader 与渲染合成器实现。

## 色彩属性
每个切片（`Clip`）支持基础颜色分级参数：
- **亮度 (`color_brightness`)**：范围 `-1.0 ~ 1.0`（默认 `0.0`），如 `:brightness 0.1` 提升整体画面明度。
- **对比度 (`color_contrast`)**：范围 `0.0 ~ 3.0`（默认 `1.0`），如 `:contrast 1.2` 提升画面对比度。
- **饱和度 (`color_saturation`)**：范围 `0.0 ~ 3.0`（默认 `1.0`），`0.0` 为纯黑白单色，`1.5` 为鲜艳色彩。
- **色温 (`color_temperature`)**：范围 `-1.0 ~ 1.0`（默认 `0.0`），负值偏冷蓝调，正值偏暖橙调。
- **LUT 预设 (`color_lut_preset`)**：内置常用电影级调色预设（如 `Vibrant` 鲜艳、`CinematicWarm` 暖色电影感、`TealOrange` 青橙调、`Monochrome` 灰度黑白、`Vintage` 复古胶片）。

## 交互命令与上下文菜单
- `:brightness <val>`：调节亮度（例如 `:brightness 0.2`）。
- `:contrast <val>`：调节对比度（例如 `:contrast 1.2`）。
- `:saturation <val>`：调节饱和度（例如 `:saturation 1.5` 或 `:saturation 0.0` 变黑白）。
- `:temp <val>`：调节色温（例如 `:temp 0.3` 暖调，`:temp -0.3` 冷调）。
- `:lut <preset>`：应用色彩预设（如 `:lut teal_orange`、`:lut cinematic`、`:lut vibrant`、`:lut bw`）。
- `:reset_color`：重置当前切片的所有调色参数至默认值。
- 在切片右键菜单中提供 `🎨 颜色分级与滤镜` 独立子菜单，支持一键选取预设与重置。
