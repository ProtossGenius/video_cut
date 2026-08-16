use serde::{Deserialize, Serialize};
use crate::timeline::{AnchorRegistry, FrameTime, Timeline};

/// 吸附目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapTargetKind {
    /// 切片起始点
    ClipStart,
    /// 切片终止点
    ClipEnd,
    /// 全局锚点
    GlobalAnchor,
    /// 切片局部锚点
    LocalAnchor,
    /// 播放指针
    Playhead,
}

/// 吸附点信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapPoint {
    pub time: FrameTime,
    pub kind: SnapTargetKind,
    pub description: String,
}

/// 吸附计算命中结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapResult {
    pub snapped_time: FrameTime,
    pub snap_point: SnapPoint,
    pub distance_px: f32,
}

/// 磁性吸附引擎
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapEngine {
    pub enabled: bool,
    /// 屏幕像素吸附容差阈值 (默认 8.0px)
    pub threshold_px: f32,
}

impl Default for SnapEngine {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_px: 8.0,
        }
    }
}

impl SnapEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// 在时间线上收集所有潜在对齐点并寻找最近吸附点
    pub fn find_snap_point(
        &self,
        target_time: FrameTime,
        timeline: &Timeline,
        global_anchors: &AnchorRegistry,
        zoom_level: f32,
    ) -> Option<SnapResult> {
        if !self.enabled || zoom_level <= 0.0 {
            return None;
        }

        let us_per_sec = 1_000_000.0f32;
        let mut candidates = Vec::new();

        // 1. 收集所有轨道的切片起止点
        for track in &timeline.tracks {
            for clip in &track.clips {
                candidates.push(SnapPoint {
                    time: clip.timeline_start,
                    kind: SnapTargetKind::ClipStart,
                    description: format!("切片 '{}' 起点", clip.name),
                });
                candidates.push(SnapPoint {
                    time: clip.timeline_end(),
                    kind: SnapTargetKind::ClipEnd,
                    description: format!("切片 '{}' 终点", clip.name),
                });

                // 2. 收集切片局部锚点
                for (anchor_id, anchor_pt) in &clip.anchors {
                    candidates.push(SnapPoint {
                        time: clip.timeline_start + anchor_pt.position,
                        kind: SnapTargetKind::LocalAnchor,
                        description: format!("切片 '{}' 局部锚点 '{}'", clip.name, anchor_id),
                    });
                }
            }
        }

        // 3. 收集全局锚点
        for anchor in &global_anchors.anchors {
            candidates.push(SnapPoint {
                time: anchor.position,
                kind: SnapTargetKind::GlobalAnchor,
                description: format!("全局锚点 '{}'", anchor.name),
            });
        }

        // 寻找距离最近且在阈值内的候选点
        let mut best: Option<SnapResult> = None;
        let mut min_dist_px = self.threshold_px;

        for pt in candidates {
            let diff_us = (pt.time.0 - target_time.0).abs();
            let diff_px = (diff_us as f32 / us_per_sec) * zoom_level;

            if diff_px <= min_dist_px {
                min_dist_px = diff_px;
                best = Some(SnapResult {
                    snapped_time: pt.time,
                    snap_point: pt,
                    distance_px: diff_px,
                });
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, Clip, ClipId, Track, TrackId};

    #[test]
    fn test_snap_engine_finds_nearest_clip_edge() {
        let engine = SnapEngine::default();
        let mut timeline = Timeline::new();
        let mut track = Track::new(TrackId(1), "Video");
        let clip = Clip::new(ClipId(1), "clip1".into(), AssetId(1), FrameTime(0), FrameTime(5_000_000));
        track.add_clip(clip);
        timeline.tracks.push(track);

        let global_anchors = AnchorRegistry::new();
        let zoom = 100.0; // 100 px/sec

        // 目标时间: 4.95 秒 (距离 5.0 秒终点差 0.05s = 5px, 小于 8px 阈值)
        let target = FrameTime(4_950_000);
        let res = engine.find_snap_point(target, &timeline, &global_anchors, zoom);

        assert!(res.is_some());
        let snap = res.unwrap();
        assert_eq!(snap.snapped_time, FrameTime(5_000_000));
        assert_eq!(snap.snap_point.kind, SnapTargetKind::ClipEnd);
    }

    #[test]
    fn test_snap_engine_ignores_distant_points() {
        let engine = SnapEngine::default();
        let mut timeline = Timeline::new();
        let mut track = Track::new(TrackId(1), "Video");
        let clip = Clip::new(ClipId(1), "clip1".into(), AssetId(1), FrameTime(0), FrameTime(5_000_000));
        track.add_clip(clip);
        timeline.tracks.push(track);

        let global_anchors = AnchorRegistry::new();
        let zoom = 100.0;

        // 目标时间: 4.80 秒 (距离 5.0 秒终点差 0.20s = 20px, 超过 8px 阈值)
        let target = FrameTime(4_800_000);
        let res = engine.find_snap_point(target, &timeline, &global_anchors, zoom);

        assert!(res.is_none());
    }
}
