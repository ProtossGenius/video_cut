# 键位预设、Trie 冲突检测与 JSON/Lua 导入导出

VideoCut 现在提供一套独立的键位映射管理器，支持 **Vim / Premiere Pro / Final Cut Pro** 三种预设，以及自定义 JSON / Lua 键位字典的导入导出。

## 内置预设
- **Vim**：`h/j/k/l` 导航、`s` 分割、`v/V` 选择
- **Premiere Pro**：`J/K/L` 预览、`C` Razor 分割、`Ctrl+Z` 撤销
- **Final Cut Pro**：`J/K/L` 预览、`B` Blade 分割、`Cmd+Z` 撤销

状态栏 `⌨` 按钮可在三种内置预设间循环切换。

## Trie 冲突检测
导入或切换预设后，系统会用按键 Trie 做两类冲突扫描：
1. **重复绑定**：同模式下同一键位被多次绑定
2. **前缀冲突**：已有完整键位被更长序列抢占，或新键位本身成为其他绑定的前缀

可通过 `:keymap_conflicts` 将冲突列表写入消息面板。

## JSON / Lua
- `:export_keymap profile.json`
- `:export_keymap profile.lua`
- `:import_keymap profile.json`
- `:import_keymap profile.lua`

Lua 采用与现有脚本体系一致的形式：

```lua
main:bind_key("normal", "Space", "play_pause", "播放 / 暂停")
main:bind_key("normal", "C", ":split<CR>", "Razor / 切刀分割")
main:bind_group_desc("ad", "添加元素组")
```

## 命令行入口
- `:keymap vim`
- `:keymap premiere`
- `:keymap fcp`
- `:keymap cycle`
- `:keymap status`
