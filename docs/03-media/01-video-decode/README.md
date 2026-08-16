# 视频解码

基于 `ffmpeg-next` (v9.0.0) 和 `ffmpeg-sys-next` 构建。

## 解码架构
1. **多线程解码池**：解码不能阻塞 UI 和时间线的主控制线程，在独立的线程池中并行解码多个轨道的切片。
2. **缓冲队列**：解码后的 YUV/NV12 原始帧被推送到有界通道（Bounded Channel），等待 UI 线程取用，避免积压造成内存 OOM。

## 帧精确定位 (Frame-Accurate Seeking)
NLE 的核心在于不能像普通播放器那样有误差。
- **步骤 1**：如果目标帧 `N` 不是关键帧（I-frame），则调用 `av_seek_frame` 带着 `AVSEEK_FLAG_BACKWARD` 标志，跳到 `N` 之前的最近的一个 I-frame。
- **步骤 2**：静默解码丢弃从 I-frame 到 `N-1` 之间的所有帧。
- **步骤 3**：获取准确的第 `N` 帧返回。
