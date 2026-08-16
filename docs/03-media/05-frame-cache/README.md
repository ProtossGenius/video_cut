# 帧缓存策略 (Frame Cache)

为实现实时流畅 scrubbing（倒播/快播），必须在内存中缓冲视频帧。

## 三级缓存架构
1. **L1 纹理池 (GPU Texture Pool)**
   - 处于显存中。避免每渲染一帧就创建和销毁 `wgpu::Texture`。系统会持有几个常驻的 Texture 壳子，利用 `queue.write_texture()` 每帧仅覆盖数据，实现极低开销的 GPU blit。
2. **L2 RAM 解码帧 (moka LRU Cache)**
   - `moka` (v0.12) 管理的最近最少使用内存缓存。
   - 缓存解压好的原始 YUV / RGB 像素数组 `Arc<Vec<u8>>`。
   - 必须配置**物理内存上限阀值**（如设置上限 4GB），超过即淘汰远离当前游标的帧。
   - 预取策略：永远让解码池提前解出游标前方 2~5 秒的帧数据。
3. **L3 代理文件 (Proxy Cache)**
   - 对于超高规格源素材，后台转换为 ProRes 或小分辨率 intra-frame 编码的暂存文件放在 SSD 上。
