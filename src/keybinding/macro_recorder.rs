use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::keybinding::action::Action;

/// Vim 风格键盘宏录制与回放器
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroRecorder {
    pub recording_register: Option<char>,
    pub current_macro: Vec<Action>,
    pub registers: HashMap<char, Vec<Action>>,
    pub last_played_register: Option<char>,
}

impl MacroRecorder {
    /// 当前是否处于宏录制状态
    pub fn is_recording(&self) -> bool {
        self.recording_register.is_some()
    }

    /// 获取当前正在录制的寄存器名
    pub fn current_register(&self) -> Option<char> {
        self.recording_register
    }

    /// 开始录制宏至指定寄存器 (a-z)
    pub fn start_recording(&mut self, reg: char) {
        let clean_reg = reg.to_ascii_lowercase();
        self.recording_register = Some(clean_reg);
        self.current_macro.clear();
    }

    /// 停止当前宏录制并保存至寄存器
    pub fn stop_recording(&mut self) -> Option<char> {
        if let Some(reg) = self.recording_register.take() {
            self.registers.insert(reg, self.current_macro.clone());
            Some(reg)
        } else {
            None
        }
    }

    /// 记录一条触发的动作
    pub fn record_action(&mut self, action: Action) {
        if self.recording_register.is_some() {
            self.current_macro.push(action);
        }
    }

    /// 从指定寄存器获取宏动作列表准备回放
    pub fn get_macro(&mut self, reg: char) -> Option<Vec<Action>> {
        let clean_reg = reg.to_ascii_lowercase();
        if let Some(actions) = self.registers.get(&clean_reg).cloned() {
            self.last_played_register = Some(clean_reg);
            Some(actions)
        } else {
            None
        }
    }

    /// 获取上一次执行过的宏动作列表 (@@ 重复执行)
    pub fn get_last_macro(&mut self) -> Option<Vec<Action>> {
        if let Some(last) = self.last_played_register {
            self.registers.get(&last).cloned()
        } else {
            None
        }
    }

    /// 保存所有已录制的宏至 JSON 文件
    pub fn save_to_json(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        std::fs::write(path, json_str)
    }

    /// 从 JSON 文件读取载入宏数据
    pub fn load_from_json(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let recorder: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(recorder)
    }

    /// 导出指定寄存器的宏为可执行的 Lua 自动化脚本
    pub fn export_to_lua(&self, reg: char) -> Option<String> {
        let clean_reg = reg.to_ascii_lowercase();
        let actions = self.registers.get(&clean_reg)?;
        let mut script = format!("-- Macro '{}' exported from VideoCut\nfunction replay_macro_{}()\n", clean_reg, clean_reg);
        for action in actions {
            match action {
                Action::PlayPause => script.push_str("    app:play_pause()\n"),
                Action::Split => script.push_str("    app:split()\n"),
                Action::MoveLeft => script.push_str("    app:move_left()\n"),
                Action::MoveRight => script.push_str("    app:move_right()\n"),
                Action::Delete => script.push_str("    app:delete()\n"),
                Action::RippleDelete => script.push_str("    app:ripple_delete()\n"),
                Action::CloseGaps => script.push_str("    app:close_gaps()\n"),
                Action::LuaCommand(cmd) => script.push_str(&format!("    app:run_command({:?})\n", cmd)),
                _ => script.push_str(&format!("    -- {:?}\n", action)),
            }
        }
        script.push_str("end\n");
        Some(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_record_and_replay() {
        let mut recorder = MacroRecorder::default();
        assert!(!recorder.is_recording());

        // 1. 开始录制到寄存器 'a'
        recorder.start_recording('a');
        assert!(recorder.is_recording());
        assert_eq!(recorder.current_register(), Some('a'));

        // 2. 录制动作
        recorder.record_action(Action::PlayPause);
        recorder.record_action(Action::Split);
        recorder.record_action(Action::MoveRight);

        // 3. 停止录制
        let saved_reg = recorder.stop_recording();
        assert_eq!(saved_reg, Some('a'));
        assert!(!recorder.is_recording());

        // 4. 回放 @a
        let replayed = recorder.get_macro('a').unwrap();
        assert_eq!(replayed.len(), 3);
        assert_eq!(replayed[0], Action::PlayPause);
        assert_eq!(replayed[1], Action::Split);
        assert_eq!(replayed[2], Action::MoveRight);

        // 5. 回放 @@ (重复上一次宏)
        let last_replayed = recorder.get_last_macro().unwrap();
        assert_eq!(last_replayed, replayed);
    }

    #[test]
    fn test_macro_persistence_and_lua_export() {
        let mut recorder = MacroRecorder::default();
        recorder.start_recording('m');
        recorder.record_action(Action::Split);
        recorder.record_action(Action::RippleDelete);
        recorder.stop_recording();

        // 1. 导出为 Lua 脚本
        let lua_code = recorder.export_to_lua('m').unwrap();
        assert!(lua_code.contains("function replay_macro_m()"));
        assert!(lua_code.contains("app:split()"));
        assert!(lua_code.contains("app:ripple_delete()"));

        // 2. 保存至 JSON 文件并载入验证
        let temp_path = std::env::temp_dir().join("test_macros_persistence.json");
        recorder.save_to_json(&temp_path).unwrap();

        let loaded = MacroRecorder::load_from_json(&temp_path).unwrap();
        assert_eq!(loaded.registers.get(&'m'), recorder.registers.get(&'m'));

        let _ = std::fs::remove_file(temp_path);
    }
}
