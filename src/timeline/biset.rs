use crate::timeline::FrameTime;
use serde::{Deserialize, Serialize};

/// 二分法快速区间筛选会话 (Biset Session)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BisetSession {
    pub initial_range: (FrameTime, FrameTime),
    pub current_range: (FrameTime, FrameTime),
    pub history_stack: Vec<(FrameTime, FrameTime)>,
    pub is_active: bool,
}

impl BisetSession {
    pub fn new(start: FrameTime, end: FrameTime) -> Self {
        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        Self {
            initial_range: (s, e),
            current_range: (s, e),
            history_stack: Vec::new(),
            is_active: true,
        }
    }

    /// 当前区间的二分中点
    pub fn midpoint(&self) -> FrameTime {
        let mid_us = (self.current_range.0 .0 + self.current_range.1 .0) / 2;
        FrameTime(mid_us)
    }

    /// 选取左半部分 (按 '-')
    pub fn select_left(&mut self) {
        if !self.is_active {
            return;
        }
        self.history_stack.push(self.current_range);
        let mid = self.midpoint();
        self.current_range.1 = mid;
    }

    /// 选取右半部分 (按 '=')
    pub fn select_right(&mut self) {
        if !self.is_active {
            return;
        }
        self.history_stack.push(self.current_range);
        let mid = self.midpoint();
        self.current_range.0 = mid;
    }

    /// 返回上一层二分层级 (按 Backspace)
    pub fn step_back(&mut self) -> bool {
        if let Some(prev) = self.history_stack.pop() {
            self.current_range = prev;
            true
        } else {
            false
        }
    }

    /// 确认选择点 (按 Enter)，返回最终决策切断点
    pub fn confirm(&mut self) -> FrameTime {
        self.is_active = false;
        self.midpoint()
    }

    /// 取消选择 (按 Esc)
    pub fn cancel(&mut self) {
        self.is_active = false;
        self.current_range = self.initial_range;
        self.history_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biset_binary_division() {
        // 初始区间 0s ~ 8s
        let mut session = BisetSession::new(FrameTime(0), FrameTime(8_000_000));
        assert_eq!(session.midpoint(), FrameTime(4_000_000));

        // 选左半边 -> 0s ~ 4s
        session.select_left();
        assert_eq!(session.current_range, (FrameTime(0), FrameTime(4_000_000)));
        assert_eq!(session.midpoint(), FrameTime(2_000_000));

        // 再选右半边 -> 2s ~ 4s
        session.select_right();
        assert_eq!(
            session.current_range,
            (FrameTime(2_000_000), FrameTime(4_000_000))
        );
        assert_eq!(session.midpoint(), FrameTime(3_000_000));

        // 退回一步 (Backspace) -> 0s ~ 4s
        assert!(session.step_back());
        assert_eq!(session.current_range, (FrameTime(0), FrameTime(4_000_000)));

        // 确认切割
        let cut_point = session.confirm();
        assert_eq!(cut_point, FrameTime(2_000_000));
        assert!(!session.is_active);
    }
}
