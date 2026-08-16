# 加速减速 (时序映射)

播放速度是一个非常复杂的系统级属性修改，因为它**会改变切片的物理时长**。

## 时间映射算法
源读取时间戳 $t_{src}$ 与时间线时间戳 $t_{tl}$ 的关系为：
$$ t_{src} = (t_{tl} - start_{tl}) 	imes Speed + in_{src} $$

## 慢动作插帧混合预览与时间重映射 (Frame Blending & Slow-Motion Interpolation)
当播放速度低于 1.0x（例如 0.5x、0.25x 慢放）时，原始源视频帧率无法满足时间线连续渲染需求，直接取整会导致画面卡顿或掉帧。
引擎支持在渲染管线中启用**双帧加权插值混合（Frame Blending）**：
- **相邻帧插值因子**：
  $$ \alpha = \text{fract}\left(\frac{t_{src}}{\Delta t_{frame}}\right) $$
- **像素加权混合公式**：
  $$ C_{blend}(x, y) = (1 - \alpha) \cdot C_{frame0}(x, y) + \alpha \cdot C_{frame1}(x, y) $$
- **重映射模式 (`FrameInterpolationMode`)**：
  - `Nearest` (临近截断，性能极佳)
  - `LinearBlend` (双帧线性交叉混合，平滑慢动作)
  - `MotionAdaptive` (运动自适应平滑插帧)
- **指令与右键支持**：
  - `:blend [linear|nearest|adaptive]` 或 `:interp [linear|nearest|adaptive]`
  - 在切片右键菜单中支持 `⏱ 慢动作插帧模式` 快速切换。

## 锁定处理边界
当加速变快时，切片总长变短，后方会留出空隙（Gap）。
当减速变慢时，切片总长拉长。此时引擎**必须检查右侧是否存在拥有锁定属性（Lock）的切片**。
- 如果没有阻碍：将右方的切片集体向右推移（Ripple Insert）。
- 如果碰到锁定的切片：报错，不允许设置该速度，或者强制进行切断（Split/Trim）。

对于音频：速度变化默认改变音调（Pitch），若要保持音高需启动 `rubato` 配合相位声码器预处理。
