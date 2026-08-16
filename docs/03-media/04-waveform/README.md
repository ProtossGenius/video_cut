# 波形图生成

音频可视化数据的预处理机制。

## 多分辨率金字塔 LOD
音频一秒钟有 48000 个采样点，无法在每一帧实时绘制。必须预处理为多个精度的聚合数据：
- L0: `100 samples/pt` （供极大放大时查看）
- L1: `1000 samples/pt` （常规缩放层级）
- L2: `10000 samples/pt` （全局概览层级）

## 聚合算法
使用 `symphonia` 解码出 PCM 后，每隔固定窗口大小的 samples 计算其：
- 最小值 `min` （向下绘制多深）
- 最大值 `max` （向上绘制多高）
将这对峰值浮点数作为顶点，交由 UI 的 `egui::Painter::lines` 绘制出致密的线条矩阵。

## 音频瞬态与节奏能量峰值检测 (Audio Transient Detection)
- **瞬态能量计算**：通过短时能量导数（Spectral Flux / RMS Derivative: $\Delta E = E(t) - E(t-1)$）和阈值过滤，自动提取出音频切片中的重音拍点与瞬态打击点（Beats & Transient Onsets）。
- **节奏吸附联动**：提取出的瞬态打击点作为 `SnapTargetKind::AudioTransient` 注册至时间线吸附引擎 `SnapEngine` 中，在时间线上呈现青色辅助垂直线，助力卡点剪辑。

