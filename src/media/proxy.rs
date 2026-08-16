use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::timeline::AssetId;

/// 代理文件分辨率预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProxyResolution {
    #[default]
    Low720p,
    Low360p,
    Half,
    Quarter,
}

impl ProxyResolution {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "720p" | "720" | "hd" | "default" => Some(Self::Low720p),
            "360p" | "360" | "sd" | "fast" => Some(Self::Low360p),
            "half" | "1/2" | "50%" => Some(Self::Half),
            "quarter" | "1/4" | "25%" => Some(Self::Quarter),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low720p => "720p 高清代理 (1280x720)",
            Self::Low360p => "360p 极速代理 (640x360)",
            Self::Half => "1/2 等比缩放",
            Self::Quarter => "1/4 等比缩放",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Low720p => "720p",
            Self::Low360p => "360p",
            Self::Half => "1/2",
            Self::Quarter => "1/4",
        }
    }

    pub fn calculate_target_dim(&self, src_w: u32, src_h: u32) -> (u32, u32) {
        let src_w = src_w.max(2);
        let src_h = src_h.max(2);
        match self {
            Self::Low720p => {
                let aspect = src_w as f32 / src_h as f32;
                let target_h = 720u32.min(src_h);
                let target_w = (((target_h as f32 * aspect).round() as u32) / 2) * 2;
                (target_w.max(2), target_h.max(2))
            }
            Self::Low360p => {
                let aspect = src_w as f32 / src_h as f32;
                let target_h = 360u32.min(src_h);
                let target_w = (((target_h as f32 * aspect).round() as u32) / 2) * 2;
                (target_w.max(2), target_h.max(2))
            }
            Self::Half => (((src_w / 2) / 2) * 2, ((src_h / 2) / 2) * 2),
            Self::Quarter => (((src_w / 4) / 2) * 2, ((src_h / 4) / 2) * 2),
        }
    }
}

/// 单个素材的代理状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProxyStatus {
    Missing,
    Queued,
    Generating(f32),
    Ready(PathBuf),
    Error(String),
}

impl ProxyStatus {
    pub fn label(&self) -> String {
        match self {
            Self::Missing => "未生成".into(),
            Self::Queued => "排队中".into(),
            Self::Generating(progress) => format!("生成中 {:.0}%", progress * 100.0),
            Self::Ready(path) => format!("就绪 ({})", path.display()),
            Self::Error(err) => format!("失败: {}", err),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }
}

/// 媒体资产代理记录
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyMedia {
    pub asset_id: AssetId,
    pub original_path: PathBuf,
    pub proxy_path: Option<PathBuf>,
    pub resolution: ProxyResolution,
    pub source_width: u32,
    pub source_height: u32,
    pub target_width: u32,
    pub target_height: u32,
    pub status: ProxyStatus,
    pub is_proxy_active: bool,
}

impl ProxyMedia {
    pub fn summary_line(&self) -> String {
        format!(
            "Asset #{:>3} | {} -> {}x{} ({}) | {}",
            self.asset_id.0,
            self.original_path.display(),
            self.target_width,
            self.target_height,
            self.resolution.short_label(),
            self.status.label()
        )
    }

    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }
}

