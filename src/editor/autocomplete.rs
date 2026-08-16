#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub detail: String,
}

pub struct Autocomplete {
    keywords: Vec<CompletionItem>,
}

impl Autocomplete {
    pub fn new() -> Self {
        let mut keywords = vec![
            // main object
            CompletionItem { label: "main:bind_key".into(), detail: "(mode, key, cmd, desc)".into() },
            CompletionItem { label: "main:bind_group_desc".into(), detail: "(prefix, desc)".into() },
            // animation object
            CompletionItem { label: "animation:begin_animation".into(), detail: "()".into() },
            CompletionItem { label: "animation:end_animation".into(), detail: "()".into() },
            CompletionItem { label: "animation:frame".into(), detail: "(time)".into() },
            // actions
            CompletionItem { label: "actions:resize".into(), detail: "(scale, interpolate)".into() },
            CompletionItem { label: "actions:moveto".into(), detail: "(x, y)".into() },
            CompletionItem { label: "actions:rotate".into(), detail: "(angle)".into() },
            // camera object
            CompletionItem { label: "camera:begin_camera".into(), detail: "()".into() },
            CompletionItem { label: "camera:end_camera".into(), detail: "()".into() },
            CompletionItem { label: "camera:focuse".into(), detail: "(rect, interpolate)".into() },
            CompletionItem { label: "camera:frame".into(), detail: "(time)".into() },
        ];
        
        keywords.sort_by(|a, b| a.label.cmp(&b.label));

        Self { keywords }
    }

    /// 根据光标之前的这一行文本，提取出当前的输入单词，并返回匹配的补全列表
    pub fn complete(&self, line_up_to_cursor: &str) -> Vec<CompletionItem> {
        // 向前查找合法的单词字符（字母、数字、下划线、冒号）
        let prefix: String = line_up_to_cursor
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
            .collect();
        let prefix: String = prefix.chars().rev().collect();
        
        if prefix.is_empty() {
            return vec![];
        }

        self.keywords.iter()
            .filter(|item| item.label.starts_with(&prefix))
            .cloned()
            .collect()
    }
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
}
