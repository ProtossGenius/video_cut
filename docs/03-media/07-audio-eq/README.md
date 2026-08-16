# 音频三段参数均衡器与 EBU R128 响度标准化

VideoCut 在轨道级音频管线中补入了**低频 / 中频 / 高频三段参数均衡器**与**-14 LUFS / -23 LUFS** 一键响度目标。

## 3-Band Parametric EQ
- **低频 (`low`)**：Low Shelf，默认聚焦 120Hz，适合收低切泥、控制轰头。
- **中频 (`mid`)**：Peaking，默认聚焦 1.8kHz，适合提升人声存在感与对白清晰度。
- **高频 (`high`)**：High Shelf，默认聚焦 8.5kHz，适合补空气感和齿音亮度。

命令示例：
- `:eq preset podcast`
- `:eq low 90 -3.5 0.8`
- `:eq mid 2200 2.5 1.2`
- `:eq high 9500 1.8 0.7`

## EBU R128 Loudness
- `:loudnorm stream`：面向主流流媒体平台，目标 **-14 LUFS**
- `:loudnorm broadcast`：面向广播交付，目标 **-23 LUFS**
- `:loudnorm off`：关闭当前轨道的响度标准化补偿

系统会记录：
1. 目标响度
2. 估算 / 测得的输入 LUFS
3. 推荐补偿增益 dB

## UI 集成
- 左侧轨道表头显示 `EQ Flat`、`EQ -3 / +2 / +1` 一类摘要
- 同一行显示 `-14 LUFS +4.1dB` 或 `Loudness Off`
- 轨道右键菜单提供 Podcast / Vocal / Bass Cut / Bright 等快速预设
