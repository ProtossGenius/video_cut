# 项目主界面

这是用户进行剪辑的核心操作台。

## 布局概览
- **左上部分**：素材栏（Media Pool），显示当前项目导入的文件列表。
- **右上部分**：视频预览视口（Viewport）。
- **底部区域**：多轨道时间线（Timeline），默认两条主轨道。
- **最底部固定**：垃圾轨道（Trash Track）。
- **最下方单行**：状态栏（Status Bar）。

```mermaid
graph TD
    subgraph 屏幕布局
        TopHalf[上半部分]
        BottomHalf[下半部分]
    end
    TopHalf --> Left(素材栏 Media Pool)
    TopHalf --> Right(视频预览 Viewport)
    BottomHalf --> Tracks(主轨道区域 Timeline)
    Tracks --> TrashTrack(垃圾轨道 - 永远在底层)
    TrashTrack --> StatusBar(状态栏 - 最下方固定单行)
```

## 技术实现要点
- 面板的分割建议使用 `egui_tiles` 或原生的 `egui::SidePanel` + `egui::TopBottomPanel` + `egui::CentralPanel` 组合。
- 视频预览区域并非依靠 CPU 将每一帧传给 egui，而是直接渲染在一个 `wgpu::Texture` 上，egui 中只放置一个分配好 TextureId 的 `Image` 控件。
- 轨道区是自定义绘制区域，利用 `egui::Painter` 进行实时画图，确保时间线能以 60 FPS 平滑滚动缩放。

## 鼠标与右键上下文菜单 (Context Menus)
为了兼顾键鼠操作流，界面中的核心对象均支持完整的右键上下文菜单：
1. **时间线切片右键菜单**：
   - ✂ 在指针处分割 (`s` / `split`)
   - 🗑 删除切片至回收站 (`x` / `Delete`)
   - 🔒 锁定 / 🔓 解锁切片 (`:lock` / `:unlock`)
   - ⏱ 播放速度乘数调节 (`:speed`)
   - 🏷 添加/编辑局部时间锚点 (`m`)
   - 📝 打开专属 Lua 脚本 IDE (`:e`)
2. **轨道头部右键菜单**：
   - ✏ 重命名轨道 (`:name`)
   - 📌 置顶/固定轨道 (`P`)
   - ⬇ 向下移动 (`Shift+J`) / ⬆ 向上移动 (`Shift+K`)
   - ➕ 在下方新建轨道 (`O`) / ➕ 在上方新建轨道 (`Shift+O`)
   - ❌ 删除轨道

