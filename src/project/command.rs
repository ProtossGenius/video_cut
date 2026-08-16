use anyhow::{anyhow, Result};
use std::collections::VecDeque;

use super::state::ProjectState;
use crate::timeline::{Clip, ClipId, FrameTime, TrackId};

/// 编辑器命令 trait，所有的剪辑操作必须实现此 trait
pub trait EditorCommand: Send + Sync {
    /// 执行命令，改变项目状态
    fn execute(&mut self, state: &mut ProjectState) -> Result<()>;

    /// 撤销命令，恢复状态
    fn undo(&mut self, state: &mut ProjectState) -> Result<()>;

    /// 尝试与上一个命令合并（对于连续相同且可压缩的微小操作）
    fn merge(&mut self, _other: &dyn EditorCommand) -> bool {
        false
    }
}

/// 1. 分割切片命令 (:split 或按 's')
pub struct SplitClipCommand {
    pub track_id: TrackId,
    pub clip_id: ClipId,
    pub split_time: FrameTime,
    pub generated_right_clip_id: ClipId,
    // 内部暂存用于 Undo
    original_clip: Option<Clip>,
}

impl SplitClipCommand {
    pub fn new(
        track_id: TrackId,
        clip_id: ClipId,
        split_time: FrameTime,
        new_clip_id: ClipId,
    ) -> Self {
        Self {
            track_id,
            clip_id,
            split_time,
            generated_right_clip_id: new_clip_id,
            original_clip: None,
        }
    }
}

impl EditorCommand for SplitClipCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        let clip_pos = track
            .clips
            .iter()
            .position(|c| c.id == self.clip_id)
            .ok_or_else(|| anyhow!("Clip not found"))?;

        let target = &track.clips[clip_pos];
        if self.split_time <= target.timeline_start || self.split_time >= target.timeline_end() {
            return Err(anyhow!("Split time is outside of the clip range"));
        }

        self.original_clip = Some(target.clone());

        // 计算左右分割点
        let offset_in_clip = self.split_time - target.timeline_start;
        let mut left_clip = target.clone();
        left_clip.source_out = left_clip.source_in + offset_in_clip;

        let mut right_clip = target.clone();
        right_clip.id = self.generated_right_clip_id;
        right_clip.timeline_start = self.split_time;
        right_clip.source_in = left_clip.source_out;

        track.clips.remove(clip_pos);
        track.clips.insert(clip_pos, left_clip);
        track.clips.insert(clip_pos + 1, right_clip);
        track.sort_clips();

        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let original = self
            .original_clip
            .take()
            .ok_or_else(|| anyhow!("No original clip state for undo"))?;

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        track
            .clips
            .retain(|c| c.id != self.clip_id && c.id != self.generated_right_clip_id);
        track.add_clip(original);

        Ok(())
    }
}

/// 2. 删除切片至垃圾回收轨道命令
pub struct DeleteClipToTrashCommand {
    pub source_track_id: TrackId,
    pub clip_id: ClipId,
    deleted_clip: Option<Clip>,
}

impl DeleteClipToTrashCommand {
    pub fn new(source_track_id: TrackId, clip_id: ClipId) -> Self {
        Self {
            source_track_id,
            clip_id,
            deleted_clip: None,
        }
    }
}

impl EditorCommand for DeleteClipToTrashCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        let track = state
            .timeline
            .track_mut(self.source_track_id)
            .ok_or_else(|| anyhow!("Source track not found"))?;

        let clip = track
            .remove_clip(self.clip_id)
            .ok_or_else(|| anyhow!("Clip not found in track"))?;

        self.deleted_clip = Some(clip.clone());

        // 追加到垃圾回收轨道末尾
        state.timeline.trash_track.add_clip(clip);
        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let clip = self
            .deleted_clip
            .take()
            .ok_or_else(|| anyhow!("No deleted clip for undo"))?;

        // 从垃圾回收轨道移除
        state.timeline.trash_track.remove_clip(clip.id);

        // 放回原轨道
        let track = state
            .timeline
            .track_mut(self.source_track_id)
            .ok_or_else(|| anyhow!("Source track not found"))?;
        track.add_clip(clip);

        Ok(())
    }
}

