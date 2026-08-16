use serde::{Deserialize, Serialize};

use super::clip::FrameTime;
use super::track::{Track, TrackId};

/// 时间线核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub duration: FrameTime,
    pub tracks: Vec<Track>,
    
    /// 垃圾回收轨道，处于最底层
    pub trash_track: Track,
    
    // TODO: 为了快速查询重叠，应该引入 IntervalTree 进行空间索引缓存
    // 这里出于序列化考虑，IntervalTree 可以不序列化，仅在运行时基于 tracks 构建
    #[serde(skip)]
    pub is_dirty: bool,
}

impl Timeline {
    pub fn new() -> Self {
        let mut trash = Track::new(TrackId(0), "Trash Track");
        trash.is_pinned = false;
        
        Self {
            duration: FrameTime::ZERO,
            tracks: Vec::new(),
            trash_track: trash,
            is_dirty: true,
        }
    }
    
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.is_dirty = true;
    }
    
    pub fn track_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }
    
    pub fn remove_track(&mut self, id: TrackId) -> Option<Track> {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == id) {
            self.is_dirty = true;
            Some(self.tracks.remove(pos))
        } else {
            None
        }
    }
    
    /// 重新计算时间线总时长（最晚的一个切片的结束时间）
    pub fn recalculate_duration(&mut self) {
        let mut max_duration = FrameTime::ZERO;
        for track in &self.tracks {
            if let Some(last_clip) = track.clips.last() {
                let end = last_clip.timeline_end();
                if end > max_duration {
                    max_duration = end;
                }
            }
        }
        self.duration = max_duration;
        self.is_dirty = false;
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}