/// 代理媒体全局管理器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyManager {
    pub proxies: HashMap<AssetId, ProxyMedia>,
    pub global_proxy_enabled: bool,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            proxies: HashMap::new(),
            global_proxy_enabled: false,
        }
    }

    /// 切换全局代理模式
    pub fn toggle_global_proxy(&mut self) -> bool {
        self.global_proxy_enabled = !self.global_proxy_enabled;
        self.global_proxy_enabled
    }

    pub fn set_global_proxy(&mut self, enabled: bool) -> bool {
        self.global_proxy_enabled = enabled;
        self.global_proxy_enabled
    }

    pub fn proxy_for_asset(&self, asset_id: AssetId) -> Option<&ProxyMedia> {
        self.proxies.get(&asset_id)
    }

    pub fn ready_proxy_count(&self) -> usize {
        self.proxies.values().filter(|proxy| proxy.is_ready()).count()
    }

    pub fn status_summary_lines(&self) -> Vec<String> {
        let mut items: Vec<_> = self.proxies.values().map(ProxyMedia::summary_line).collect();
        items.sort();
        items
    }

    pub fn generate_proxy_for_asset(
        &mut self,
        asset_id: AssetId,
        original_path: impl Into<PathBuf>,
        source_dims: Option<(u32, u32)>,
        resolution: ProxyResolution,
    ) -> ProxyMedia {
        let original_path = original_path.into();
        let (source_width, source_height) =
            source_dims.unwrap_or_else(|| Self::infer_source_dimensions(&original_path));
        let (target_width, target_height) =
            resolution.calculate_target_dim(source_width, source_height);
        let proxy_path = Self::proxy_output_path(&original_path, resolution);

        let media = ProxyMedia {
            asset_id,
            original_path,
            proxy_path: Some(proxy_path.clone()),
            resolution,
            source_width,
            source_height,
            target_width,
            target_height,
            status: ProxyStatus::Ready(proxy_path),
            is_proxy_active: true,
        };

        self.proxies.insert(asset_id, media.clone());
        media
    }

    pub fn activate_proxy(&mut self, asset_id: AssetId, active: bool) {
        if let Some(proxy) = self.proxies.get_mut(&asset_id) {
            proxy.is_proxy_active = active;
        }
    }

    /// 解析当前应该使用的实际媒体路径
    pub fn resolve_path<'a>(&'a self, asset_id: AssetId, fallback_path: &'a Path) -> &'a Path {
        if self.global_proxy_enabled {
            if let Some(proxy) = self.proxies.get(&asset_id) {
                if proxy.is_proxy_active {
                    if let ProxyStatus::Ready(ref path) = proxy.status {
                        return path.as_path();
                    }
                }
            }
        }
        fallback_path
    }

    /// 若代理已开启则返回推荐的低清解码尺寸，否则返回原始尺寸
    pub fn preferred_decode_dimensions(
        &self,
        asset_id: AssetId,
        fallback_dims: (u32, u32),
    ) -> (u32, u32) {
        if self.global_proxy_enabled {
            if let Some(proxy) = self.proxies.get(&asset_id) {
                if proxy.is_proxy_active && proxy.is_ready() {
                    return (proxy.target_width, proxy.target_height);
                }
            }
        }
        fallback_dims
    }

    fn infer_source_dimensions(path: &Path) -> (u32, u32) {
        let lower = path.to_string_lossy().to_lowercase();
        if lower.contains("8k") {
            (7680, 4320)
        } else if lower.contains("6k") {
            (6144, 3456)
        } else if lower.contains("4k") || lower.contains("uhd") {
            (3840, 2160)
        } else if lower.contains("2k") {
            (2560, 1440)
        } else if lower.contains("720") {
            (1280, 720)
        } else {
            (1920, 1080)
        }
    }

    fn proxy_output_path(original_path: &Path, resolution: ProxyResolution) -> PathBuf {
        let parent = original_path.parent().unwrap_or_else(|| Path::new("."));
        let stem = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("asset");
        let cache_dir = parent.join(".proxy_cache");
        cache_dir.join(format!("{}_proxy_{}.mp4", stem, resolution.short_label()))
    }

    /// 快速像素下采样生成低清 RGBA 缓冲
    pub fn downsample_rgba(
        src: &[u8],
        src_w: u32,
        src_h: u32,
        target_w: u32,
        target_h: u32,
    ) -> Vec<u8> {
        let mut out = Vec::with_capacity((target_w * target_h * 4) as usize);
        let x_ratio = src_w as f32 / target_w.max(1) as f32;
        let y_ratio = src_h as f32 / target_h.max(1) as f32;

        for y in 0..target_h {
            let src_y = ((y as f32 * y_ratio) as u32).min(src_h.saturating_sub(1));
            for x in 0..target_w {
                let src_x = ((x as f32 * x_ratio) as u32).min(src_w.saturating_sub(1));
                let src_idx = ((src_y * src_w + src_x) * 4) as usize;
                if src_idx + 4 <= src.len() {
                    out.extend_from_slice(&src[src_idx..src_idx + 4]);
                } else {
                    out.extend_from_slice(&[0, 0, 0, 255]);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_manager_generation_and_resolution() {
        let mut mgr = ProxyManager::new();
        assert!(!mgr.global_proxy_enabled);

        let orig = PathBuf::from("/videos/raw_4k_footage.mp4");
        let proxy = mgr.generate_proxy_for_asset(
            AssetId(1),
            &orig,
            Some((3840, 2160)),
            ProxyResolution::Low720p,
        );

        assert_eq!(proxy.target_height, 720);
        assert!(proxy.target_width >= 1280);
        assert_eq!(mgr.resolve_path(AssetId(1), &orig), orig.as_path());

        mgr.toggle_global_proxy();
        let resolved = mgr.resolve_path(AssetId(1), &orig);
        assert!(resolved.to_string_lossy().contains("proxy_720p.mp4"));
        assert_eq!(mgr.preferred_decode_dimensions(AssetId(1), (3840, 2160)), (1280, 720));
    }

    #[test]
    fn test_proxy_status_summary_and_inference() {
        let mut mgr = ProxyManager::new();
        mgr.generate_proxy_for_asset(
            AssetId(7),
            PathBuf::from("showcase_8k_master.mov"),
            None,
            ProxyResolution::Low360p,
        );
        let lines = mgr.status_summary_lines();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("360p"));
        assert!(lines[0].contains("Asset #  7"));
    }

    #[test]
    fn test_proxy_downsample_rgba() {
        let src_pixel = vec![255u8, 0, 0, 255];
        let src_rgba = src_pixel.repeat(16);
        let downsampled = ProxyManager::downsample_rgba(&src_rgba, 4, 4, 2, 2);
        assert_eq!(downsampled.len(), 2 * 2 * 4);
        assert_eq!(&downsampled[..4], &[255, 0, 0, 255]);
    }
}
