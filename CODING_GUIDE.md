# VideoCut - AI 编程规范指南 (Coding Guide)

这是一份专门为 AI 辅助编程（包括 Copilot 和 Antigravity/AGY 等 agent）编写的规范指南。请在为本项目编写代码前，严格遵循本指南中的架构原则和代码规范。

## 1. 项目概述

VideoCut 是一个基于 Rust 开发的，支持无鼠标操作（纯键盘/vim-like 快捷键）的桌面非线性视频剪辑软件。
- **目标平台**：仅支持 Unix-like 系统（macOS / Linux）。
- **核心特色**：全键盘操作流，通过 Lua 脚本添加和自定义特效/属性。

## 2. 技术栈与核心依赖

在编写代码时，请**严格使用以下指定的 crate 及其版本**：

### UI 与渲染层
- **GUI 框架**: `egui` (v0.36.x) + `eframe`
- **GPU 渲染**: `wgpu` (v30.0)
- **文本编辑器**: `ropey` (v1.6.1) + `tree-sitter` (v0.24.x) + `tree-sitter-lua` (v0.5.0)

### 媒体与音视频处理
- **视频解码/编码**: `ffmpeg-next` (v9.0.0)
- **音频 I/O**: `cpal` (v0.18.1)
- **音频解码**: `symphonia` (v0.6.0)
- **音频重采样**: `rubato` (v5.0.0)
- **音频通道**: `rtrb` (v0.3.1) - 用于音频实时线程的 lock-free ring buffer
- **图像缩放**: `fast_image_resize` (v6.1.0)
- **图像读写**: `image` (v0.25.10)

### 数据结构与基础库
- **脚本引擎**: `mlua` (v0.12.0)
- **中文转拼音**: `pinyin` (v0.11.0)
- **模糊搜索**: `nucleo` (v0.5.0)
- **文件系统**: `notify` (v8.0.0) + `walkdir` (v2.5.0) + `dirs` (v6.0.0)
- **序列化**: `serde` (v1.0) + `serde_json` (v1.0) + `rmp-serde` (v1.3.1)
- **缓存**: `moka` (v0.12)
- **并发通信**: `crossbeam-channel`
- **时间索引**: `intervaltree`
- **错误处理**: `thiserror` (库内部) + `anyhow` (应用层)

## 3. 架构原则

1. **尽可能使用第三方开源库**：不要重复造轮子，优先使用上面技术栈中指定的成熟 crate。
2. **高内聚低耦合**：做好抽象，使各个子功能相互独立。
3. **注册式扩展**：特效（Effect）、属性（Property）、命令（Command）等必须通过注册的方式（如 `inventory` crate 配合 trait）动态添加，避免硬编码。
4. **音频为主时钟（Audio Master Clock）**：视频帧和时间线的播放进度必须向音频时钟对齐，而不是反过来。
5. **非破坏性编辑（Non-Destructive Editing）**：任何操作绝对不能修改源媒体文件。所有的编辑都应记录为时间线上的属性和切片数据。
6. **Command Pattern 实现撤销/重做**：所有的修改状态操作必须实现为 Command，以支持 Undo/Redo。

## 4. 代码规范

1. **版本与规范**：使用 Rust 2021 Edition。严格遵循 Rust 官方命名规范：
   - 变量/函数/模块：`snake_case`
   - 类型/结构体/枚举/Trait：`CamelCase`
2. **模块划分**：每个功能模块应该是一个独立的目录，包含 `mod.rs` 作为入口。
3. **注释**：公共 API、复杂逻辑、架构关键点必须包含详尽的文档注释（`///`）。
4. **错误处理**：
   - 坚决**禁止使用 `unwrap()` 或 `expect()`**（测试代码除外）。
   - 将错误向上传递，或者在合适的地方处理。
   - 使用 `thiserror` 定义具体的错误类型，使用 `anyhow` 在最顶层捕捉。
5. **编码**：所有源文件、文本处理必须使用 UTF-8 编码。

## 5. 测试规范

