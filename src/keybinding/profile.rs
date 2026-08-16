use anyhow::{Context, Result};
use mlua::{Lua, MultiValue, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{HelpSystem, KeyBindingItem, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinKeymapProfile {
    Vim,
    PremierePro,
    FinalCutPro,
}

impl Default for BuiltinKeymapProfile {
    fn default() -> Self {
        Self::Vim
    }
}

impl BuiltinKeymapProfile {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "vim" | "default" => Some(Self::Vim),
            "premiere" | "premierepro" | "premiere_pro" | "pr" => Some(Self::PremierePro),
            "finalcut" | "final_cut" | "final_cut_pro" | "fcp" | "fcpx" => {
                Some(Self::FinalCutPro)
            }
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Vim => "Vim",
            Self::PremierePro => "Premiere Pro",
            Self::FinalCutPro => "Final Cut Pro",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Vim => "VIM",
            Self::PremierePro => "PR",
            Self::FinalCutPro => "FCP",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Vim => Self::PremierePro,
            Self::PremierePro => Self::FinalCutPro,
            Self::FinalCutPro => Self::Vim,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyBindingConflictKind {
    ExactDuplicate,
    ExistingPrefix,
    IncomingPrefix,
}

impl KeyBindingConflictKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ExactDuplicate => "重复绑定",
            Self::ExistingPrefix => "已有前缀抢占",
            Self::IncomingPrefix => "新绑定成为前缀",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBindingConflict {
    pub mode: Mode,
    pub incoming_key: String,
    pub existing_key: String,
    pub kind: KeyBindingConflictKind,
    pub incoming_command: String,
    pub existing_command: String,
}

impl KeyBindingConflict {
    pub fn summary_line(&self) -> String {
        format!(
            "[{}] {}: '{}' -> {} 与 '{}' -> {} 冲突",
            self.mode.as_str(),
            self.kind.name(),
            self.incoming_key,
            self.incoming_command,
            self.existing_key,
            self.existing_command
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapProfile {
    pub name: String,
    pub bindings: Vec<KeyBindingItem>,
    pub group_descriptions: HashMap<String, String>,
}

impl Default for KeymapProfile {
    fn default() -> Self {
        Self::builtin(BuiltinKeymapProfile::Vim)
    }
}

impl KeymapProfile {
    pub fn builtin(kind: BuiltinKeymapProfile) -> Self {
        let mut profile = Self {
            name: kind.display_name().to_string(),
            bindings: Vec::new(),
            group_descriptions: HashMap::new(),
        };

        match kind {
            BuiltinKeymapProfile::Vim => {
                profile.push_binding(Mode::Normal, "Space", "play_pause", "播放 / 暂停");
                profile.push_binding(Mode::Normal, "h", "move_left", "播放头向左微调");
                profile.push_binding(Mode::Normal, "l", "move_right", "播放头向右微调");
                profile.push_binding(Mode::Normal, "j", "move_down", "选择下方轨道");
                profile.push_binding(Mode::Normal, "k", "move_up", "选择上方轨道");
                profile.push_binding(Mode::Normal, "s", ":split<CR>", "分割当前切片");
                profile.push_binding(Mode::Normal, "v", "enter_visual", "进入范围选择");
                profile.push_binding(Mode::Normal, "V", "enter_visual_line", "进入切片多选");
                profile.push_binding(Mode::Normal, "u", ":undo<CR>", "撤销上一步");
                profile.push_binding(Mode::Normal, "r", ":redo<CR>", "重做上一步");
                profile.push_binding(Mode::Normal, ":", "enter_command", "进入命令模式");
                profile.push_binding(Mode::Normal, "?", "toggle_help", "打开快捷键帮助");
            }
            BuiltinKeymapProfile::PremierePro => {
                profile.push_binding(Mode::Normal, "Space", "play_pause", "播放 / 暂停");
                profile.push_binding(Mode::Normal, "J", "move_left", "左向预览 / 后退");
                profile.push_binding(Mode::Normal, "L", "move_right", "右向预览 / 前进");
                profile.push_binding(Mode::Normal, "K", "play_pause", "停止 / 暂停");
                profile.push_binding(Mode::Normal, "ArrowUp", "move_up", "选择上方轨道");
                profile.push_binding(Mode::Normal, "ArrowDown", "move_down", "选择下方轨道");
                profile.push_binding(Mode::Normal, "C", ":split<CR>", "Razor / 切刀分割");
                profile.push_binding(Mode::Normal, "V", "enter_visual_line", "Selection / 选择工具");
                profile.push_binding(Mode::Normal, "I", "open_file_import", "导入媒体素材");
                profile.push_binding(Mode::Normal, "Ctrl+Z", ":undo<CR>", "撤销上一步");
                profile.push_binding(Mode::Normal, "Ctrl+Shift+Z", ":redo<CR>", "重做上一步");
                profile.push_binding(Mode::Normal, ":", "enter_command", "进入命令模式");
            }
            BuiltinKeymapProfile::FinalCutPro => {
                profile.push_binding(Mode::Normal, "Space", "play_pause", "播放 / 暂停");
                profile.push_binding(Mode::Normal, "J", "move_left", "左向预览 / 后退");
                profile.push_binding(Mode::Normal, "L", "move_right", "右向预览 / 前进");
                profile.push_binding(Mode::Normal, "K", "play_pause", "停止 / 暂停");
                profile.push_binding(Mode::Normal, "ArrowUp", "move_up", "选择上方轨道");
                profile.push_binding(Mode::Normal, "ArrowDown", "move_down", "选择下方轨道");
                profile.push_binding(Mode::Normal, "B", ":split<CR>", "Blade / 刀片分割");
                profile.push_binding(Mode::Normal, "A", "enter_visual_line", "Selection / 选择工具");
                profile.push_binding(Mode::Normal, "I", "open_file_import", "导入媒体素材");
                profile.push_binding(Mode::Normal, "Cmd+Z", ":undo<CR>", "撤销上一步");
                profile.push_binding(Mode::Normal, "Cmd+Shift+Z", ":redo<CR>", "重做上一步");
                profile.push_binding(Mode::Normal, ":", "enter_command", "进入命令模式");
            }
        }

        profile
    }

    pub fn push_binding(
        &mut self,
        mode: Mode,
        key: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) {
        self.bindings.push(KeyBindingItem {
            mode,
            key: key.into(),
            command: command.into(),
            description: description.into(),
        });
    }

    pub fn detect_conflicts(&self) -> Vec<KeyBindingConflict> {
        let mut tries: HashMap<Mode, BindingConflictTrie> = HashMap::new();
        let mut conflicts = Vec::new();

        for binding in &self.bindings {
            let trie = tries.entry(binding.mode).or_default();
            conflicts.extend(trie.insert_and_detect(binding));
        }

        conflicts
    }

    pub fn to_help_system(&self) -> HelpSystem {
        HelpSystem {
            bindings: self.bindings.clone(),
            group_descriptions: self.group_descriptions.clone(),
        }
    }

    pub fn save_to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("failed to serialize keymap profile as JSON")?;
        fs::write(path.as_ref(), content)
            .with_context(|| format!("failed to write keymap JSON: {}", path.as_ref().display()))
    }

    pub fn load_from_json(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read keymap JSON: {}", path.as_ref().display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("failed to parse keymap JSON: {}", path.as_ref().display()))
    }

    pub fn export_to_lua(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("-- VideoCut keymap profile: {}\n", self.name));

        let mut group_items: Vec<(&String, &String)> = self.group_descriptions.iter().collect();
        group_items.sort_by(|a, b| a.0.cmp(b.0));
        for (prefix, desc) in group_items {
            out.push_str(&format!(
                "main:bind_group_desc({:?}, {:?})\n",
                prefix, desc
            ));
        }
        if !self.group_descriptions.is_empty() {
            out.push('\n');
        }

        for binding in &self.bindings {
            out.push_str(&format!(
                "main:bind_key({:?}, {:?}, {:?}, {:?})\n",
                binding.mode.as_binding_str(),
                binding.key,
                binding.command,
                binding.description
            ));
        }
        out
    }

    pub fn load_from_lua(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read keymap Lua file: {}", path.display()))?;

        let lua = Lua::new();
        let globals = lua.globals();
        let _ = globals.set("os", Value::Nil);
        let _ = globals.set("io", Value::Nil);
        let _ = globals.set("debug", Value::Nil);
        let _ = globals.set("package", Value::Nil);
        let _ = globals.set("dofile", Value::Nil);
        let _ = globals.set("loadfile", Value::Nil);

        let profile = Arc::new(Mutex::new(Self {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("imported_keymap")
                .to_string(),
            bindings: Vec::new(),
            group_descriptions: HashMap::new(),
        }));

        let main_table = lua
            .create_table()
            .map_err(|e| anyhow::anyhow!("failed to create Lua main table: {}", e))?;

        {
            let profile = Arc::clone(&profile);
            let bind_key_fn = lua
                .create_function(move |_, args: MultiValue| {
                    let mut strings: Vec<String> = Vec::new();
                    for val in args {
                        if let Value::String(s) = val {
                            strings.push(s.to_string_lossy());
                        }
                    }

                    let (mode_str, key, command, description) = match strings.len() {
                        4 => (&strings[0], &strings[1], &strings[2], &strings[3]),
                        5 => (&strings[1], &strings[2], &strings[3], &strings[4]),
                        _ => return Ok(()),
                    };

                    profile
                        .lock()
                        .map_err(|_| mlua::Error::RuntimeError("keymap profile lock poisoned".into()))?
                        .push_binding(
                            Mode::from_str_loose(mode_str),
                            key,
                            command,
                            description,
                        );
                    Ok(())
                })
                .map_err(|e| anyhow::anyhow!("failed to create bind_key collector: {}", e))?;
            main_table
                .set("bind_key", bind_key_fn)
                .map_err(|e| anyhow::anyhow!("failed to register bind_key collector: {}", e))?;
        }

        {
            let profile = Arc::clone(&profile);
            let bind_group_fn = lua
                .create_function(move |_, args: MultiValue| {
                    let mut strings: Vec<String> = Vec::new();
                    for val in args {
                        if let Value::String(s) = val {
                            strings.push(s.to_string_lossy());
                        }
                    }

                    let (prefix, description) = match strings.len() {
                        2 => (&strings[0], &strings[1]),
                        3 => (&strings[1], &strings[2]),
                        _ => return Ok(()),
                    };

                    profile
                        .lock()
                        .map_err(|_| mlua::Error::RuntimeError("keymap profile lock poisoned".into()))?
                        .group_descriptions
                        .insert(prefix.to_string(), description.to_string());
                    Ok(())
                })
                .map_err(|e| {
                    anyhow::anyhow!("failed to create bind_group_desc collector: {}", e)
                })?;
            main_table
                .set("bind_group_desc", bind_group_fn)
                .map_err(|e| {
                    anyhow::anyhow!("failed to register bind_group_desc collector: {}", e)
                })?;
        }

        globals
            .set("main", main_table)
            .map_err(|e| anyhow::anyhow!("failed to set Lua main table: {}", e))?;
        lua.load(&content)
            .exec()
            .map_err(|e| anyhow::anyhow!("failed to execute keymap Lua {}: {}", path.display(), e))?;

        let imported = profile
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to lock imported keymap profile"))?
            .clone();
        Ok(imported)
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let is_lua = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("lua"))
            .unwrap_or(false);

        if is_lua {
            fs::write(path, self.export_to_lua())
                .with_context(|| format!("failed to write keymap Lua: {}", path.display()))?;
            Ok(())
        } else {
            self.save_to_json(path)
        }
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let is_lua = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("lua"))
            .unwrap_or(false);

        if is_lua {
            Self::load_from_lua(path)
        } else {
            Self::load_from_json(path)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapProfileManager {
    pub active_profile: KeymapProfile,
    pub active_builtin: Option<BuiltinKeymapProfile>,
    pub conflicts: Vec<KeyBindingConflict>,
}

impl Default for KeymapProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeymapProfileManager {
    pub fn new() -> Self {
        let profile = KeymapProfile::builtin(BuiltinKeymapProfile::Vim);
        let conflicts = profile.detect_conflicts();
        Self {
            active_profile: profile,
            active_builtin: Some(BuiltinKeymapProfile::Vim),
            conflicts,
        }
    }

    pub fn switch_builtin(&mut self, kind: BuiltinKeymapProfile) {
        self.active_profile = KeymapProfile::builtin(kind);
        self.active_builtin = Some(kind);
        self.conflicts = self.active_profile.detect_conflicts();
    }

    pub fn cycle_builtin_profile(&mut self) -> BuiltinKeymapProfile {
        let next = self
            .active_builtin
            .unwrap_or(BuiltinKeymapProfile::FinalCutPro)
            .next();
        self.switch_builtin(next);
        next
    }

    pub fn import_from_path(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.active_profile = KeymapProfile::load_from_path(path)?;
        self.active_builtin = None;
        self.conflicts = self.active_profile.detect_conflicts();
        Ok(())
    }

    pub fn export_active_profile(&self, path: impl AsRef<Path>) -> Result<()> {
        self.active_profile.save_to_path(path)
    }

    pub fn active_profile_name(&self) -> &str {
        &self.active_profile.name
    }

    pub fn active_profile_short_label(&self) -> String {
        self.active_builtin
            .map(|kind| kind.short_label().to_string())
            .unwrap_or_else(|| self.active_profile.name.to_uppercase())
    }

    pub fn status_line(&self) -> String {
        format!(
            "{} ({} 条绑定, {} 个冲突)",
            self.active_profile.name,
            self.active_profile.bindings.len(),
            self.conflicts.len()
        )
    }

    pub fn conflict_report_lines(&self) -> Vec<String> {
        self.conflicts
            .iter()
            .map(KeyBindingConflict::summary_line)
            .collect()
    }

    pub fn as_help_system(&self) -> HelpSystem {
        self.active_profile.to_help_system()
    }
}

#[derive(Debug, Clone)]
struct BindingRecord {
    key: String,
    command: String,
}

#[derive(Debug, Clone, Default)]
struct BindingConflictNode {
    terminal: Option<BindingRecord>,
    children: HashMap<String, BindingConflictNode>,
}

#[derive(Debug, Clone, Default)]
struct BindingConflictTrie {
    root: BindingConflictNode,
}

impl BindingConflictTrie {
    fn insert_and_detect(&mut self, binding: &KeyBindingItem) -> Vec<KeyBindingConflict> {
        let tokens = tokenize_binding_key(&binding.key);
        let mut node = &mut self.root;
        let mut conflicts = Vec::new();

        for (idx, token) in tokens.iter().enumerate() {
            if let Some(existing) = &node.terminal {
                if idx < tokens.len() {
                    conflicts.push(KeyBindingConflict {
                        mode: binding.mode,
                        incoming_key: binding.key.clone(),
                        existing_key: existing.key.clone(),
                        kind: KeyBindingConflictKind::ExistingPrefix,
                        incoming_command: binding.command.clone(),
                        existing_command: existing.command.clone(),
                    });
                }
            }
            node = node.children.entry(token.clone()).or_default();
        }

        if let Some(existing) = &node.terminal {
            conflicts.push(KeyBindingConflict {
                mode: binding.mode,
                incoming_key: binding.key.clone(),
                existing_key: existing.key.clone(),
                kind: KeyBindingConflictKind::ExactDuplicate,
                incoming_command: binding.command.clone(),
                existing_command: existing.command.clone(),
            });
        }

        for existing in collect_terminals(node) {
            conflicts.push(KeyBindingConflict {
                mode: binding.mode,
                incoming_key: binding.key.clone(),
                existing_key: existing.key,
                kind: KeyBindingConflictKind::IncomingPrefix,
                incoming_command: binding.command.clone(),
                existing_command: existing.command,
            });
        }

        if node.terminal.is_none() {
            node.terminal = Some(BindingRecord {
                key: binding.key.clone(),
                command: binding.command.clone(),
            });
        }

        conflicts
    }
}

fn collect_terminals(node: &BindingConflictNode) -> Vec<BindingRecord> {
    let mut records = Vec::new();
    for child in node.children.values() {
        if let Some(terminal) = &child.terminal {
            records.push(terminal.clone());
        }
        records.extend(collect_terminals(child));
    }
    records
}

fn tokenize_binding_key(key: &str) -> Vec<String> {
    const SPECIAL_KEYS: &[&str] = &[
        "Space",
        "Enter",
        "Backspace",
        "Escape",
        "Tab",
        "Quote",
        "Slash",
        "ArrowUp",
        "ArrowDown",
        "ArrowLeft",
        "ArrowRight",
    ];

    if key.contains('+') || SPECIAL_KEYS.iter().any(|special| key.eq_ignore_ascii_case(special)) {
        vec![key.to_string()]
    } else {
        key.chars().map(|ch| ch.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_profiles_are_conflict_free() {
        for preset in [
            BuiltinKeymapProfile::Vim,
            BuiltinKeymapProfile::PremierePro,
            BuiltinKeymapProfile::FinalCutPro,
        ] {
            let profile = KeymapProfile::builtin(preset);
            assert!(profile.detect_conflicts().is_empty(), "{:?}", preset);
        }
    }

    #[test]
    fn test_conflict_detector_reports_prefix_and_duplicate() {
        let mut profile = KeymapProfile {
            name: "Conflict Demo".into(),
            bindings: Vec::new(),
            group_descriptions: HashMap::new(),
        };
        profile.push_binding(Mode::Normal, "a", ":alpha<CR>", "A");
        profile.push_binding(Mode::Normal, "ab", ":beta<CR>", "B");
        profile.push_binding(Mode::Normal, "a", ":again<CR>", "A2");

        let conflicts = profile.detect_conflicts();
        assert!(conflicts.iter().any(|c| c.kind == KeyBindingConflictKind::ExistingPrefix));
        assert!(conflicts.iter().any(|c| c.kind == KeyBindingConflictKind::ExactDuplicate));
    }

    #[test]
    fn test_keymap_json_and_lua_roundtrip() {
        let profile = KeymapProfile::builtin(BuiltinKeymapProfile::PremierePro);
        let tmp_dir = std::env::temp_dir().join(format!("vcut_keymap_test_{}", std::process::id()));
        let json_path = tmp_dir.join("premiere_keymap.json");
        let lua_path = tmp_dir.join("premiere_keymap.lua");
        let _ = std::fs::create_dir_all(&tmp_dir);

        profile.save_to_json(&json_path).unwrap();
        profile.save_to_path(&lua_path).unwrap();

        let from_json = KeymapProfile::load_from_json(&json_path).unwrap();
        let from_lua = KeymapProfile::load_from_lua(&lua_path).unwrap();

        assert_eq!(from_json.name, "Premiere Pro");
        assert_eq!(from_json.bindings.len(), profile.bindings.len());
        assert_eq!(from_lua.bindings.len(), profile.bindings.len());
        assert!(from_lua
            .bindings
            .iter()
            .any(|b| b.key == "C" && b.command == ":split<CR>"));

        let _ = std::fs::remove_dir_all(tmp_dir);
    }
}
