# 动画与关键帧系统 (Animation & Easing Curves)

用于切片随时间进行形变、缩放、旋转或移动的插值系统。

## 缓动曲线类型 (`EasingCurve`)
- **线性 (`Linear`)**：匀速直线运动。
- **渐快 (`EaseIn`)**：慢启动加速。
- **渐慢 (`EaseOut`)**：高速进入并平滑减速。
- **平滑缓入缓出 (`EaseInOut`)**：两端慢、中间快（默认自然过渡）。
- **弹力弹跳 (`BounceOut`)**：模拟重力坠地弹跳衰减物理效果。
- **弹性振荡 (`ElasticOut`)**：模拟橡皮筋拉伸回弹阻尼振荡效果。
- **自定义三次方贝塞尔 (`CubicBezier(x1, y1, x2, y2)`)**：支持 4 控制点三次方贝塞尔曲线牛顿迭代法求解。

## 交互命令与可视化编辑器
- `:easing <preset>`：为当前切片设置默认关键帧缓动曲线（如 `:easing ease_in_out`、`:easing bounce`、`:easing linear`）。
- `:curve [bezier x1 y1 x2 y2]`：设置或打开贝塞尔曲线交互调节视窗。
- 视频切片右键菜单提供 `📈 缓动曲线 (Easing Curve)` 子菜单。

## Lua API 设计
关键帧通过**相对时间**绑定，并且定义顺序严格要求**时间递增**递交（防止状态机错乱）：

```lua
begin_animation()

frame('1:00') -- 切片第 1 分钟处的控制点
actions.add(resize(0.3, '30%', true)) 
actions.add(moveto('30%', 400)) 

frame('+1:05', actions) -- 距离上一个 frame 向后推 1分钟 05秒的位置

end_animation()
```