1. **TDD / 必须有测试**：每个新增功能必须编写对应的单元测试。
2. **提交流程**：在完成功能后，必须确保 `cargo test` 全部通过，才能提交代码。
3. **Bug 修复流程**：
   - 第一步：编写一个能复现该 bug 的**最小单元测试**（此时测试应该是失败的）。
   - 第二步：修复 bug。
   - 第三步：运行所有测试，确保该 bug 测试通过，且未破坏其他旧功能。
4. **测试位置**：测试代码应紧随业务代码，放在同文件的 `#[cfg(test)] mod tests { ... }` 块中。

## 6. Git 与提交流程

1. **单一职责**：每个提交（Commit）只包含一个逻辑功能。
2. **Commit Message 格式**：`<type>(<scope>): <description>`
   - `type` 取值：`feat` (新功能), `fix` (修复), `refactor` (重构), `test` (测试), `docs` (文档), `chore` (日常事务/构建相关)。
   - `scope`：影响的模块（如 `timeline`, `gui`, `audio`）。
3. **格式化与代码检查**：提交前必须运行 `cargo fmt` 和 `cargo clippy` 且没有警告。

## 7. 文档维护

1. **进度更新**：每完成一个子功能，必须在对应的 Markdown 文档中标注已完成（如修改 `docs/stack.md`）。
2. **任务栈**：严格按照 `docs/stack.md` 规划的顺序进行开发，处理完栈顶任务才能进行下一个。
3. **术语**：命名和文档中涉及专业词汇时，参考 `docs/glossary.md`。

## 8. 项目目录结构推荐

```
video_cut/
├── Cargo.toml
├── PLAN.md                  # 原始需求规划
├── CODING_GUIDE.md          # AI 开发指南（本文档）
├── docs/                    # 项目技术文档
│   ├── README.md            # 总架构设计
│   ├── stack.md             # 任务栈
│   ├── glossary.md          # 术语表
│   ├── 01-gui/              # GUI框架与界面系统
│   ├── 02-timeline/         # 时间线系统
│   ├── 03-media/            # 媒体处理(音视频)
│   ├── 04-keybinding/       # 快捷键与模态
│   ├── 05-effects/          # 特效与属性
│   ├── 06-lua/              # Lua 脚本系统
│   ├── 07-editor-view/      # 内置代码编辑器
│   ├── 08-search/           # 拼音与模糊搜索
│   ├── 09-project/          # 项目管理与存储
│   └── 10-rendering/        # GPU渲染管线
└── src/
    ├── main.rs
    ├── app/                 # 应用层整合
    ├── gui/                 # 界面实现
    ├── timeline/            # 轨道、切片、选择模型
    ├── media/               # 解码、音视频管线
    ├── keybinding/          # 快捷键解析与执行
    ├── effects/             # 属性注册与动画
    ├── lua_engine/          # mlua 桥接与沙箱
    ├── editor/              # 文本缓冲与语法高亮
    ├── search/              # pinyin + nucleo 集成
    ├── project/             # 序列化、Undo/Redo
    └── rendering/           # wgpu 自定义管线
```

## 9. 模块间通信规范

1. **通信机制**：不同线程/大模块间通信应使用 `crossbeam-channel`（例如解码器池发帧给 UI）。
2. **公共事件**：跨模块通信的事件（Event）结构体应定义在一个单独的共享库或公共模块中，避免循环依赖。
3. **解耦**：UI 层只负责发送命令和渲染状态，不包含核心业务逻辑。

## 10. 性能规范

1. **UI 线程 (Thread 1)**：绝对禁止任何阻塞型 I/O 操作（包括长时间的同步解码、大文件读取），以保证 `egui` 能稳定跑在 60 FPS 以上。
2. **音频线程 (Thread 2)**：由于 `cpal` 回调是实时性要求极高的，在这个线程中**绝对禁止分配内存、阻塞锁**，只能使用 `rtrb` 这类 lock-free 队列获取数据。
3. **解码池 (Thread 4..N)**：视频解码属于 CPU 密集型任务，必须放在后台线程池（或专用线程）中执行。
4. **GPU 内存**：视频帧作为纹理（Texture）上传至 GPU 时，必须实现纹理池（Texture Pool），避免每帧都重新分配和销毁 `wgpu::Texture`。
