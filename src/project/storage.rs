use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::project::ProjectState;
use crate::timeline::FrameTime;

/// 缓存元数据与波形采样点（用于二进制 .cache 文件存储）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectBinaryCache {
    pub project_name: String,
    pub version: u32,
    pub waveform_samples: Vec<f32>,
    pub thumbnail_timestamps: Vec<FrameTime>,
}

impl Default for ProjectBinaryCache {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            version: 1,
            waveform_samples: Vec::new(),
            thumbnail_timestamps: Vec::new(),
        }
    }
}

/// WAL 日志条目，记录每次命令操作以支持崩溃恢复
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalEntry {
    pub timestamp_ms: u64,
    pub command_name: String,
    pub payload_json: String,
}

/// 项目持久化存储管理器
pub struct ProjectStorage;

impl ProjectStorage {
    /// 以安全原子覆盖方式保存项目至 .vcut 文件
    /// 步骤：先写入 .vcut.temp 临时文件，刷写成功后原子重命名覆盖目标文件
    pub fn save_project_atomic(path: &Path, state: &ProjectState) -> Result<()> {
        let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .with_context(|| format!("Failed to create directory: {:?}", parent_dir))?;
        }

        let temp_file_name = format!(
            "{}.temp.{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project"),
            std::process::id()
        );
        let temp_path = parent_dir.join(temp_file_name);

        // 1. 序列化为格式化的 JSON
        let json_data = serde_json::to_string_pretty(state)
            .context("Failed to serialize ProjectState to JSON")?;

        // 2. 写入临时文件并同步刷盘
        {
            let mut file = File::create(&temp_path)
                .with_context(|| format!("Failed to create temp file: {:?}", temp_path))?;
            file.write_all(json_data.as_bytes())
                .context("Failed to write JSON data to temp file")?;
            file.sync_all()
                .context("Failed to sync temp file to disk")?;
        }

        // 3. 原子重命名覆盖目标文件
        fs::rename(&temp_path, path).with_context(|| {
            format!("Failed to atomically rename {:?} to {:?}", temp_path, path)
        })?;

        Ok(())
    }

    /// 从 .vcut JSON 文件加载项目
    pub fn load_project(path: &Path) -> Result<ProjectState> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read project file: {:?}", path))?;
        let state: ProjectState = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse project JSON from: {:?}", path))?;
        Ok(state)
    }
}

/// 快速二进制缓存文件 (.cache) 存储管理器，使用 MessagePack (rmp-serde)
pub struct CacheStorage;

impl CacheStorage {
    pub fn save_cache(path: &Path, cache: &ProjectBinaryCache) -> Result<()> {
        let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }

        let bytes = rmp_serde::to_vec(cache).context("Failed to serialize cache to MessagePack")?;

        let temp_path = path.with_extension("cache.temp");
        {
            let mut file = File::create(&temp_path)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        fs::rename(&temp_path, path)?;
        Ok(())
    }

    pub fn load_cache(path: &Path) -> Result<ProjectBinaryCache> {
        let bytes =
            fs::read(path).with_context(|| format!("Failed to read cache file: {:?}", path))?;
        let cache: ProjectBinaryCache = rmp_serde::from_slice(&bytes)
            .with_context(|| format!("Failed to deserialize cache from: {:?}", path))?;
        Ok(cache)
    }
}

/// WAL 日志写入与恢复器
pub struct WalLog {
    wal_path: PathBuf,
}

impl WalLog {
    pub fn new(wal_path: impl Into<PathBuf>) -> Self {
        Self {
            wal_path: wal_path.into(),
        }
    }

