use crate::keybinding::Mode;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// 快捷键绑定定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBindingItem {
    pub mode: Mode,
    pub key: String,
    pub command: String,
    pub description: String,
}

/// 帮助面板中展示给用户的一条条目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpDisplayEntry {
    /// 快捷键或前缀
    pub key_label: String,
    /// 描述，若为未命名组则为 "..."
    pub description: String,
    /// 是否为折叠组（包含子快捷键）
    pub is_group: bool,
}

/// 快捷键与帮助面板注册管理系统
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelpSystem {
    pub bindings: Vec<KeyBindingItem>,
    pub group_descriptions: HashMap<String, String>,
}

impl HelpSystem {
    pub fn new() -> Self {
        let mut sys = Self::default();
        sys.register_default_bindings();
        sys
    }

    /// 注册默认常用快捷键
    pub fn register_default_bindings(&mut self) {
        self.register_binding(Mode::Normal, "s", ":split<CR>", "分割当前切片");
        self.register_binding(Mode::Normal, "v", "enter_visual", "进入范围选择模式");
        self.register_binding(Mode::Normal, "V", "enter_visual_line", "进入切片选择模式");
        self.register_binding(Mode::Normal, "w", "jump_next_audio", "跳转到下一个发声点");
        self.register_binding(Mode::Normal, "b", "jump_prev_audio", "跳转到上一个发声点");
        self.register_binding(Mode::Normal, "W", "jump_next_anchor", "跳转到下一个锚点");
        self.register_binding(Mode::Normal, "B", "jump_prev_anchor", "跳转到上一个锚点");
        self.register_binding(Mode::Normal, "J", "jump_next_cut", "跳转到下一个切片断点");
        self.register_binding(Mode::Normal, "K", "jump_prev_cut", "跳转到上一个切片断点");
        self.register_binding(Mode::Normal, "m", "set_clip_anchor", "在切片内打锚点");
        self.register_binding(Mode::Normal, "M", "set_track_anchor", "在轨道上打锚点");
        self.register_binding(Mode::Normal, "'", "goto_anchor", "跳转到指定锚点");
        self.register_binding(Mode::Normal, "i", "open_file_import", "打开媒体导入窗口");
        self.register_binding(Mode::Normal, ":", "enter_command", "命令行模式");
        self.register_binding(Mode::Normal, "?", "toggle_help", "打开/关闭帮助面板");

        self.register_binding(Mode::Visual, "-", "biset_left", "Biset 选取左半部分");
        self.register_binding(Mode::Visual, "=", "biset_right", "Biset 选取右半部分");
        self.register_binding(Mode::Visual, ":merge", ":merge", "合并选区切片");
        self.register_binding(Mode::Visual, ":mergecut", ":mergecut", "边缘切断并合并切片");
    }

    pub fn register_binding(&mut self, mode: Mode, key: &str, command: &str, desc: &str) {
        self.bindings.retain(|b| !(b.mode == mode && b.key == key));
        self.bindings.push(KeyBindingItem {
            mode,
            key: key.to_string(),
            command: command.to_string(),
            description: desc.to_string(),
        });
    }

    pub fn register_group_desc(&mut self, prefix: &str, desc: &str) {
        self.group_descriptions
            .insert(prefix.to_string(), desc.to_string());
    }

    /// 查询帮助列表（根据当前模式与已输入的前缀进行层级聚合）
    /// 遵循 PLAN.md：
    /// 如果若干条以相同字母开头的快捷键，且没有绑定组描述，显示为 "..."
    /// 如果绑定了组描述，显示组描述。进入前缀后展开下一层。
    pub fn query_help_items(&self, mode: Mode, prefix: &str) -> Vec<HelpDisplayEntry> {
        let matching: Vec<&KeyBindingItem> = self
            .bindings
            .iter()
            .filter(|b| b.mode == mode && b.key.starts_with(prefix))
            .collect();

        // 统计紧随 prefix 之后的一个字符或子前缀
        let mut groups: BTreeMap<String, Vec<&KeyBindingItem>> = BTreeMap::new();

        for item in matching {
            let suffix = &item.key[prefix.len()..];
            if suffix.is_empty() {
                // 完全等于 prefix
                groups.entry(item.key.clone()).or_default().push(item);
            } else {
                // 取下一个字符作为分支
                let next_char = suffix.chars().next().unwrap();
                let branch_key = format!("{}{}", prefix, next_char);
                groups.entry(branch_key).or_default().push(item);
            }
        }

        let mut results = Vec::new();

        for (key_prefix, items) in groups {
            if items.len() == 1 && items[0].key == key_prefix {
                // 单个确定的按键
                results.push(HelpDisplayEntry {
                    key_label: items[0].key.clone(),
                    description: items[0].description.clone(),
                    is_group: false,
                });
            } else {
                // 多条按键共享同一个前缀分支
                let desc = if let Some(custom_desc) = self.group_descriptions.get(&key_prefix) {
                    custom_desc.clone()
                } else if items.len() == 1 {
                    items[0].description.clone()
                } else {
                    "...".to_string()
                };

                results.push(HelpDisplayEntry {
                    is_group: items.len() > 1 || items[0].key.len() > key_prefix.len(),
                    key_label: key_prefix,
                    description: desc,
                });
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_exact_and_ellipsis_grouping() {
        let mut sys = HelpSystem::default();
        sys.register_binding(Mode::Normal, "ad1", ":add_audio", "添加音频");
        sys.register_binding(Mode::Normal, "ad2", ":add_video", "添加视频");
        sys.register_binding(Mode::Normal, "s", ":split", "分割轨道");

        // 1. 未绑定组描述时，前缀 "ad" 应显示为 "..."
        let items_root = sys.query_help_items(Mode::Normal, "");
        let a_entry = items_root
            .iter()
            .find(|e| e.key_label == "a")
            .expect("Should have 'a' group");
        assert_eq!(a_entry.description, "...");
        assert!(a_entry.is_group);

        let s_entry = items_root
            .iter()
            .find(|e| e.key_label == "s")
            .expect("Should have 's'");
        assert_eq!(s_entry.description, "分割轨道");
        assert!(!s_entry.is_group);

        // 2. 深入前缀 "a"
        let items_a = sys.query_help_items(Mode::Normal, "a");
        let ad_entry = items_a
            .iter()
            .find(|e| e.key_label == "ad")
            .expect("Should have 'ad'");
        assert_eq!(ad_entry.description, "...");

        // 3. 注册组描述后
        sys.register_group_desc("ad", "添加元素菜单");
        let items_a_with_desc = sys.query_help_items(Mode::Normal, "a");
        let ad_with_desc = items_a_with_desc
            .iter()
            .find(|e| e.key_label == "ad")
            .unwrap();
        assert_eq!(ad_with_desc.description, "添加元素菜单");

        // 4. 深入前缀 "ad" 展开最终叶子
        let items_ad = sys.query_help_items(Mode::Normal, "ad");
        assert_eq!(items_ad.len(), 2);
        assert_eq!(items_ad[0].key_label, "ad1");
        assert_eq!(items_ad[0].description, "添加音频");
        assert_eq!(items_ad[1].key_label, "ad2");
        assert_eq!(items_ad[1].description, "添加视频");
    }
}
