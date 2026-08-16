# 摄像机系统

管理在渲染合成之前对特定视窗区域进行提取、裁切与跟拍的能力。

## 聚焦区域 (Focuse Rect)
支持百分比表达式、坐标计算以及反向计算（从右下角的负数扣除）。

```lua
begin_camera()

-- 使用 # 加上输入阶段命名的锚点进行绑定
frame('#start_input') 
actions.add( focuse(rect(0, 'height-10', -100, -500), true) )

frame('end', {
    focuse(rect(0,0, '100%', '100%'), true)
})

end_camera()
```
