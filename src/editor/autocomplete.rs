#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub detail: String,
}

pub struct Autocomplete {
    static_keywords: Vec<CompletionItem>,
    dynamic_keywords: Vec<CompletionItem>,
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

impl Autocomplete {
    pub fn new() -> Self {
        let mut static_keywords = vec![
            // animation DSL globals
            CompletionItem {
                label: "begin_animation".into(),
                detail: "() -- 开始关键帧动画定义块".into(),
            },
            CompletionItem {
                label: "end_animation".into(),
                detail: "() -- 结束关键帧动画定义块".into(),
            },
            // camera DSL globals
            CompletionItem {
                label: "begin_camera".into(),
                detail: "() -- 开始摄像机运镜视口定义块".into(),
            },
            CompletionItem {
                label: "end_camera".into(),
                detail: "() -- 结束摄像机运镜视口定义块".into(),
            },
            CompletionItem {
                label: "focuse".into(),
                detail: "(rect, interpolate) -- 镜头聚焦区域".into(),
            },
            CompletionItem {
                label: "frame".into(),
                detail: "(time, actions) -- 设置关键帧时间点".into(),
            },
            CompletionItem {
                label: "moveto".into(),
                detail: "(x, y) -- 平移位移变换".into(),
            },
            CompletionItem {
                label: "rect".into(),
                detail: "(x, y, width, height) -- 几何区域".into(),
            },
            CompletionItem {
                label: "resize".into(),
                detail: "(scale, aspect, interpolate) -- 缩放变换".into(),
            },
            CompletionItem {
                label: "follow".into(),
                detail: "(target_clip, offset) -- 设置切片跟随锚定".into(),
            },
            // main object
            CompletionItem {
                label: "main:bind_key".into(),
                detail: "(mode, key, cmd, desc)".into(),
            },
            CompletionItem {
                label: "main:bind_group_desc".into(),
                detail: "(prefix, desc)".into(),
            },
            // animation object
            CompletionItem {
                label: "animation:begin_animation".into(),
                detail: "()".into(),
            },
            CompletionItem {
                label: "animation:end_animation".into(),
                detail: "()".into(),
            },
            CompletionItem {
                label: "animation:frame".into(),
                detail: "(time)".into(),
            },
            // actions
            CompletionItem {
                label: "actions:resize".into(),
                detail: "(scale, interpolate)".into(),
            },
            CompletionItem {
                label: "actions:moveto".into(),
                detail: "(x, y)".into(),
            },
            CompletionItem {
                label: "actions:rotate".into(),
                detail: "(angle)".into(),
            },
            // camera object
            CompletionItem {
                label: "camera:begin_camera".into(),
                detail: "()".into(),
            },
            CompletionItem {
                label: "camera:end_camera".into(),
                detail: "()".into(),
            },
            CompletionItem {
                label: "camera:focuse".into(),
                detail: "(rect, interpolate)".into(),
            },
            CompletionItem {
                label: "camera:frame".into(),
                detail: "(time)".into(),
            },
        ];

        static_keywords.sort_by(|a, b| a.label.cmp(&b.label));

        Self {
            static_keywords,
            dynamic_keywords: Vec::new(),
        }
    }

    pub fn set_follow_targets(&mut self, targets: Vec<(String, String)>) {
        let mut dynamic_keywords = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for (identifier, detail) in targets {
            for label in [
                identifier.clone(),
                format!("'{}'", identifier),
                format!("\"{}\"", identifier),
            ] {
                if seen.insert(label.clone()) {
                    dynamic_keywords.push(CompletionItem {
                        label,
                        detail: format!("follow target -- {}", detail),
                    });
                }
            }
        }

        dynamic_keywords.sort_by(|a, b| a.label.cmp(&b.label));
        self.dynamic_keywords = dynamic_keywords;
    }

    pub fn extract_prefix(line_up_to_cursor: &str) -> String {
        if let Some(quoted_prefix) = extract_unclosed_quote_prefix(line_up_to_cursor) {
            return quoted_prefix;
        }

        line_up_to_cursor
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':' || *c == '.')
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    }

