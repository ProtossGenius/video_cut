# VideoCut - 任务栈 (Task Stack)

> 说明：本项目采用任务栈管理开发进度。
> 这是一个后进先出（LIFO）的栈结构。最上面的是终极任务，后面跟随的是分解出的一系列子任务。
> 列表中的最后一个任务是当前正在执行的任务。在编程过程中，所有的开发行动都必须围绕当前栈顶的最后一个任务展开。

## 任务执行原则
1. **完成当前任务**：当前栈顶的任务完成后，标记为 `[x]` 或将其移出栈，然后处理上一个任务。
2. **分解子任务**：如果当前任务过于庞大，应当在列表底部将其分解为更小的子任务，先执行最小的子任务。
3. **保持更新**：每完成一个子功能或遇到新的必须解决的 blockers，请及时更新此任务栈。

---

## 任务栈 (当前状态)

1. [ ] **终极任务**：完成 VideoCut 视频剪辑软件的完整开发并发布 1.0 版本
2. [ ] **阶段 1**：搭建核心引擎与基础架构 (Core Engine & Foundation)
   1. [x] 完成项目基础配置 (Cargo.toml 及依赖引入)
   2. [x] 搭建项目目录结构 (按照 CODING_GUIDE.md 建立 src 目录)
3. [x] **完成任务**：初始化项目结构、依赖及编写剩余的设计文档
4. [x] **完成任务**：实现核心领域模型 (Timeline, Track, Clip, ProjectState) 及撤销重做架构
5. [x] **完成任务**：实现快捷键状态机 (Key Trie, Modals) 及指令解析系统
6. [x] **完成任务**：配置 eframe 并在屏幕上绘制四大核心界面的黑盒占位 (GUI 黑盒框架)
7. [x] **完成任务**：实现按键拦截器，将 KeyParser 与 eframe 事件循环接通，能够用按键在黑盒页面间切换
8. [x] **完成任务**：实装文件导入与浏览悬浮窗 (File Browser)（三列布局、读取本地目录、快捷键交互）
9. [x] **完成任务**：实装时间线轨道区域与缩放渲染 (Timeline Panel) 或主界面的其他核心视图
10. [x] **完成任务**：实装 Lua 脚本编辑器接入 (Editor Panel)（自带基于 egui 的代码高亮与自动补全窗，并经过单元测试）
11. [x] **完成任务**：全面升级与美化 UI 界面 (暗黑极客主题、2D 网格导航页、Vimium 字母跳跃、专业剪辑时间线、波形图、垃圾回收轨道与浮动快捷键帮助面板)
12. [x] **完成任务**：实现拼音与模糊搜索引擎 (Pinyin & Fuzzy Matcher)，完成汉字拼音首字母与连续子串匹配排名算法
13. [x] **完成任务**：实现切片特效与属性系统 (Effects & Properties)，包括关键帧动画插值、摄像机聚焦、遮罩、变速碰撞校验与动态注册表
14. [x] **完成任务**：搭建基于 mlua 的 Lua 运行时沙箱引擎 (Lua Runtime)，实现 begin_animation, begin_camera, frame, actions 等脚本接口
15. [x] **完成任务**：实现时间线剪辑命令系统 (Command Pattern)，完成 SplitClipCommand (:split) 与 DeleteClipToTrashCommand 及完整的撤销/重做支持
16. [x] **完成任务**：实现媒体音频波形提取器 (AudioWaveform) 与发声点跳跃算法 (支持 w/b 快捷键定位)
17. [x] **完成任务**：实现视频缩略图 (ThumbnailCache) 与视频解码帧 (VideoFrameCache) 内存缓存池
18. [x] **完成任务**：实现无锁音频环形队列通道 (AudioRingChannel) 用于 CPAL 实时无阻塞音频流驱动
19. [x] **完成任务**：实现二进制二分法区间决策会话 (:biset)，支持 -, =, Backspace 递归分层与确认
20. [x] **完成任务**：实现锚点注册与快捷跳跃系统 (AnchorRegistry)，支持 m/M 局部与全局标记及 W/B 上下一标记跳转
21. [x] **完成任务**：实现时间线轨道层级操作 (Timeline Tracks)，支持 Alt+j/k 轨道上下移、Shift+Enter 插入轨道与 ^/$ 置顶/取消置顶
22. [x] **完成任务**：实现项目持久化、原子覆盖保存 (.vcut)、MessagePack 二进制缓存 (.cache) 及 WAL 崩溃恢复系统 (ProjectStorage & WalLog)
23. [x] **完成任务**：实现时间格式与相对时间解析器 (GotoTimeParser)，支持 :goto 10, 10.12, 1:10, +1:15.3, -30 精准寻迹
24. [x] **完成任务**：实现高级剪辑命令系统，支持重命名轨道 (:name)、切片合并 (:merge) 与选区独立切断合并 (:mergecut) 及其完整撤销重做
25. [x] **完成任务**：实现 Lua 快捷键与分组描述注册绑定系统 (main:bind_key & main:bind_group_desc)，支持多层级动态帮助面板 (HelpSystem)
26. [x] **完成任务**：实现多轨道 GPU 画面合成器 (VideoCompositor) 与纹理对象缓存池 (TexturePool)
27. [x] **完成任务**：在应用主循环中深度集成剪辑快捷键、命令模式、锚点/发声点跳转、轨道层级操作、撤销重做与系统输出浮窗 (:message / :history / :Marks)
28. [x] **完成任务**：实现基于 Ropey B-Tree 架构的文本编辑缓冲区 (TextBuffer)，支持任意字符插入/删除、行列映射与文本级 Undo/Redo
29. [x] **完成任务**：集成 Tree-sitter (0.25) 与 tree-sitter-lua 实现增量 Lua 语法高亮解析器 (LuaSyntaxHighlighter)
30. [x] **完成任务**：实现基于 notify 的 Lua 脚本热重载监听器 (ScriptWatcher)，支持文件修改异步非阻塞通道通知
31. [x] **完成任务**：实现 GPU 与 CPU YUV420P/NV12 到 RGBA 色彩空间转换管线 (YuvConverter)，提供 BT.709/BT.601 转换算法与 WGSL 片元着色器
32. [x] **完成任务**：实现多线程视频解码请求与帧缓存池调度器 (VideoDecoderPool)，支持精确 PTS 寻帧与并发后台解码
33. [x] **完成任务**：实现物理音频主时钟驱动器 (AudioMasterClock) 与多轨道立体声防爆音混音器 (AudioMixer)
34. [x] **完成任务**：全面升级 Miller Columns 三列式媒体导入浏览器 (FileBrowser)，支持媒体智能过滤、Y/y 路径复制、Shift+Enter 目录选择及一键导入时间线
35. [x] **完成任务**：深度实装二分法决策模式 (:biset) UI 视觉化双半区渲染、黑底白字标签 ([-] / [=]) 及键盘二分搜索交互
36. [x] **完成任务**：实装导航页项目垃圾回收备份系统 (/tmp/vcut_trash)、复制/粘贴与全链路撤销恢复机制
37. [x] **完成任务**：将 IDE 脚本编辑器全面升级接入 Tree-sitter Lua AST 增量语法高亮渲染与 Lua 实时脚本执行控制台 (Editor Panel)
38. [x] **完成任务**：主事件循环集成 60FPS 平滑高刷播放引擎与音频主时钟时间同步调度
39. [x] **完成任务**：强化 Lua 引擎沙箱安全 (Sandbox Sanitization)，清理 os/io/debug/package/dofile/loadfile 危险环境并增加沙箱安全防护测试
40. [x] **完成任务**：完整实装切片跟随属性 (FollowProperty)、锚点基准偏移计算与有向环循环依赖检测图 (FollowDependencyGraph)
41. [x] **完成任务**：完整实装绿幕/蓝幕颜色抠像 (ChromaKey) 与矩形羽化/反转透明遮罩算法 (MaskEffect)
42. [x] **完成任务**：实装交互式锚点标记与跳转面板 (`m`/`M`/`'`) 及中文全角标点兼容 (`：`, `’`, `？`, `／`)
43. [x] **完成任务**：实装 Line Visual (`V`) 切片整块多选、`h`/`l` 扩展与批量垃圾箱删除与 `:merge` 合并
44. [x] **完成任务**：升级 IDE 脚本编辑器 Vim 交互、分栏焦点切换 (`Alt+h/l/j`)、F5 运行与分栏状态高亮指示
45. [x] **完成任务**：监视器动效/运镜视口与多轨道合成器 (VideoCompositor) 实时渲染数据桥接联动与 Audio Master VU 电平表指示
46. [x] **完成任务**：在 IDE 脚本编辑器中实装紧凑型切片上下文轨道条 (Context Track Strip) 与 Lua 动画/摄像机/跟随 DSL 全局智能自动补全
47. [x] **完成任务**：深度完善 Command 模式解析器，支持 `:e/:editor <name>` 动态指定目标切片脚本编辑、纯时间码/相对时间直接跳转语法与全套指令自动化单元测试
48. [x] **完成任务**：切片模型实装 `locked`、`z_index` 与 `speed` 属性及 `:lock`、`:unlock`、`:speed` 命令行交互与 UI 锁定/变速角标渲染
49. [x] **完成任务**：实装 `:help`/`:h` 命令行唤起与 `Ctrl+o`/`Ctrl+i` 页面历史导航机制
50. [x] **完成任务**：实装鼠标右键快捷上下文菜单 (Clip & Track Context Menu)
51. [x] **完成任务**：实装基于 Lua 的全流程剪切工作流执行引擎 (`CutWorkflowEngine`) 与整套流程自动化测试
52. [x] **完成任务**：实装剪切项目序列化保存与读取为 Lua 项目工程文件 (`project.lua`)
53. [x] **完成任务**：实装后台空闲分片预渲染调度器 (`ChunkPreRenderer`) 与无重编码极速拼接导出引擎 (`SmartConcatExporter`)
54. [x] **完成任务**：实装 `:help`/`:h` 独立命令参考帮助弹窗，按行展示命令名与详细描述，支持按斜线 `/` 开启搜索筛选与键盘快速选择执行
55. [x] **完成任务**：实装 `:Marks`/`:marks` 独立交互式锚点管理弹窗，支持全部全局与局部锚点展示、内联描述编辑、实时搜索过滤与极速跳转定位