    /// 追加写入一条命令日志
    pub fn append(&self, command_name: &str, payload_json: &str) -> Result<()> {
        if let Some(parent) = self.wal_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let entry = WalEntry {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            command_name: command_name.to_string(),
            payload_json: payload_json.to_string(),
        };

        let mut line = serde_json::to_string(&entry)?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .with_context(|| format!("Failed to open WAL file: {:?}", self.wal_path))?;

        file.write_all(line.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// 读取并回放所有 WAL 日志条目
    pub fn read_entries(&self) -> Result<Vec<WalEntry>> {
        if !self.wal_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.wal_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (idx, line_res) in reader.lines().enumerate() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: WalEntry = serde_json::from_str(trimmed)
                .with_context(|| format!("Failed to parse WAL entry at line {}", idx + 1))?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// 清空或删除 WAL 文件（在全量保存落盘后调用）
    pub fn clear(&self) -> Result<()> {
        if self.wal_path.exists() {
            fs::remove_file(&self.wal_path)?;
        }
        Ok(())
    }
}

/// 自包含工程归档包清单元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectBundleManifest {
    pub project_name: String,
    pub version: u32,
    pub created_at_ms: u64,
    pub media_assets: Vec<String>,
    pub scripts: Vec<String>,
}

/// 自包含工程打包与解包归档器
pub struct ProjectBundle;

impl ProjectBundle {
    /// 将工程和引用的外部素材统一打包为自包含目录
    pub fn pack_bundle(
        bundle_dir: &Path,
        state: &ProjectState,
        asset_paths: &[PathBuf],
        script_paths: &[PathBuf],
    ) -> Result<ProjectBundleManifest> {
        if !bundle_dir.exists() {
            fs::create_dir_all(bundle_dir)
                .with_context(|| format!("Failed to create bundle dir: {:?}", bundle_dir))?;
        }

        let assets_dir = bundle_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        let scripts_dir = bundle_dir.join("scripts");
        fs::create_dir_all(&scripts_dir)?;

        let mut copied_assets = Vec::new();
        for src in asset_paths {
            if src.exists() && src.is_file() {
                if let Some(file_name) = src.file_name() {
                    let dest = assets_dir.join(file_name);
                    fs::copy(src, &dest)?;
                    copied_assets.push(file_name.to_string_lossy().to_string());
                }
            }
        }

        let mut copied_scripts = Vec::new();
        for src in script_paths {
            if src.exists() && src.is_file() {
                if let Some(file_name) = src.file_name() {
                    let dest = scripts_dir.join(file_name);
                    fs::copy(src, &dest)?;
                    copied_scripts.push(file_name.to_string_lossy().to_string());
                }
            }
        }

        // 写入核心工程文件 project.vcut
        let proj_file = bundle_dir.join("project.vcut");
        ProjectStorage::save_project_atomic(&proj_file, state)?;

        // 写入 manifest.json
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let manifest = ProjectBundleManifest {
            project_name: state.name.clone(),
            version: 1,
            created_at_ms: now_ms,
            media_assets: copied_assets,
            scripts: copied_scripts,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(bundle_dir.join("manifest.json"), manifest_json)?;

        Ok(manifest)
    }

    /// 从自包含归档包目录解包并载入 ProjectState
    pub fn unpack_bundle(bundle_dir: &Path) -> Result<ProjectState> {
        let manifest_file = bundle_dir.join("manifest.json");
        if !manifest_file.exists() {
            anyhow::bail!("Invalid project bundle: missing manifest.json");
        }

        let proj_file = bundle_dir.join("project.vcut");
        if !proj_file.exists() {
            anyhow::bail!("Invalid project bundle: missing project.vcut");
        }

        ProjectStorage::load_project(&proj_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{AssetId, Clip, ClipId, Track, TrackId};
    use std::env;

    #[test]
    fn test_project_atomic_save_and_load() {
        let tmp_dir = env::temp_dir().join(format!("vcut_test_{}", std::process::id()));
        let proj_path = tmp_dir.join("my_awesome_project.vcut");

        let mut state = ProjectState::new("My Awesome Project");
        let mut track = Track::new(TrackId(1), "V1");
        track.add_clip(Clip::new(
            ClipId(10),
            "intro.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(5_000_000),
        ));
        state.timeline.add_track(track);

        // 1. 保存
        assert!(ProjectStorage::save_project_atomic(&proj_path, &state).is_ok());
        assert!(proj_path.exists());

        // 2. 加载
        let loaded = ProjectStorage::load_project(&proj_path).expect("Failed to load project");
        assert_eq!(loaded.name, "My Awesome Project");
        assert_eq!(loaded.timeline.tracks.len(), 1);
        assert_eq!(loaded.timeline.tracks[0].clips.len(), 1);
        assert_eq!(loaded.timeline.tracks[0].clips[0].name, "intro.mp4");

        // 清理
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_cache_binary_serialization() {
        let tmp_dir = env::temp_dir().join(format!("vcut_cache_test_{}", std::process::id()));
        let cache_path = tmp_dir.join("project.cache");

        let cache = ProjectBinaryCache {
            project_name: "Test Cache".into(),
            version: 1,
            waveform_samples: vec![0.1, 0.45, 0.9, 0.32, 0.05],
            thumbnail_timestamps: vec![FrameTime(0), FrameTime(1_000_000), FrameTime(2_000_000)],
        };

        assert!(CacheStorage::save_cache(&cache_path, &cache).is_ok());
        assert!(cache_path.exists());

        let loaded = CacheStorage::load_cache(&cache_path).expect("Failed to load cache");
        assert_eq!(loaded, cache);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_wal_append_read_and_clear() {
        let tmp_dir = env::temp_dir().join(format!("vcut_wal_test_{}", std::process::id()));
        let wal_path = tmp_dir.join("project.wal");

        let wal = WalLog::new(&wal_path);

        assert!(wal
            .append("split", r#"{"track_id": 1, "clip_id": 10}"#)
            .is_ok());
        assert!(wal
            .append("delete", r#"{"track_id": 1, "clip_id": 11}"#)
            .is_ok());

        let entries = wal.read_entries().expect("Failed to read WAL");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command_name, "split");
        assert_eq!(entries[1].command_name, "delete");

        assert!(wal.clear().is_ok());
        assert!(!wal_path.exists());

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_project_bundle_pack_and_unpack() {
        let tmp_dir = env::temp_dir().join(format!("vcut_bundle_test_{}", std::process::id()));
        let bundle_dir = tmp_dir.join("my_bundle.vcutpkg");

        let dummy_asset = tmp_dir.join("asset1.mp4");
        let dummy_script = tmp_dir.join("auto.lua");
        let _ = fs::create_dir_all(&tmp_dir);
        let _ = fs::write(&dummy_asset, b"video_data_12345");
        let _ = fs::write(&dummy_script, b"-- lua script");

        let mut state = ProjectState::new("Bundle Proj");
        let mut track = Track::new(TrackId(1), "V1");
        track.add_clip(Clip::new(
            ClipId(1),
            "asset1.mp4".into(),
            AssetId(1),
            FrameTime(0),
            FrameTime(3_000_000),
        ));
        state.timeline.add_track(track);

        // 1. 打包
        let manifest = ProjectBundle::pack_bundle(
            &bundle_dir,
            &state,
            &[dummy_asset],
            &[dummy_script],
        ).expect("Failed to pack bundle");

        assert_eq!(manifest.project_name, "Bundle Proj");
        assert_eq!(manifest.media_assets, vec!["asset1.mp4"]);
        assert_eq!(manifest.scripts, vec!["auto.lua"]);
        assert!(bundle_dir.join("manifest.json").exists());
        assert!(bundle_dir.join("assets/asset1.mp4").exists());
        assert!(bundle_dir.join("scripts/auto.lua").exists());

        // 2. 解包
        let loaded = ProjectBundle::unpack_bundle(&bundle_dir).expect("Failed to unpack bundle");
        assert_eq!(loaded.name, "Bundle Proj");
        assert_eq!(loaded.timeline.tracks[0].clips[0].name, "asset1.mp4");

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
