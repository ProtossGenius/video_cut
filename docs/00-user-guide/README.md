# VideoCut 使用指南

> 面向日常剪辑使用者的快速上手文档。若你想知道“怎么开工、怎么粗剪、怎么保存、怎么导出”，从这里开始。

## 1. 启动方式

```bash
video_cut
video_cut project.vcut
video_cut project.lua
video_cut run workflow.lua
```

- `video_cut`：启动并进入项目导航页
- `video_cut project.vcut`：直接打开保存过的工程
- `video_cut project.lua`：直接加载 Lua 工程脚本
- `video_cut run workflow.lua`：无头执行工作流脚本，并自动产出 `.vcut`

## 2. 界面总览

VideoCut 主要有 3 个工作页面：

1. **导航页**：选择、新建、搜索项目
2. **主界面**：导入素材、预览、剪切、调音、导出
3. **编辑器**：给当前切片编写 Lua 动画 / 摄像机 / 跟随脚本

主界面又分为四块：

1. **左上**：素材栏
2. **右上**：监视器预览
3. **下方**：多轨道时间线
4. **最底部**：状态栏 / 命令行

## 3. 第一次使用的最短路径

1. 在导航页按 `n` 新建项目，或按 `Enter` 打开已有项目。
2. 进入主界面后按 `i` 或 `+` 打开媒体导入浏览器。
3. 用 `j/k` 选中文件，`Enter` 导入到当前轨道。
4. 用 `Space` 播放，`h/l` 微调播放头，`j/k` 切换轨道。
5. 在要下刀的位置按 `s` 分割切片。
6. 需要选区时按 `v`，需要整块多选时按 `V`。
7. 用 `:w project.vcut` 保存工程。
8. 用 `:export output.mp4 h264` 打开导出面板并开始导出。

## 4. 常用键位

| 场景 | 按键 | 作用 |
|------|------|------|
| 导航页 | `f` | Vimium 风格字母快速打开项目 |
| 导航页 | `/` | 拼音 / 模糊搜索项目 |
| 全局 | `?` | 打开 / 关闭快捷键帮助面板 |
| 主界面 | `Space` | 播放 / 暂停 |
| 主界面 | `h / l` | 左右微调播放头 |
| 主界面 | `j / k` | 切换上下轨道 |
| 主界面 | `J / K` | 跳到下一个 / 上一个切片边缘 |
| 主界面 | `w / b` | 跳到下一个 / 上一个发声点 |
| 主界面 | `W / B` | 跳到下一个 / 上一个锚点 |
| 主界面 | `s` | 在播放头处分割切片 |
| 主界面 | `x` | 波纹删除并闭合间隙 |
| 主界面 | `u / Ctrl+r` | 撤销 / 重做 |
| 主界面 | `v / V` | 时间选区 / 切片多选 |
| 主界面 | `m / M` | 设置局部 / 全局锚点 |
| 主界面 | `'` | 打开锚点跳转面板 |
| 主界面 | `:` | 进入命令模式 |
| 主界面 | `Shift+Enter` | 在当前轨道下方插入新轨道 |
| 主界面 | `Alt+j / Alt+k` | 轨道下移 / 上移 |
| 主界面 | `Ctrl+o / Ctrl+i` | 页面历史后退 / 前进 |
| 编辑器 | `F5` | 运行当前 Lua 脚本 |
| 编辑器 | `Alt+h/l/j` | 切换编辑器 / 控制台 / 检查面板焦点 |

## 5. 常用命令

### 5.1 工程与页面

| 命令 | 作用 |
|------|------|
| `:w [path]` | 保存工程到 `.vcut` 或 `.lua` |
| `:q` | 返回导航页 |
| `:history` | 查看历史命令 |
| `:message` | 查看系统消息面板 |
| `:help` | 打开命令参考面板 |

### 5.2 时间线与轨道

| 命令 | 作用 |
|------|------|
| `:split` | 按播放头分割当前切片 |
| `:merge` | 合并选中的切片 |
| `:mergecut` | 切断选区左右边缘并合并中段 |
| `:name <new name>` | 重命名当前轨道 |
| `:pin` / `:unpin` | 置顶 / 取消置顶当前轨道 |
| `:track_up` / `:track_down` | 当前轨道上移 / 下移 |
| `:new_track_above` / `:new_track_below` | 在当前轨道上下插入新轨道 |
| `:delete_track` | 删除当前轨道（至少保留一条） |
| `:import_media` | 打开导入浏览器 |

### 5.3 视频与文本

| 命令 | 作用 |
|------|------|
| `:lock` / `:unlock` | 锁定 / 解锁切片 |
| `:speed 0.5` / `:speed 2.0` | 设置切片变速 |
| `:rotate 90` | 旋转切片 |
| `:flip h` / `:flip v` | 水平 / 垂直翻转 |
| `:scale_clip 1.5` | 缩放当前切片 |
| `:brightness 0.2` | 调亮度 |
| `:contrast 1.2` | 调对比度 |
| `:saturation 1.4` | 调饱和度 |
| `:temp -0.2` | 调色温 |
| `:lut teal_orange` | 应用 LUT 预设 |
| `:text 欢迎观看` | 添加字幕 |
| `:fontsize 32` | 设置字幕字号 |
| `:textcolor yellow` | 设置字幕颜色 |
| `:bgbox on` | 打开字幕底框 |

