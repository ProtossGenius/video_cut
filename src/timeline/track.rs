use serde::{Deserialize, Serialize};

use super::clip::{Clip, ClipId};

/// 轨道ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub u64);

/// 轨道数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    
    /// 轨道上的所有切片
    /// 应当维持按 timeline_start 排序
    pub clips: Vec<Clip>,
    
    /// 是否置顶
    pub is_pinned: bool,
    /// 置顶顺序 (若为None则非置顶，或按自然顺序)
    pub pin_order: Option<u32>,
}

impl Track {
    pub fn new(id: TrackId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            clips: Vec::new(),
            is_pinned: false,
            pin_order: None,
        }
    }
    
    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
        self.sort_clips();
    }
    
    pub fn remove_clip(&mut self, id: ClipId) -> Option<Clip> {
        if let Some(pos) = self.clips.iter().position(|c| c.id == id) {
            Some(self.clips.remove(pos))
        } else {
            None
        }
    }
    
    pub fn sort_clips(&mut self) {
        self.clips.sort_by_key(|c| c.timeline_start);
    }
}
