use pinyin::ToPinyin;

/// 匹配结果，包含匹配得分和命中的字符下标区间（用于 UI 高亮）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub score: u32,
    pub matched_indices: Vec<usize>, // 原始字符串中命中的字符下标 (char index)
}

pub struct PinyinFuzzyMatcher;

impl PinyinFuzzyMatcher {
    /// 将文本转为拼音字符串
    pub fn to_pinyin_string(text: &str) -> String {
        let mut result = String::new();
        for c in text.chars() {
            if let Some(p) = c.to_pinyin() {
                result.push_str(p.plain());
            } else {
                result.push(c.to_ascii_lowercase());
            }
        }
        result
    }

    /// 针对单个候选字符串进行拼音 + 模糊匹配
    /// 返回匹配得分与高亮索引。如果未匹配则返回 None
    pub fn match_query(target: &str, query: &str) -> Option<MatchResult> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Some(MatchResult {
                score: 100,
                matched_indices: Vec::new(),
            });
        }

        let target_chars: Vec<char> = target.chars().collect();
        if target_chars.is_empty() {
            return None;
        }

        // 1. 直接子串匹配（最优先）
        let target_lower = target.to_lowercase();
        if let Some(byte_pos) = target_lower.find(&query) {
            let char_start = target[..byte_pos].chars().count();
            let match_char_count = query.chars().count();
            let matched_indices: Vec<usize> = (char_start..char_start + match_char_count).collect();
            return Some(MatchResult {
                score: 1000 + (100 / (char_start as u32 + 1)),
                matched_indices,
            });
        }

        // 2. 拼音拼写匹配
        // 为每个字符提取拼音信息 (首字母, 全拼)
        let mut char_pinyins: Vec<Vec<String>> = Vec::with_capacity(target_chars.len());
        for &c in &target_chars {
            let mut pinyins = Vec::new();
            if let Some(p) = c.to_pinyin() {
                pinyins.push(p.plain().to_string());
                let first_letter = p.plain().chars().next().unwrap_or(' ').to_string();
                if !pinyins.contains(&first_letter) {
                    pinyins.push(first_letter);
                }
            } else {
                pinyins.push(c.to_lowercase().to_string());
            }
            char_pinyins.push(pinyins);
        }

        // 尝试首字母缩写序列匹配 (例如 "hl" 匹配 "hello", "hlbt" 匹配 "哈利波特")
        if let Some(indices) = Self::match_initials(&char_pinyins, &query) {
            return Some(MatchResult {
                score: 500 + (100 / (indices.first().copied().unwrap_or(0) as u32 + 1)),
                matched_indices: indices,
            });
        }

        // 尝试拼音连续子序列匹配 (例如 "bot" 匹配 "哈利波特" 中的 "波特" (bo te))
        if let Some(indices) = Self::match_pinyin_substring(&char_pinyins, &query) {
            return Some(MatchResult {
                score: 300 + (100 / (indices.first().copied().unwrap_or(0) as u32 + 1)),
                matched_indices: indices,
            });
        }

        // 3. 散列字符子序列匹配 (Vim-like fuzzy)
        if let Some(indices) = Self::match_fuzzy_subsequence(&target_chars, &query) {
            return Some(MatchResult {
                score: 100 + (50 / (indices.last().copied().unwrap_or(0) as u32 + 1)),
                matched_indices: indices,
            });
        }

        None
    }

    /// 匹配拼音首字母序列
    fn match_initials(char_pinyins: &[Vec<String>], query: &str) -> Option<Vec<usize>> {
        let q_chars: Vec<char> = query.chars().collect();
        let mut q_idx = 0;
        let mut matched = Vec::new();

        for (i, p_list) in char_pinyins.iter().enumerate() {
            let q_char = q_chars[q_idx];
            let has_match = p_list.iter().any(|p| p.starts_with(q_char));
            if has_match {
                matched.push(i);
                q_idx += 1;
                if q_idx == q_chars.len() {
                    return Some(matched);
                }
            }
        }
        None
    }

    /// 匹配汉字拼音连续拼接的子串 (如 "bot" -> "bo"+"te")
    #[allow(clippy::needless_range_loop)]
    fn match_pinyin_substring(char_pinyins: &[Vec<String>], query: &str) -> Option<Vec<usize>> {
        let n = char_pinyins.len();
        for start in 0..n {
            let mut acc = String::new();
            let mut indices = Vec::new();
            for i in start..n {
                let p = char_pinyins[i].first()?;
                acc.push_str(p);
                indices.push(i);
                if acc.contains(query) || query.starts_with(&acc) || acc.starts_with(query) {
                    if acc.len() >= query.len() && acc.contains(query) {
                        return Some(indices);
                    }
                } else if !query.starts_with(&acc) {
                    break;
                }
            }
        }
        None
    }

    /// 简单模糊子序列匹配
    fn match_fuzzy_subsequence(target_chars: &[char], query: &str) -> Option<Vec<usize>> {
        let q_chars: Vec<char> = query.chars().collect();
        let mut q_idx = 0;
        let mut matched = Vec::new();

        for (i, &tc) in target_chars.iter().enumerate() {
            if tc.eq_ignore_ascii_case(&q_chars[q_idx]) {
                matched.push(i);
                q_idx += 1;
                if q_idx == q_chars.len() {
                    return Some(matched);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_substring_match() {
        let res = PinyinFuzzyMatcher::match_query("hello world", "world");
        assert!(res.is_some());
        let m = res.unwrap();
        assert_eq!(m.matched_indices, vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_pinyin_initials_match() {
        let res = PinyinFuzzyMatcher::match_query("哈利波特", "hl");
        assert!(res.is_some());
        let m = res.unwrap();
        assert_eq!(m.matched_indices, vec![0, 1]); // 哈 (h) 利 (l)
    }

    #[test]
    fn test_pinyin_substring_bot_match() {
        let res = PinyinFuzzyMatcher::match_query("哈利波特", "bot");
        assert!(res.is_some());
        let m = res.unwrap();
        // 应该匹配并高亮“波特” (下标 2, 3)
        assert!(m.matched_indices.contains(&2));
    }

    #[test]
    fn test_english_fuzzy_subsequence() {
        let res = PinyinFuzzyMatcher::match_query("summer_vlog_2026.mp4", "svlog");
        assert!(res.is_some());
    }
}
