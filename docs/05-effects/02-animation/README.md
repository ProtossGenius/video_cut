# 动画与关键帧系统

用于切片随时间进行形变、缩放、旋转或移动的插值系统。

## Lua API 设计
关键帧只能通过**相对时间**绑定，并且定义顺序严格要求**时间递增**递交（防止状态机错乱），否则会在 Lua 运行时抛错。

```lua
begin_animation()

frame('1:00') -- 切片第 1 分钟处的控制点
-- 增加动作，第二个参数 true 代表与上一个控制点之间进行平滑插值（渐变），false 则是硬切。
actions.add(resize(0.3, '30%', true)) 
actions.add(moveto('30%', 400)) 

frame('+1:05', actions) -- 距离上一个 frame 向后推 1分钟 05秒的位置

end_animation()
```