### 5.4 音频

| 命令 | 作用 |
|------|------|
| `:vol 1.2` | 主音量 120% |
| `:mute` / `:unmute` | 主静音 / 取消静音 |
| `:track_vol 0.8` | 当前轨道音量 |
| `:pan -0.5` | 当前轨道声相偏左 |
| `:fadein 0.5` / `:fadeout 1.0` | 音频淡入 / 淡出 |
| `:eq preset podcast` | 应用播客 EQ 预设 |
| `:eq low 90 -3.5 0.8` | 设置低频 EQ |
| `:eq mid 2200 2.5 1.2` | 设置中频 EQ |
| `:eq high 9500 1.8 0.7` | 设置高频 EQ |
| `:loudnorm stream` | 按 -14 LUFS 计算响度增益 |
| `:loudnorm broadcast` | 按 -23 LUFS 计算响度增益 |
| `:loudnorm off` | 关闭响度标准化 |

### 5.5 高性能粗剪

| 命令 | 作用 |
|------|------|
| `:proxy on` / `:proxy off` | 打开 / 关闭代理媒体预览 |
| `:gen_proxy 720p` | 为当前切片或多选切片生成 720p 代理 |
| `:gen_proxy 360p` | 生成 360p 极速代理 |
| `:proxy_status` | 查看代理媒体状态 |
| `:monitor_zoom fit` | 监视器视口适应窗口 |
| `:monitor_zoom 150` | 监视器视口放大到 150% |

### 5.6 键位、宏与自动化

| 命令 | 作用 |
|------|------|
| `:keymap vim` / `:keymap premiere` / `:keymap fcp` | 切换键位预设 |
| `:keymap_conflicts` | 查看 Trie 键位冲突 |
| `:export_keymap profile.json` | 导出当前键位 |
| `:import_keymap profile.lua` | 导入键位配置 |
| `:macros` | 查看已录制宏 |
| `:save_macros path.json` | 保存键盘宏 |
| `:load_macros path.json` | 载入键盘宏 |
| `:export_macro a` | 导出寄存器 `a` 的宏为 Lua |

### 5.7 导出与打包

| 命令 | 作用 |
|------|------|
| `:export output.mp4 h264` | 打开导出面板并设置输出 |
| `:export_queue` | 查看导出队列 |
| `:cancel_export` | 取消当前与排队导出 |
| `:export_lua project.lua` | 导出 Lua 工程脚本 |
| `:pack_project bundle_dir` | 打包项目与资源 |
| `:unpack_project bundle_dir` | 解包并打开项目 |

## 6. 典型工作流

### 6.1 快速粗剪

1. 导入素材：`i`
2. 打开代理：`:proxy on`
3. 为 4K/8K 素材生成代理：`:gen_proxy 720p`
4. 播放与走位：`Space`、`h/l`、`w/b`
5. 下刀：`s`
6. 波纹删减：`x`
7. 多选并合并：`V` → `h/l` → `:merge`

### 6.2 对白修整

1. 用 `w/b` 在发声点间跳跃
2. 用 `:fadein` / `:fadeout` 清理切口
3. 用 `:eq preset podcast` 或 `:eq preset vocal`
4. 用 `:loudnorm stream` 对齐到流媒体响度目标

### 6.3 特效脚本

1. 将播放头放到目标切片
2. 输入 `:editor`
3. 在编辑器中写 `begin_animation()` / `begin_camera()`
4. 按 `F5` 运行
5. 对 Follow 目标可直接输入 `follow('Track 1.Clip_A')`，补全会根据当前工程动态给出候选

### 6.4 交付与归档

1. `:w project.vcut`
2. `:export final.mp4 hevc`
3. 若需打包给别人：`:pack_project deliverable_bundle`
4. 若需保存可审阅脚本版本：`:export_lua project.lua`

## 7. 鼠标工作流

虽然 VideoCut 是键盘优先的，但鼠标右键菜单也已经补齐：

- **切片右键**：分割、锁定、变速、代理、字幕、颜色、转场等
- **轨道头右键**：重命名、置顶、上下移动、插入轨道、音量 / 声相 / EQ / 响度
- **轨道空白区右键**：导入媒体、插入轨道、删除轨道、置顶轨道
- **监视器右键**：切换代理、调整视口缩放、打开导入浏览器

## 8. 文件与工程格式

- `.vcut`：主工程文件
- `.lua`：可脚本化 / 可版本控制的工程描述
- `.vcutpkg`（目录形态）：项目归档包，包含工程、脚本与资源副本

## 9. 推荐记忆顺序

如果你刚上手，先记住下面这组就够：

1. `i` 导入
2. `Space` 播放
3. `h/l` 微调
4. `s` 分割
5. `x` 波纹删除
6. `v/V` 选择
7. `:` 命令
8. `:w` 保存
9. `:export` 导出
10. `?` 查看帮助
