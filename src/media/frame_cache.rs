use crate::timeline::{AssetId, FrameTime};
use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

/// 解码后的视频 RGBA 帧数据
#[derive(Debug, Clone)]
pub struct DecodedVideoFrame {
    pub width: u32,
    pub height: u32,
    pub pts: FrameTime,
    pub data: Arc<Vec<u8>>,
}

/// 视频帧缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameKey {
    pub asset_id: AssetId,
    pub pts: FrameTime,
}

/// 内存视频帧缓存池
pub struct VideoFrameCache {
    cache: Cache<FrameKey, DecodedVideoFrame>,
}

impl Default for VideoFrameCache {
    fn default() -> Self {
        Self::new(120) // 默认缓存 120 帧 (约 2 秒 60fps 帧)
    }
}

impl VideoFrameCache {
    pub fn new(capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_idle(Duration::from_secs(300))
            .build();
        Self { cache }
    }

    pub fn get(&self, key: &FrameKey) -> Option<DecodedVideoFrame> {
        self.cache.get(key)
    }

    pub fn insert(&self, key: FrameKey, frame: DecodedVideoFrame) {
        self.cache.insert(key, frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_frame_cache() {
        let cache = VideoFrameCache::new(5);
        let key = FrameKey {
            asset_id: AssetId(10),
            pts: FrameTime(33_333),
        };
        let frame = DecodedVideoFrame {
            width: 1920,
            height: 1080,
            pts: FrameTime(33_333),
            data: Arc::new(vec![0u8; 1920 * 1080 * 4]),
        };

        cache.insert(key.clone(), frame);
        let hit = cache.get(&key);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().width, 1920);
    }
}
