use crate::timeline::FrameTime;
use serde::{Deserialize, Serialize};

/// 锚点类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorScope {
    /// 局部锚点，作用于某个具体的切片 (快捷键 'm')
    Local,
    /// 全局锚点，作用于整条轨道（绝对时间点，快捷键 'M'）
    Global,
}

/// 锚点数据结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorPoint {
    pub name: String,
    pub description: String,

    /// 锚点的位置。
    /// 若为 Local，则相对于切片的 source_in。
    /// 若为 Global，则为时间线绝对位置。
    pub position: FrameTime,

    pub scope: AnchorScope,
}

impl AnchorPoint {
    pub fn new(name: impl Into<String>, position: FrameTime, scope: AnchorScope) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            position,
            scope,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// 锚点集合管理器
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorRegistry {
    pub anchors: Vec<AnchorPoint>,
}

impl AnchorRegistry {
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
        }
    }

    /// 插入或覆盖同名锚点
    pub fn set_anchor(&mut self, anchor: AnchorPoint) {
        if let Some(pos) = self
            .anchors
            .iter()
            .position(|a| a.name == anchor.name && a.scope == anchor.scope)
        {
            self.anchors[pos] = anchor;
        } else {
            self.anchors.push(anchor);
        }
    }

    /// 根据名字查询锚点
    pub fn get_anchor(&self, name: &str) -> Option<&AnchorPoint> {
        self.anchors.iter().find(|a| a.name == name)
    }

    /// 寻找下一个锚点 (快捷键 'W')
    pub fn find_next_anchor(&self, current_pos: FrameTime) -> Option<&AnchorPoint> {
        self.anchors
            .iter()
            .filter(|a| a.position > current_pos)
            .min_by_key(|a| a.position)
    }

    /// 寻找上一个锚点 (快捷键 'B')
    pub fn find_prev_anchor(&self, current_pos: FrameTime) -> Option<&AnchorPoint> {
        self.anchors
            .iter()
            .filter(|a| a.position < current_pos)
            .max_by_key(|a| a.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_registry_set_and_jump() {
        let mut reg = AnchorRegistry::new();
        reg.set_anchor(AnchorPoint::new(
            "a",
            FrameTime(1_000_000),
            AnchorScope::Global,
        ));
        reg.set_anchor(AnchorPoint::new(
            "b",
            FrameTime(3_000_000),
            AnchorScope::Global,
        ));
        reg.set_anchor(AnchorPoint::new(
            "c",
            FrameTime(7_000_000),
            AnchorScope::Global,
        ));

        // 覆盖同名锚点
        reg.set_anchor(AnchorPoint::new(
            "b",
            FrameTime(4_000_000),
            AnchorScope::Global,
        ));
        assert_eq!(reg.get_anchor("b").unwrap().position, FrameTime(4_000_000));

        // 寻找 2s 后的下一个锚点 -> b (4s)
        let next = reg.find_next_anchor(FrameTime(2_000_000));
        assert!(next.is_some());
        assert_eq!(next.unwrap().name, "b");

        // 寻找 5s 前的上一个锚点 -> b (4s)
        let prev = reg.find_prev_anchor(FrameTime(5_000_000));
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().name, "b");
    }
}
