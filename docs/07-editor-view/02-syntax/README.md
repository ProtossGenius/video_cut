# 语法高亮 (Tree-Sitter)

摒弃传统的正则匹配高亮方法（太慢、不精确且容易遇到边界换行 bug）。引入目前业界（如 Helix, Zed）的标配增量解析器 `tree-sitter` (v0.24.x) 以及官方的 `tree-sitter-lua`。

1. 初始化 Lua Parser 并将 `rope` 的字符串喂给其生成 AST Tree。
2. 当发生局部编辑时，利用 byte 偏移量和 point (行列号) 调用 `tree.edit(&input_edit)`。
3. 然后再次解析：`parser.parse_with(&mut chunk_reader, Some(&old_tree))` 实现无感瞬间重解析。
4. 遍历生成的 AST 结合 queries 查询表（`highlights.scm`），计算出对应关键字的高亮范围。最后推给 egui LayoutJob 进行 UI 着色渲染。
