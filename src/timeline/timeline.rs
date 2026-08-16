use serde::{Deserialize, Serialize};

use super::anchor::AnchorRegistry;
use super::clip::FrameTime;
use super::track::{Track, TrackId};

/// 时间线核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub duration: FrameTime,
    pub tracks: Vec<Track>,

    /// 垃圾回收轨道，处于最底层
    pub trash_track: Track,

    /// 全局锚点注册表
    pub global_anchors: AnchorRegistry,

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
            global_anchors: AnchorRegistry::new(),
            is_dirty: true,
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.is_dirty = true;
    }

    /// 在指定轨道下方新增一条轨道 (Shift+Enter)
    pub fn insert_track_below(&mut self, after_id: TrackId, new_track: Track) {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == after_id) {
            self.tracks.insert(pos + 1, new_track);
        } else {
            self.tracks.push(new_track);
        }
        self.is_dirty = true;
    }

    /// 在指定轨道上方新增一条轨道
    pub fn insert_track_above(&mut self, before_id: TrackId, new_track: Track) {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == before_id) {
            self.tracks.insert(pos, new_track);
        } else {
            self.tracks.insert(0, new_track);
        }
        self.is_dirty = true;
    }

    pub fn track_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }

    pub fn track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    pub fn remove_track(&mut self, id: TrackId) -> Option<Track> {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == id) {
            self.is_dirty = true;
            Some(self.tracks.remove(pos))
        } else {
            None
        }
    }

    /// 轨道上移 (Alt+k)
    pub fn move_track_up(&mut self, id: TrackId) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == id) {
            if pos > 0 {
                self.tracks.swap(pos, pos - 1);
                self.is_dirty = true;
                return true;
            }
        }
        false
    }

    /// 轨道下移 (Alt+j)
    pub fn move_track_down(&mut self, id: TrackId) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == id) {
            if pos < self.tracks.len().saturating_sub(1) {
                self.tracks.swap(pos, pos + 1);
                self.is_dirty = true;
                return true;
            }
        }
        false
    }

    /// 置顶轨道 (快捷键 '^')
    pub fn pin_track(&mut self, id: TrackId) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == id) {
            let mut track = self.tracks.remove(pos);
            track.is_pinned = true;
            self.tracks.insert(0, track);
            self.is_dirty = true;
            return true;
        }
        false
    }

    /// 取消置顶轨道 (快捷键 '$')
    pub fn unpin_track(&mut self, id: TrackId) -> bool {
        if let Some(track) = self.track_mut(id) {
            track.is_pinned = false;
            self.is_dirty = true;
            return true;
        }
        false
    }

    /// 查找当前时间之后的下一个切片边缘（起点或终点）(快捷键 'J')
    pub fn next_clip_edge(&self, current: FrameTime) -> Option<FrameTime> {
        let mut edges = Vec::new();
        for track in &self.tracks {
            for clip in &track.clips {
                if clip.timeline_start > current {
                    edges.push(clip.timeline_start);
                }
                let end = clip.timeline_end();
                if end > current {
                    edges.push(end);
                }
            }
        }
        edges.into_iter().min()
    }

    /// 查找当前时间之前的上一个切片边缘（起点或终点）(快捷键 'K')
    pub fn prev_clip_edge(&self, current: FrameTime) -> Option<FrameTime> {
        let mut edges = Vec::new();
        for track in &self.tracks {
            for clip in &track.clips {
                if clip.timeline_start < current {
                    edges.push(clip.timeline_start);
                }
                let end = clip.timeline_end();
                if end < current {
                    edges.push(end);
                }
            }
        }
        edges.into_iter().max()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, Clip, ClipId};

    #[test]
    fn test_track_reordering_and_pinning() {
        let mut timeline = Timeline::new();
        timeline.add_track(Track::new(TrackId(1), "V1"));
        timeline.add_track(Track::new(TrackId(2), "V2"));
        timeline.add_track(Track::new(TrackId(3), "A1"));

        // 下移 V1 -> [V2, V1, A1]
        assert!(timeline.move_track_down(TrackId(1)));
        assert_eq!(timeline.tracks[0].id, TrackId(2));
        assert_eq!(timeline.tracks[1].id, TrackId(1));

        // 上移 V1 -> [V1, V2, A1]
        assert!(timeline.move_track_up(TrackId(1)));
        assert_eq!(timeline.tracks[0].id, TrackId(1));

        // 置顶 A1 (快捷键 ^) -> [A1, V1, V2]
        assert!(timeline.pin_track(TrackId(3)));
        assert_eq!(timeline.tracks[0].id, TrackId(3));
        assert!(timeline.tracks[0].is_pinned);

        // 取消置顶 A1 (快捷键 $)
        assert!(timeline.unpin_track(TrackId(3)));
        assert!(!timeline.tracks[0].is_pinned);
    }

    #[test]
    fn test_insert_track_below() {
        let mut timeline = Timeline::new();
        timeline.add_track(Track::new(TrackId(1), "V1"));
        timeline.add_track(Track::new(TrackId(2), "A1"));

        // 在 V1 下方插入 V2
        timeline.insert_track_below(TrackId(1), Track::new(TrackId(3), "V2"));
        assert_eq!(timeline.tracks.len(), 3);
        assert_eq!(timeline.tracks[1].id, TrackId(3));
    }

    #[test]
    fn test_insert_track_above() {
        let mut timeline = Timeline::new();
        timeline.add_track(Track::new(TrackId(1), "V1"));
        timeline.add_track(Track::new(TrackId(2), "A1"));

        timeline.insert_track_above(TrackId(2), Track::new(TrackId(3), "V2"));
        assert_eq!(timeline.tracks.len(), 3);
        assert_eq!(timeline.tracks[0].id, TrackId(1));
        assert_eq!(timeline.tracks[1].id, TrackId(3));
        assert_eq!(timeline.tracks[2].id, TrackId(2));
    }

    #[test]
    fn test_clip_edge_jumping() {
        let mut timeline = Timeline::new();
        let mut track = Track::new(TrackId(1), "V1");
        track.add_clip(Clip::new(
            ClipId(1),
            "clip1.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(3_000_000),
        )); // [0, 3s]
        track.add_clip(Clip::new(
            ClipId(2),
            "clip2.mp4".into(),
            AssetId(2),
            FrameTime(5_000_000),
            FrameTime(4_000_000),
        )); // [5s, 9s]
        timeline.add_track(track);

        // 从 1s 向后找下一个边缘 -> 3s (clip1 终点)
        assert_eq!(
            timeline.next_clip_edge(FrameTime(1_000_000)),
            Some(FrameTime(3_000_000))
        );
        // 从 3s 向后找 -> 5s (clip2 起点)
        assert_eq!(
            timeline.next_clip_edge(FrameTime(3_000_000)),
            Some(FrameTime(5_000_000))
        );
        // 从 7s 向前找上一个边缘 -> 5s (clip2 起点)
        assert_eq!(
            timeline.prev_clip_edge(FrameTime(7_000_000)),
            Some(FrameTime(5_000_000))
        );
        // 从 5s 向前找 -> 3s (clip1 终点)
        assert_eq!(
            timeline.prev_clip_edge(FrameTime(5_000_000)),
            Some(FrameTime(3_000_000))
        );
    }
}
