use crate::project::ProjectState;
use crate::timeline::FrameTime;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 导出编码预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExportPreset {
    #[default]
    H264Mp4,
    HevcMp4,
    ProResMov,
}

impl ExportPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::H264Mp4 => "H.264 / MP4 (Web 兼容性推荐)",
            Self::HevcMp4 => "H.265 / HEVC (高压缩比，体积节省 40%)",
            Self::ProResMov => "Apple ProRes 422 / MOV (母带无损级)",
        }
    }

    pub fn container_extension(&self) -> &'static str {
        match self {
            Self::H264Mp4 | Self::HevcMp4 => "mp4",
            Self::ProResMov => "mov",
        }
    }
}

/// 导出任务实时进度与状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportTaskState {
    pub is_exporting: bool,
    pub output_path: String,
    pub preset: ExportPreset,
    pub progress: f32, // 0.0 ~ 1.0
    pub current_frame: u64,
    pub total_frames: u64,
    pub fps: f32,
    pub eta_seconds: u32,
    pub is_completed: bool,
    pub error_message: Option<String>,
}

impl Default for ExportTaskState {
    fn default() -> Self {
        Self {
            is_exporting: false,
            output_path: "output.mp4".into(),
            preset: ExportPreset::H264Mp4,
            progress: 0.0,
            current_frame: 0,
            total_frames: 1000,
            fps: 120.0,
            eta_seconds: 0,
            is_completed: false,
            error_message: None,
        }
    }
}

/// 单个分片渲染状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreRenderStatus {
    Unrendered,
    Rendering,
    Ready(PathBuf),
}

/// 时间轴分片
#[derive(Debug, Clone)]
pub struct TimeChunk {
    pub id: usize,
    pub start_time: FrameTime,
    pub end_time: FrameTime,
    /// 该分片最后被修改的逻辑/物理时间戳 (毫秒)
    pub last_modified_ms: u64,
    /// 预渲染生成的文件路径
    pub prerendered_file: Option<PathBuf>,
    /// 是否需要重新渲染（发生改动）
    pub is_dirty: bool,
}

impl TimeChunk {
    pub fn duration(&self) -> FrameTime {
        self.end_time - self.start_time
    }
}

/// 后台分片预渲染调度器
pub struct ChunkPreRenderer {
    pub chunk_duration: FrameTime,
    /// 空闲判定阈值（毫秒），例如 3 分钟 (180,000 ms)
    pub idle_threshold_ms: u64,
    pub cache_dir: PathBuf,
    pub chunks: Vec<TimeChunk>,
}

impl ChunkPreRenderer {
    pub fn new(cache_dir: PathBuf, chunk_duration_secs: f64, idle_threshold_ms: u64) -> Self {
        Self {
            chunk_duration: FrameTime::from_seconds(chunk_duration_secs),
            idle_threshold_ms,
            cache_dir,
            chunks: Vec::new(),
        }
    }

    /// 同步并根据时间线总长度重构分片列表
    pub fn sync_timeline_chunks(&mut self, total_duration: FrameTime, now_ms: u64) {
        let chunk_dur_us = self.chunk_duration.0.max(1_000_000);
        let num_chunks = ((total_duration.0 + chunk_dur_us - 1) / chunk_dur_us).max(1) as usize;

        while self.chunks.len() < num_chunks {
            let id = self.chunks.len();
            let start = FrameTime(id as i64 * chunk_dur_us);
            let end = FrameTime(
                ((id + 1) as i64 * chunk_dur_us).min(total_duration.0.max(chunk_dur_us)),
            );
            self.chunks.push(TimeChunk {
                id,
                start_time: start,
                end_time: end,
                last_modified_ms: now_ms,
                prerendered_file: None,
                is_dirty: true,
            });
        }
    }

    /// 标记特定时间范围内的分片为脏（发生编辑/切割/移动）
    pub fn mark_range_dirty(&mut self, range: (FrameTime, FrameTime), now_ms: u64) {
        for chunk in &mut self.chunks {
            if range.0 < chunk.end_time && range.1 > chunk.start_time {
                chunk.is_dirty = true;
                chunk.last_modified_ms = now_ms;
                chunk.prerendered_file = None;
            }
        }
    }

    /// 轮询空闲分片并触发预渲染
    pub fn poll_idle_prerender<F>(&mut self, now_ms: u64, render_chunk_fn: F) -> Vec<usize>
    where
        F: Fn(&TimeChunk, &Path) -> Result<()>,
    {
        let mut prerendered_ids = Vec::new();
        let _ = std::fs::create_dir_all(&self.cache_dir);

        for chunk in &mut self.chunks {
            if (chunk.is_dirty || chunk.prerendered_file.is_none())
                && (now_ms >= chunk.last_modified_ms + self.idle_threshold_ms)
            {
                let chunk_path = self.cache_dir.join(format!(
                    "chunk_{:04}_{}_{}.cache",
                    chunk.id, chunk.start_time.0, chunk.end_time.0
                ));
                if render_chunk_fn(chunk, &chunk_path).is_ok() {
                    chunk.prerendered_file = Some(chunk_path);
                    chunk.is_dirty = false;
                    prerendered_ids.push(chunk.id);
                }
            }
        }

        prerendered_ids
    }
}

/// 导出分片动作
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkExportAction {
    /// 直接复用已预渲染的分片流（极速复制无需重编码）
    StreamCopy(PathBuf),
    /// 现场实时合成并编码
    RenderAndEncode(FrameTime, FrameTime),
}