/// 3. 重命名轨道命令 (:name <name>)
pub struct RenameTrackCommand {
    pub track_id: TrackId,
    pub new_name: String,
    old_name: Option<String>,
}

impl RenameTrackCommand {
    pub fn new(track_id: TrackId, new_name: impl Into<String>) -> Self {
        Self {
            track_id,
            new_name: new_name.into(),
            old_name: None,
        }
    }
}

impl EditorCommand for RenameTrackCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;
        self.old_name = Some(track.name.clone());
        track.name = self.new_name.clone();
        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let old = self
            .old_name
            .take()
            .ok_or_else(|| anyhow!("No old track name for undo"))?;
        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;
        track.name = old;
        Ok(())
    }
}

/// 4. 合并选中切片命令 (:merge)
pub struct MergeClipsCommand {
    pub track_id: TrackId,
    pub clip_ids: Vec<ClipId>,
    pub merged_clip_id: ClipId,
    original_clips: Option<Vec<Clip>>,
}

impl MergeClipsCommand {
    pub fn new(track_id: TrackId, clip_ids: Vec<ClipId>, merged_clip_id: ClipId) -> Self {
        Self {
            track_id,
            clip_ids,
            merged_clip_id,
            original_clips: None,
        }
    }
}

impl EditorCommand for MergeClipsCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        if self.clip_ids.len() < 2 {
            return Err(anyhow!("Need at least 2 clips to merge"));
        }

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        let mut targets: Vec<Clip> = track
            .clips
            .iter()
            .filter(|c| self.clip_ids.contains(&c.id))
            .cloned()
            .collect();

        if targets.len() != self.clip_ids.len() {
            return Err(anyhow!("Some clips were not found in track"));
        }

        targets.sort_by_key(|c| c.timeline_start);
        self.original_clips = Some(targets.clone());

        let first = &targets[0];
        let last = &targets[targets.len() - 1];

        let min_start = first.timeline_start;
        let max_end = last.timeline_end();
        let total_duration = max_end - min_start;

        let merged_clip = Clip {
            id: self.merged_clip_id,
            name: format!("{}_merged", first.name),
            source: first.source,
            timeline_start: min_start,
            source_in: first.source_in,
            source_out: first.source_in + total_duration,
            anchors: std::collections::HashMap::new(),
            locked: false,
            z_index: first.z_index,
            speed: 1.0,
            audio_fade_in: FrameTime::ZERO,
            audio_fade_out: FrameTime::ZERO,
            transform_rotation_deg: 0.0,
            transform_scale: [1.0, 1.0],
            transform_flip_h: false,
            transform_flip_v: false,
            transform_offset: [0.0, 0.0],
            color_grading: crate::effects::ColorGradingParams::default(),
            transition_out: None,
            text_overlay: None,
        };

        track.clips.retain(|c| !self.clip_ids.contains(&c.id));
        track.add_clip(merged_clip);

        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let originals = self
            .original_clips
            .take()
            .ok_or_else(|| anyhow!("No original clips for undo"))?;

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        track.clips.retain(|c| c.id != self.merged_clip_id);
        for orig in originals {
            track.add_clip(orig);
        }

        Ok(())
    }
}

/// 5. 选区切断并独立合并命令 (:mergecut)
pub struct MergeCutCommand {
    pub track_id: TrackId,
    pub start_time: FrameTime,
    pub end_time: FrameTime,
    pub new_clip_id: ClipId,
    original_clips: Option<Vec<Clip>>,
}

impl MergeCutCommand {
    pub fn new(
        track_id: TrackId,
        start_time: FrameTime,
        end_time: FrameTime,
        new_clip_id: ClipId,
    ) -> Self {
        Self {
            track_id,
            start_time,
            end_time,
            new_clip_id,
            original_clips: None,
        }
    }
}

