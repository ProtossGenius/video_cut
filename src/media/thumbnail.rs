use crate::timeline::{AssetId, FrameTime};
use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

/// 缩略图图像数据（RGBA 字节）
#[derive(Clone)]
pub struct ThumbnailImage {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: Arc<Vec<u8>>,
}

/// 缩略图缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThumbnailKey {
    pub asset_id: AssetId,
    pub time_offset: FrameTime,
    pub width: u32,
    pub height: u32,
}

/// 线程安全的内存缩略图缓存池
pub struct ThumbnailCache {
    cache: Cache<ThumbnailKey, ThumbnailImage>,
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new(500) // 默认缓存 500 张缩略图
    }
}

impl ThumbnailCache {
    pub fn new(max_entries: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_idle(Duration::from_secs(600))
            .build();
        Self { cache }
    }

    /// 查询缓存中的缩略图
    pub fn get(&self, key: &ThumbnailKey) -> Option<ThumbnailImage> {
        self.cache.get(key)
    }

    /// 插入缩略图到缓存中
    pub fn insert(&self, key: ThumbnailKey, image: ThumbnailImage) {
        self.cache.insert(key, image);
    }

    /// 生成纯色/模拟缩略图（用于测试或占位）
    pub fn create_mock_thumbnail(width: u32, height: u32, color_rgba: [u8; 4]) -> ThumbnailImage {
        let total_pixels = (width * height) as usize;
        let mut bytes = Vec::with_capacity(total_pixels * 4);
        for _ in 0..total_pixels {
            bytes.extend_from_slice(&color_rgba);
        }
        ThumbnailImage {
            width,
            height,
            rgba_bytes: Arc::new(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_cache_insert_get() {
        let cache = ThumbnailCache::new(10);
        let key = ThumbnailKey {
            asset_id: AssetId(1),
            time_offset: FrameTime(0),
            width: 128,
            height: 72,
        };

        let mock_img = ThumbnailCache::create_mock_thumbnail(128, 72, [255, 0, 0, 255]);
        cache.insert(key.clone(), mock_img);

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        let img = retrieved.unwrap();
        assert_eq!(img.width, 128);
        assert_eq!(img.height, 72);
        assert_eq!(img.rgba_bytes.len(), 128 * 72 * 4);
    }
}
