use serde::{Deserialize, Serialize};

use super::clip::FrameTime;

/// 锚点类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorScope {
    /// 局部锚点，作用于某个具体的切片
    Local,
    /// 全局锚点，作用于整条轨道（绝对时间点）
    Global,
}

/// 锚点数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
