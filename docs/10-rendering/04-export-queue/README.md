# 视频导出进度预估与多格式转码队列 (Render Progress ETA & Multi-Format Transcode Queue)

高效管理和并发调度视频渲染导出任务，提供实时的渲染百分比、帧率监控、剩余时间预估（ETA）与多格式批量导出支持。

## 导出预设全景 (ExportPreset)
- `H264Mp4`：H.264 / MP4 (标准 1080p，通用性最佳)
- `HevcMp4`：H.265 / HEVC (超清 4K，极致体积压缩)
- `ProResMov`：Apple ProRes 422 / MOV (广播影视级无损母带)
- `GifAnimation`：GIF 动态表情包 / 动图 (社交平台轻量分享)
- `AudioOnlyAac`：AAC 音频流 / M4A (播客 / 音轨提取)

## 实时进度监控与 ETA 估算
- **帧率监控**：实时采集后台硬件编码器的瞬时渲染帧率 $FPS$。
- **剩余时间预估**：
  $$ \text{ETA} = \frac{\text{TotalFrames} - \text{CurrentFrame}}{FPS} \quad (\text{seconds}) $$
- **多任务队列 (`ExportQueue`)**：支持将多个不同分辨率或不同编码预设的任务按顺序或并发推入队列自动批处理。

## 交互命令与快捷操作
- `:export`：唤起导出设置与实时进度监控弹窗。
- `:export <path> [h264|hevc|prores|gif|audio]`：快速以指定格式添加导出任务。
- `:export_queue` / `:exports`：查看当前转码队列中的所有任务与进度。
- `:cancel_export`：取消当前正在运行的导出任务。
