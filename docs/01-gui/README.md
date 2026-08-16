# GUI框架与界面系统总述

本项目使用 `egui`/`eframe` (v0.36.x) 作为 GUI 框架，`wgpu` (v30.0) 作为渲染后端。
界面系统被划分为四个核心页面：
- [导航页面](./01-navigation/README.md)：项目管理与选择
- [主界面](./02-main-interface/README.md)：多轨道剪辑和预览
- [文件导入](./03-file-browser/README.md)：资源管理和选择
- [内置编辑器](./04-editor/README.md)：基于 Lua 的代码特效编写
