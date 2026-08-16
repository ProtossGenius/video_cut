# 模糊搜索 (Fuzzy Matcher)

字符串包含判定不仅要做子串搜索，更要容错式地跳跃匹配。选用 `nucleo` (v0.5.0)。
它是最顶级的多线程模式非阻塞搜索器：

在 Rust 中的基本模式：
每按下一个字母，将更新后的组合词通过 `nucleo.pattern.reparse(0, &query, CaseMatching::Ignore)` 推送。
系统从全量拼音缓存库中并行计算得分，越是头部连续命中的内容得分越高并返回 `match_indices`（也就是那几个命中的字母的高亮游标位）。最后交给 egui 使用不同颜色画出。