impl EditorCommand for MergeCutCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        if self.start_time >= self.end_time {
            return Err(anyhow!("Invalid range for mergecut: start >= end"));
        }

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        // 备份当前轨道所有切片用于 Undo
        self.original_clips = Some(track.clips.clone());

        let mut remaining_clips = Vec::new();
        let mut first_source = None;
        let mut first_name = String::from("CutSegment");

        for clip in &track.clips {
            let clip_start = clip.timeline_start;
            let clip_end = clip.timeline_end();

            if clip_end <= self.start_time || clip_start >= self.end_time {
                // 完全在选区外部，保留
                remaining_clips.push(clip.clone());
            } else {
                // 有重叠
                if first_source.is_none() {
                    first_source = Some(clip.source);
                    first_name = clip.name.clone();
                }

                // 左侧截断保留
                if clip_start < self.start_time {
                    let mut left = clip.clone();
                    let left_len = self.start_time - clip_start;
                    left.source_out = left.source_in + left_len;
                    remaining_clips.push(left);
                }

                // 右侧截断保留
                if clip_end > self.end_time {
                    let mut right = clip.clone();
                    let right_offset = self.end_time - clip_start;
                    right.timeline_start = self.end_time;
                    right.source_in = clip.source_in + right_offset;
                    remaining_clips.push(right);
                }
            }
        }

        // 创建选区内部合并切片
        let duration = self.end_time - self.start_time;
        let new_cut_clip = Clip {
            id: self.new_clip_id,
            name: format!("{}_cut", first_name),
            source: first_source.unwrap_or(crate::timeline::AssetId(1)),
            timeline_start: self.start_time,
            source_in: FrameTime::ZERO,
            source_out: duration,
            anchors: std::collections::HashMap::new(),
            locked: false,
            z_index: 0,
            speed: 1.0,
            audio_fade_in: FrameTime::ZERO,
            audio_fade_out: FrameTime::ZERO,
            transform_rotation_deg: 0.0,
            transform_scale: [1.0, 1.0],
            transform_flip_h: false,
            transform_flip_v: false,
            transform_offset: [0.0, 0.0],
            color_grading: crate::effects::ColorGradingParams::default(),
            transition_out: None,
            text_overlay: None,
        };

        remaining_clips.push(new_cut_clip);
        track.clips = remaining_clips;
        track.sort_clips();

        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let originals = self
            .original_clips
            .take()
            .ok_or_else(|| anyhow!("No original clips for undo"))?;

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        track.clips = originals;
        track.sort_clips();

        Ok(())
    }
}

/// 6. 波纹删除命令 (Ripple Delete: 删除切片并自动将后续切片向左平移闭合间隙)
pub struct RippleDeleteClipCommand {
    pub track_id: TrackId,
    pub clip_id: ClipId,
    deleted_clip: Option<Clip>,
    shifted_clips: Option<Vec<(ClipId, FrameTime)>>, // (ClipId, 原始开始时间)
}

impl RippleDeleteClipCommand {
    pub fn new(track_id: TrackId, clip_id: ClipId) -> Self {
        Self {
            track_id,
            clip_id,
            deleted_clip: None,
            shifted_clips: None,
        }
    }
}

