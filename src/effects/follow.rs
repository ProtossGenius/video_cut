use crate::timeline::{Clip, FrameTime, Timeline};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FollowError {
    #[error("未找到跟随目标切片: '{0}'")]
    TargetNotFound(String),
    #[error("检测到切片跟随循环依赖: {0}")]
    CircularDependency(String),
}

/// 切片跟随基准点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowAnchorPoint {
    Start,
    End,
}

/// 切片跟随属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowProperty {
    /// 目标切片的唯一标识，支持 "track_name.clip_name" 或 "clip_id"
    pub target_identifier: String,
    /// 跟随基准点 (目标起点或终点)
    pub anchor: FollowAnchorPoint,
    /// 与基准点的相对时间偏移 (正数为向后偏移，负数为向前偏移)
    pub offset: FrameTime,
}

impl FollowProperty {
    pub fn new(target: impl Into<String>, anchor: FollowAnchorPoint, offset: FrameTime) -> Self {
        Self {
            target_identifier: target.into(),
            anchor,
            offset,
        }
    }

    /// 根据目标切片的当前时间线位置计算跟随切片的新起点
    pub fn compute_new_start(&self, target_clip: &Clip) -> FrameTime {
        let base_time = match self.anchor {
            FollowAnchorPoint::Start => target_clip.timeline_start,
            FollowAnchorPoint::End => target_clip.timeline_end(),
        };
        base_time + self.offset
    }
}

/// 跟随依赖关系图与拓扑解析器
#[derive(Default)]
pub struct FollowDependencyGraph {
    /// key: source clip name/id -> value: target identifier
    dependencies: HashMap<String, String>,
}

impl FollowDependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dependency(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Result<(), FollowError> {
        let src = source.into();
        let tgt = target.into();

        // 校验自环依赖
        if src == tgt {
            return Err(FollowError::CircularDependency(format!(
                "{} -> {}",
                src, tgt
            )));
        }

        // 临时插入并检测环路
        self.dependencies.insert(src.clone(), tgt);
        if let Some(cycle) = self.find_cycle() {
            self.dependencies.remove(&src);
            return Err(FollowError::CircularDependency(cycle));
        }

        Ok(())
    }

    /// 检测是否存在依赖环
    fn find_cycle(&self) -> Option<String> {
        for start_node in self.dependencies.keys() {
            let mut visited = HashSet::new();
            let mut current = start_node.clone();

            while let Some(next) = self.dependencies.get(&current) {
                if !visited.insert(current.clone()) {
                    return Some(format!("{} -> {}", current, next));
                }
                current = next.clone();
            }
        }
        None
    }

    /// 在时间线上解析指定目标标识符对应的切片
    pub fn resolve_target<'a>(timeline: &'a Timeline, identifier: &str) -> Option<&'a Clip> {
        // 1. 尝试按 "track_name.clip_name" 格式解析
        if let Some((track_part, clip_part)) = identifier.split_once('.') {
            if let Some(track) = timeline.tracks.iter().find(|t| t.name == track_part) {
                if let Some(clip) = track.clips.iter().find(|c| c.name == clip_part) {
                    return Some(clip);
                }
            }
        }

        // 2. 尝试全局切片名称或 ID 匹配
        for track in &timeline.tracks {
            for clip in &track.clips {
                if clip.name == identifier || clip.id.0.to_string() == identifier {
                    return Some(clip);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, ClipId, Track};

    #[test]
    fn test_follow_position_calculation() {
        let target = Clip::new(
            ClipId(1),
            "TargetClip".into(),
            AssetId(100),
            FrameTime(5_000_000), // 5s 起点
            FrameTime(3_000_000), // 3s 时长 (8s 终点)
        );

        // 跟随目标结束点 + 500ms
        let follow_end =
            FollowProperty::new("TargetClip", FollowAnchorPoint::End, FrameTime(500_000));
        assert_eq!(follow_end.compute_new_start(&target), FrameTime(8_500_000));

        // 跟随目标起始点 - 1s
        let follow_start = FollowProperty::new(
            "TargetClip",
            FollowAnchorPoint::Start,
            FrameTime(-1_000_000),
        );
        assert_eq!(
            follow_start.compute_new_start(&target),
            FrameTime(4_000_000)
        );
    }

    #[test]
    fn test_follow_circular_dependency_detection() {
        let mut graph = FollowDependencyGraph::new();

        assert!(graph.add_dependency("ClipA", "ClipB").is_ok());
        assert!(graph.add_dependency("ClipB", "ClipC").is_ok());

        // 引入环 A -> B -> C -> A
        let err = graph.add_dependency("ClipC", "ClipA");
        assert!(err.is_err());
        assert!(matches!(
            err.unwrap_err(),
            FollowError::CircularDependency(_)
        ));
    }

    #[test]
    fn test_follow_resolve_target() {
        let mut timeline = Timeline::new();
        let mut track = Track::new(crate::timeline::TrackId(1), "V1");
        let clip = Clip::new(
            ClipId(1),
            "Intro".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(5_000_000),
        );
        track.clips.push(clip);
        timeline.tracks.push(track);

        let resolved_by_path = FollowDependencyGraph::resolve_target(&timeline, "V1.Intro");
        assert!(resolved_by_path.is_some());
        assert_eq!(resolved_by_path.unwrap().name, "Intro");

        let resolved_by_name = FollowDependencyGraph::resolve_target(&timeline, "Intro");
        assert!(resolved_by_name.is_some());

        let resolved_none = FollowDependencyGraph::resolve_target(&timeline, "V1.NotFound");
        assert!(resolved_none.is_none());
    }
}