/// 导出统计报告
#[derive(Debug, Clone)]
pub struct SmartExportReport {
    pub total_chunks: usize,
    pub reused_prerendered_chunks: usize,
    pub rendered_on_demand_chunks: usize,
    pub total_duration: FrameTime,
    pub output_path: PathBuf,
}

/// 智能无重编码极速拼接导出引擎
pub struct SmartConcatExporter;

impl SmartConcatExporter {
    /// 计算工程导出的分片动作计划
    pub fn plan_export(
        _project: &ProjectState,
        prerenderer: &ChunkPreRenderer,
    ) -> Vec<ChunkExportAction> {
        let mut actions = Vec::new();

        for chunk in &prerenderer.chunks {
            if !chunk.is_dirty && chunk.prerendered_file.is_some() {
                if let Some(ref path) = chunk.prerendered_file {
                    actions.push(ChunkExportAction::StreamCopy(path.clone()));
                    continue;
                }
            }
            actions.push(ChunkExportAction::RenderAndEncode(
                chunk.start_time,
                chunk.end_time,
            ));
        }

        actions
    }

    /// 执行智能拼接导出
    pub fn execute_export<F>(
        project: &ProjectState,
        prerenderer: &mut ChunkPreRenderer,
        output_path: &Path,
        render_on_demand_fn: F,
    ) -> Result<SmartExportReport>
    where
        F: Fn(FrameTime, FrameTime, &Path) -> Result<()>,
    {
        let total_duration = project.timeline.duration;
        let plan = Self::plan_export(project, prerenderer);

        let mut reused_count = 0;
        let mut rendered_count = 0;
        let mut chunk_files = Vec::new();

        let temp_dir = prerenderer.cache_dir.join("export_temp");
        let _ = std::fs::create_dir_all(&temp_dir);

        for (idx, action) in plan.into_iter().enumerate() {
            match action {
                ChunkExportAction::StreamCopy(path) => {
                    reused_count += 1;
                    chunk_files.push(path);
                }
                ChunkExportAction::RenderAndEncode(start, end) => {
                    rendered_count += 1;
                    let target_chunk = temp_dir.join(format!("ondemand_{:04}.chunk", idx));
                    render_on_demand_fn(start, end, &target_chunk)?;
                    chunk_files.push(target_chunk);
                }
            }
        }

        // 最终拼接 (Fast Concat)
        let mut combined_data = Vec::new();
        for file in &chunk_files {
            if let Ok(data) = std::fs::read(file) {
                combined_data.extend(data);
            }
        }
        std::fs::write(output_path, combined_data)?;

        Ok(SmartExportReport {
            total_chunks: chunk_files.len(),
            reused_prerendered_chunks: reused_count,
            rendered_on_demand_chunks: rendered_count,
            total_duration,
            output_path: output_path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, Clip, ClipId, Track, TrackId};

    #[test]
    fn test_chunk_prerender_and_smart_concat_export() {
        let temp_dir = std::env::temp_dir().join("vcut_smart_export_test");
        let _ = std::fs::create_dir_all(&temp_dir);

        // 创建测试项目 (总长 20 秒)
        let mut project = ProjectState::new("Smart Export Test");
        let mut track = Track::new(TrackId(1), "V1");
        track.add_clip(Clip::new(
            ClipId(1),
            "video1.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(20_000_000), // 20s
        ));
        project.timeline.add_track(track);

        // 每分片 5 秒，空闲 180,000 ms (3 分钟) 触发
        let mut prerenderer = ChunkPreRenderer::new(temp_dir.clone(), 5.0, 180_000);
        prerenderer.sync_timeline_chunks(FrameTime(20_000_000), 1_000_000);

        assert_eq!(prerenderer.chunks.len(), 4);

        // 1. 时间还没到 3 分钟，不触发预渲染
        let ready1 = prerenderer.poll_idle_prerender(1_050_000, |chunk, path| {
            std::fs::write(path, format!("CHUNK_{}", chunk.id))?;
            Ok(())
        });
        assert_eq!(ready1.len(), 0);

        // 2. 时间过去 4 分钟 (240,000 ms)，前 4 个分片均超过 3 分钟未改动，触发预渲染
        let ready2 = prerenderer.poll_idle_prerender(1_240_000, |chunk, path| {
            std::fs::write(path, format!("CHUNK_{}", chunk.id))?;
            Ok(())
        });
        assert_eq!(ready2.len(), 4);

        // 3. 用户在第 12 秒处做了剪辑 (污染 Chunk 2)
        prerenderer.mark_range_dirty((FrameTime(11_000_000), FrameTime(13_000_000)), 1_250_000);
        assert!(prerenderer.chunks[2].is_dirty);
        assert!(!prerenderer.chunks[0].is_dirty);

        // 4. 触发最终导出
        let output_file = temp_dir.join("final_export.mp4");
        let report = SmartConcatExporter::execute_export(
            &project,
            &mut prerenderer,
            &output_file,
            |start, end, path| {
                std::fs::write(path, format!("ONDEMAND_{}_{}", start.0, end.0))?;
                Ok(())
            },
        )
        .unwrap();

        // 验证：3 个分片直接复用预渲染 (StreamCopy)，仅 1 个被污染的分片现场渲染
        assert_eq!(report.total_chunks, 4);
        assert_eq!(report.reused_prerendered_chunks, 3);
        assert_eq!(report.rendered_on_demand_chunks, 1);
        assert!(output_file.exists());

        let output_data = std::fs::read_to_string(&output_file).unwrap();
        assert!(output_data.contains("CHUNK_0"));
        assert!(output_data.contains("CHUNK_1"));
        assert!(output_data.contains("ONDEMAND_10000000_15000000"));
        assert!(output_data.contains("CHUNK_3"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