impl EditorCommand for RippleDeleteClipCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        let pos = track
            .clips
            .iter()
            .position(|c| c.id == self.clip_id)
            .ok_or_else(|| anyhow!("Clip not found"))?;

        let clip = track.clips.remove(pos);
        let clip_dur = clip.duration();
        let clip_end = clip.timeline_end();

        // 记录后续切片的原始开始时间，并将其整体向左平移 clip_dur
        let mut original_shifts = Vec::new();
        for c in &mut track.clips {
            if c.timeline_start >= clip_end {
                original_shifts.push((c.id, c.timeline_start));
                c.timeline_start = c.timeline_start - clip_dur;
            }
        }

        // 放入垃圾回收轨道
        state.timeline.trash_track.add_clip(clip.clone());
        self.deleted_clip = Some(clip);
        self.shifted_clips = Some(original_shifts);

        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let deleted = self
            .deleted_clip
            .take()
            .ok_or_else(|| anyhow!("No deleted clip for undo"))?;
        let shifts = self
            .shifted_clips
            .take()
            .ok_or_else(|| anyhow!("No shifted clips for undo"))?;

        // 从垃圾回收轨道移出
        state.timeline.trash_track.remove_clip(deleted.id);

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        // 恢复后续切片的原始位置
        for (cid, orig_start) in shifts {
            if let Some(c) = track.clips.iter_mut().find(|c| c.id == cid) {
                c.timeline_start = orig_start;
            }
        }

        // 放回被删除切片
        track.add_clip(deleted);
        track.sort_clips();

        Ok(())
    }
}

/// 7. 闭合轨道所有空白间隙命令 (:close_gaps / Close Gaps)
pub struct CloseGapsCommand {
    pub track_id: TrackId,
    original_positions: Option<Vec<(ClipId, FrameTime)>>,
}

impl CloseGapsCommand {
    pub fn new(track_id: TrackId) -> Self {
        Self {
            track_id,
            original_positions: None,
        }
    }
}

impl EditorCommand for CloseGapsCommand {
    fn execute(&mut self, state: &mut ProjectState) -> Result<()> {
        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        if track.clips.is_empty() {
            return Ok(());
        }

        track.clips.sort_by_key(|c| c.timeline_start);
        let mut originals = Vec::with_capacity(track.clips.len());
        let mut cur_pos = FrameTime(0);

        for clip in &mut track.clips {
            originals.push((clip.id, clip.timeline_start));
            let dur = clip.duration();
            clip.timeline_start = cur_pos;
            cur_pos = cur_pos + dur;
        }

        self.original_positions = Some(originals);
        Ok(())
    }

    fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        let originals = self
            .original_positions
            .take()
            .ok_or_else(|| anyhow!("No original positions for undo"))?;

        let track = state
            .timeline
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow!("Track not found"))?;

        for (cid, orig_start) in originals {
            if let Some(c) = track.clips.iter_mut().find(|c| c.id == cid) {
                c.timeline_start = orig_start;
            }
        }
        track.clips.sort_by_key(|c| c.timeline_start);

        Ok(())
    }
}

