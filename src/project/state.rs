use serde::{Deserialize, Serialize};

use crate::timeline::Timeline;
// 未来引入 MediaPool 和配置等

/// 代表一个完整的视频编辑工程状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectState {
    pub name: String,
    pub timeline: Timeline,
    // pub media_pool: MediaPool,
    // pub settings: ProjectSettings,
}

impl ProjectState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            timeline: Timeline::new(),
        }
    }
}
