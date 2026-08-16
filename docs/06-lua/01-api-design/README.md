## 剪切流程脚本化与工作流控制 API (Cut Workflow Scripting)

软件支持通过 Lua 脚本精确描述和批量重现用户的全部剪切与轨道操作（无头化/脚本驱动剪辑）：

```lua
-- 1. 项目与轨道创建
local project = timeline:new_project("My Awesome Video")
local t_video = timeline:create_track("Video 1") -- 返回轨道对象或 ID
local t_audio = timeline:create_track("Audio 1")

-- 2. 切片添加
local c1 = timeline:add_clip(t_video, "intro.mp4", "0:00", "0:10") -- (track, asset, start, duration)
local c2 = timeline:add_clip(t_video, "main_interview.mp4", "0:10", "1:30")

-- 3. 切片剪辑操作 (分割、合并、移动、删除)
local c2_left, c2_right = timeline:split(t_video, c2, "0:45") -- 在 45s 处下刀
timeline:delete_clip(t_video, c2_left) -- 将前半段删除至垃圾回收轨道
timeline:merge(t_video, {c1, c2_right}) -- 物理合并选中的切片列表

-- 4. 属性与特效设置
timeline:set_speed(t_video, c2_right, 2.0) -- 设置 2 倍速
timeline:set_lock(t_video, c2_right, true) -- 锁定切片防移位
timeline:add_anchor(t_video, c2_right, "highlight", "0:05", "关键高光时刻") -- 添加锚点

-- 5. 播放头定位与选区
timeline:goto_time("0:30")
```

## 剪切项目保存为 Lua 脚本 (Lua Project Format)
项目不仅支持 `.vcut` 二进制持久化，还支持完整序列化保存为 `.lua` 项目文件。该文件由干净规范的 Lua 领域调用构成，可纳入 Git 进行代码级版本追踪、代码 Review 和协同合并。