    /// 根据光标之前的这一行文本，提取出当前的输入单词，并返回匹配的补全列表
    pub fn complete(&self, line_up_to_cursor: &str) -> Vec<CompletionItem> {
        let prefix = Self::extract_prefix(line_up_to_cursor);
        if prefix.is_empty() {
            return vec![];
        }

        let mut matches: Vec<CompletionItem> = self
            .static_keywords
            .iter()
            .chain(self.dynamic_keywords.iter())
            .filter(|item| item.label.starts_with(&prefix))
            .cloned()
            .collect();
        matches.sort_by(|a, b| a.label.cmp(&b.label));
        matches
    }
}

fn extract_unclosed_quote_prefix(line_up_to_cursor: &str) -> Option<String> {
    let mut in_single = None;
    let mut in_double = None;
    let mut prev = '\0';

    for (idx, ch) in line_up_to_cursor.char_indices() {
        match ch {
            '\'' if prev != '\\' && in_double.is_none() => {
                if in_single.is_some() {
                    in_single = None;
                } else {
                    in_single = Some(idx);
                }
            }
            '"' if prev != '\\' && in_single.is_none() => {
                if in_double.is_some() {
                    in_double = None;
                } else {
                    in_double = Some(idx);
                }
            }
            _ => {}
        }
        prev = ch;
    }

    if let Some(start) = in_single.or(in_double) {
        return Some(line_up_to_cursor[start..].to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_completion() {
        let ac = Autocomplete::new();
        assert_eq!(ac.complete("  ").len(), 0);
    }

    #[test]
    fn test_main_completion() {
        let ac = Autocomplete::new();
        let res = ac.complete("m");
        assert!(res.iter().any(|i| i.label == "main:bind_key"));
        assert!(res.iter().any(|i| i.label == "main:bind_group_desc"));
    }

    #[test]
    fn test_colon_completion() {
        let ac = Autocomplete::new();
        let res = ac.complete("main:");
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].label, "main:bind_group_desc");
        assert_eq!(res[1].label, "main:bind_key");
    }

    #[test]
    fn test_method_completion() {
        let ac = Autocomplete::new();
        let res = ac.complete("main:bind_k");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].label, "main:bind_key");
    }

    #[test]
    fn test_context_ignores_previous_words() {
        let ac = Autocomplete::new();
        let res = ac.complete("local x = main:bind_g");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].label, "main:bind_group_desc");
    }

    #[test]
    fn test_dsl_global_completions() {
        let ac = Autocomplete::new();
        let res_anim = ac.complete("begin_a");
        assert!(res_anim.iter().any(|i| i.label == "begin_animation"));

        let res_cam = ac.complete("begin_c");
        assert!(res_cam.iter().any(|i| i.label == "begin_camera"));

        let res_follow = ac.complete("fol");
        assert_eq!(res_follow.len(), 1);
        assert_eq!(res_follow[0].label, "follow");
    }

    #[test]
    fn test_extract_prefix_supports_quoted_follow_targets() {
        assert_eq!(Autocomplete::extract_prefix("follow('V1.In"), "'V1.In");
        assert_eq!(
            Autocomplete::extract_prefix("follow(\"Track 1.Intro"),
            "\"Track 1.Intro"
        );
        assert_eq!(Autocomplete::extract_prefix("main:bind_k"), "main:bind_k");
    }

    #[test]
    fn test_dynamic_follow_target_completion() {
        let mut ac = Autocomplete::new();
        ac.set_follow_targets(vec![
            ("V1.Intro".into(), "轨道 V1 / 切片 Intro".into()),
            ("Track 1.video1.mp4".into(), "轨道 Track 1 / 切片 video1.mp4".into()),
        ]);

        let path_res = ac.complete("'V1.In");
        assert_eq!(path_res.len(), 1);
        assert_eq!(path_res[0].label, "'V1.Intro'");

        let spaced_res = ac.complete("\"Track 1.vid");
        assert_eq!(spaced_res.len(), 1);
        assert_eq!(spaced_res[0].label, "\"Track 1.video1.mp4\"");

        let bare_res = ac.complete("V1.In");
        assert_eq!(bare_res.len(), 1);
        assert_eq!(bare_res[0].label, "V1.Intro");
    }
}