/// 历史栈管理
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
    pub pending_wal: VecDeque<String>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            pending_wal: VecDeque::new(),
        }
    }

    /// 执行并记录新命令
    pub fn execute(
        &mut self,
        mut cmd: Box<dyn EditorCommand>,
        state: &mut ProjectState,
    ) -> Result<()> {
        cmd.execute(state)?;
        self.redo_stack.clear();
        self.undo_stack.push(cmd);
        Ok(())
    }

    pub fn undo(&mut self, state: &mut ProjectState) -> Result<()> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state)?;
            self.redo_stack.push(cmd);
        }
        Ok(())
    }

    pub fn redo(&mut self, state: &mut ProjectState) -> Result<()> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(state)?;
            self.undo_stack.push(cmd);
        }
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, Track};

    #[test]
    fn test_split_and_undo() {
        let mut state = ProjectState::new("Test Project");
        let mut track = Track::new(TrackId(1), "V1");
        let clip = Clip::new(
            ClipId(100),
            "video.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(10_000_000), // 10s
        );
        track.add_clip(clip);
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        // 在 4s 处切割
        let split_cmd =
            SplitClipCommand::new(TrackId(1), ClipId(100), FrameTime(4_000_000), ClipId(101));
        assert!(history.execute(Box::new(split_cmd), &mut state).is_ok());

        let t = state.timeline.track_mut(TrackId(1)).unwrap();
        assert_eq!(t.clips.len(), 2);
        assert_eq!(t.clips[0].duration(), FrameTime(4_000_000));
        assert_eq!(t.clips[1].duration(), FrameTime(6_000_000));

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        let t_after = state.timeline.track_mut(TrackId(1)).unwrap();
        assert_eq!(t_after.clips.len(), 1);
        assert_eq!(t_after.clips[0].duration(), FrameTime(10_000_000));
    }

    #[test]
    fn test_delete_to_trash_and_undo() {
        let mut state = ProjectState::new("Test Project");
        let mut track = Track::new(TrackId(1), "V1");
        let clip = Clip::new(
            ClipId(100),
            "video.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(5_000_000),
        );
        track.add_clip(clip);
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        let del_cmd = DeleteClipToTrashCommand::new(TrackId(1), ClipId(100));
        assert!(history.execute(Box::new(del_cmd), &mut state).is_ok());

        assert_eq!(state.timeline.track_mut(TrackId(1)).unwrap().clips.len(), 0);
        assert_eq!(state.timeline.trash_track.clips.len(), 1);

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        assert_eq!(state.timeline.track_mut(TrackId(1)).unwrap().clips.len(), 1);
        assert_eq!(state.timeline.trash_track.clips.len(), 0);
    }

    #[test]
    fn test_rename_track_and_undo() {
        let mut state = ProjectState::new("Test Project");
        let track = Track::new(TrackId(1), "Old Track Name");
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        let rename_cmd = RenameTrackCommand::new(TrackId(1), "New Track Name");
        assert!(history.execute(Box::new(rename_cmd), &mut state).is_ok());
        assert_eq!(
            state.timeline.track_mut(TrackId(1)).unwrap().name,
            "New Track Name"
        );

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        assert_eq!(
            state.timeline.track_mut(TrackId(1)).unwrap().name,
            "Old Track Name"
        );
    }

    #[test]
    fn test_merge_clips_and_undo() {
        let mut state = ProjectState::new("Test Project");
        let mut track = Track::new(TrackId(1), "V1");
        track.add_clip(Clip::new(
            ClipId(1),
            "part1.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(3_000_000),
        ));
        track.add_clip(Clip::new(
            ClipId(2),
            "part2.mp4".into(),
            AssetId(1),
            FrameTime(3_000_000),
            FrameTime(4_000_000),
        ));
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        let merge_cmd = MergeClipsCommand::new(TrackId(1), vec![ClipId(1), ClipId(2)], ClipId(10));
        assert!(history.execute(Box::new(merge_cmd), &mut state).is_ok());

        let t = state.timeline.track_mut(TrackId(1)).unwrap();
        assert_eq!(t.clips.len(), 1);
        assert_eq!(t.clips[0].id, ClipId(10));
        assert_eq!(t.clips[0].duration(), FrameTime(7_000_000));

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        let t_restored = state.timeline.track_mut(TrackId(1)).unwrap();
        assert_eq!(t_restored.clips.len(), 2);
    }

    #[test]
    fn test_mergecut_and_undo() {
        let mut state = ProjectState::new("Test Project");
        let mut track = Track::new(TrackId(1), "V1");
        track.add_clip(Clip::new(
            ClipId(1),
            "main.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(10_000_000),
        ));
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        // 在 2s 到 6s 选区内进行 mergecut
        let cut_cmd = MergeCutCommand::new(
            TrackId(1),
            FrameTime(2_000_000),
            FrameTime(6_000_000),
            ClipId(99),
        );
        assert!(history.execute(Box::new(cut_cmd), &mut state).is_ok());

        let t = state.timeline.track_mut(TrackId(1)).unwrap();
        assert_eq!(t.clips.len(), 3);
        assert_eq!(t.clips[0].timeline_start, FrameTime(0));
        assert_eq!(t.clips[0].duration(), FrameTime(2_000_000));
        assert_eq!(t.clips[1].id, ClipId(99));
        assert_eq!(t.clips[1].timeline_start, FrameTime(2_000_000));
        assert_eq!(t.clips[1].duration(), FrameTime(4_000_000));
        assert_eq!(t.clips[2].timeline_start, FrameTime(6_000_000));
        assert_eq!(t.clips[2].duration(), FrameTime(4_000_000));

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        let t_restored = state.timeline.track_mut(TrackId(1)).unwrap();
        assert_eq!(t_restored.clips.len(), 1);
        assert_eq!(t_restored.clips[0].duration(), FrameTime(10_000_000));
    }

    #[test]
    fn test_ripple_delete_and_undo() {
        let mut state = ProjectState::new("Test Ripple Delete");
        let mut track = Track::new(TrackId(1), "V1");
        // Clip 1: 0s ~ 5s (5s 长)
        track.add_clip(Clip::new(ClipId(1), "clip1.mp4".into(), AssetId(1), FrameTime(0), FrameTime(5_000_000)));
        // Clip 2: 5s ~ 8s (3s 长)
        track.add_clip(Clip::new(ClipId(2), "clip2.mp4".into(), AssetId(2), FrameTime(5_000_000), FrameTime(3_000_000)));
        // Clip 3: 8s ~ 15s (7s 长)
        track.add_clip(Clip::new(ClipId(3), "clip3.mp4".into(), AssetId(3), FrameTime(8_000_000), FrameTime(7_000_000)));
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        // 波纹删除 Clip 2 (长度 3s)
        let cmd = RippleDeleteClipCommand::new(TrackId(1), ClipId(2));
        assert!(history.execute(Box::new(cmd), &mut state).is_ok());

        let t = state.timeline.track(TrackId(1)).unwrap();
        assert_eq!(t.clips.len(), 2);
        assert_eq!(t.clips[0].id, ClipId(1));
        assert_eq!(t.clips[0].timeline_start, FrameTime(0));
        assert_eq!(t.clips[1].id, ClipId(3));
        // Clip 3 自动左移 3 秒: 8s - 3s = 5s
        assert_eq!(t.clips[1].timeline_start, FrameTime(5_000_000));
        assert_eq!(state.timeline.trash_track.clips.len(), 1);

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        let t_undo = state.timeline.track(TrackId(1)).unwrap();
        assert_eq!(t_undo.clips.len(), 3);
        assert_eq!(t_undo.clips[2].timeline_start, FrameTime(8_000_000));
        assert_eq!(state.timeline.trash_track.clips.len(), 0);
    }

    #[test]
    fn test_close_gaps_and_undo() {
        let mut state = ProjectState::new("Test Close Gaps");
        let mut track = Track::new(TrackId(1), "V1");
        // Clip 1: 2s ~ 5s (3s 长，有 2s 前置间隙)
        track.add_clip(Clip::new(ClipId(1), "clip1.mp4".into(), AssetId(1), FrameTime(2_000_000), FrameTime(3_000_000)));
        // Clip 2: 8s ~ 10s (2s 长，有 3s 中间间隙)
        track.add_clip(Clip::new(ClipId(2), "clip2.mp4".into(), AssetId(2), FrameTime(8_000_000), FrameTime(2_000_000)));
        state.timeline.add_track(track);

        let mut history = CommandHistory::new();
        let cmd = CloseGapsCommand::new(TrackId(1));
        assert!(history.execute(Box::new(cmd), &mut state).is_ok());

        let t = state.timeline.track(TrackId(1)).unwrap();
        assert_eq!(t.clips.len(), 2);
        // Clip 1 紧凑对齐到 0s
        assert_eq!(t.clips[0].timeline_start, FrameTime(0));
        assert_eq!(t.clips[0].duration(), FrameTime(3_000_000));
        // Clip 2 紧凑对齐到 3s
        assert_eq!(t.clips[1].timeline_start, FrameTime(3_000_000));
        assert_eq!(t.clips[1].duration(), FrameTime(2_000_000));

        // 撤销
        assert!(history.undo(&mut state).is_ok());
        let t_undo = state.timeline.track(TrackId(1)).unwrap();
        assert_eq!(t_undo.clips[0].timeline_start, FrameTime(2_000_000));
        assert_eq!(t_undo.clips[1].timeline_start, FrameTime(8_000_000));
    }
}
