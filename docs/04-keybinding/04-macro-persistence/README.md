# 键盘宏持久化与 Lua 脚本导出 (Macro Persistence & Lua Exporter)

键盘宏允许用户将高频、重复的剪辑按键与命令行序列录制到命名寄存器（`a-z`）中，支持持久化保存以及导出为可执行的 Lua 自动化剪辑脚本。

## 宏录制与执行流
- **开始录制**：输入 `q<register>`（例如 `qa` 录制到寄存器 `a`）。
- **实时指示**：状态栏左下角呈现高亮闪烁指示器 `🔴 REC [a]`。
- **结束录制**：再次输入 `q` 完成录制。
- **回放宏**：输入 `@<register>`（例如 `@a`）或 `@@`（重复上一宏），支持数字前缀（例如 `5@a` 连播 5 次）。

## 序列化与文件持久化
- **JSON 持久化**：`save_to_json(path)` 与 `load_from_json(path)`，自动在项目保存或关闭时存储宏字典至 `project_dir/.macros.json`。
- **Lua 脚本导出**：`export_to_lua(register)` 将录制的按键序列转换为标准 Lua 自动化工作流函数：
  ```lua
  function replay_macro_a()
      app:execute_command(":snap on")
      app:split_at_playhead()
      app:goto_offset(1.0)
  end
  ```

## 交互命令
- `:macros` / `:list_macros`：查看当前已录制的寄存器与按键数量。
- `:save_macros <path>`：保存宏到指定 JSON 文件。
- `:load_macros <path>`：从 JSON 文件载入宏。
