use std::collections::HashMap;
use crate::keybinding::action::Action;

/// Vim 风格键盘宏录制与回放器
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
}
