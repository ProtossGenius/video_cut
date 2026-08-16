use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::media::frame_cache::{DecodedVideoFrame, FrameKey, VideoFrameCache};
use crate::timeline::{AssetId, FrameTime};

/// 视频解码请求
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeRequest {
    pub asset_id: AssetId,
    pub target_pts: FrameTime,
    pub width: u32,
    pub height: u32,
}

/// 视频解码响应
#[derive(Debug, Clone)]
pub struct DecodeResponse {
    pub asset_id: AssetId,
    pub pts: FrameTime,
    pub frame: DecodedVideoFrame,
}

/// 多线程视频解码池
pub struct VideoDecoderPool {
    request_tx: Sender<DecodeRequest>,
    response_rx: Receiver<DecodeResponse>,
    is_running: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
    cache: VideoFrameCache,
}

impl VideoDecoderPool {
    pub fn new(num_threads: usize, queue_capacity: usize, cache_capacity: usize) -> Self {
        let (req_tx, req_rx) = bounded::<DecodeRequest>(queue_capacity);
        let (resp_tx, resp_rx) = bounded::<DecodeResponse>(queue_capacity);
        let is_running = Arc::new(AtomicBool::new(true));

        let mut workers = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let rx = req_rx.clone();
            let tx = resp_tx.clone();
            let running = Arc::clone(&is_running);

            let handle = thread::spawn(move || {
                while running.load(Ordering::Relaxed) {
                    if let Ok(req) = rx.recv_timeout(std::time::Duration::from_millis(50)) {
                        // 模拟/执行精确解码目标 PTS
                        let rgba_data = vec![200u8; (req.width * req.height * 4) as usize];
                        let frame = DecodedVideoFrame {
                            pts: req.target_pts,
                            width: req.width,
                            height: req.height,
                            data: Arc::new(rgba_data),
                        };

                        let _ = tx.send(DecodeResponse {
                            asset_id: req.asset_id,
                            pts: req.target_pts,
                            frame,
                        });
                    }
                }
            });

            workers.push(handle);
        }

        Self {
            request_tx: req_tx,
            response_rx: resp_rx,
            is_running,
            workers,
            cache: VideoFrameCache::new(cache_capacity as u64),
        }
    }

    /// 请求解码某一时刻的视频帧（先查缓存，若无则异步推入解码队列）
    pub fn request_frame(
        &mut self,
        asset_id: AssetId,
        pts: FrameTime,
        width: u32,
        height: u32,
    ) -> Option<DecodedVideoFrame> {
        let key = FrameKey { asset_id, pts };

        if let Some(cached) = self.cache.get(&key) {
            return Some(cached);
        }

        // 尝试非阻塞发送解码请求
        let req = DecodeRequest {
            asset_id,
            target_pts: pts,
            width,
            height,
        };
        let _ = self.request_tx.try_send(req);

        None
    }

    /// 轮询并收取已完成的解码帧存入缓存
    pub fn poll_completed(&mut self) -> usize {
        let mut count = 0;
        while let Ok(resp) = self.response_rx.try_recv() {
            let key = FrameKey {
                asset_id: resp.asset_id,
                pts: resp.pts,
            };
            self.cache.insert(key, resp.frame);
            count += 1;
        }
        count
    }

    /// 获取缓存引用
    pub fn cache(&self) -> &VideoFrameCache {
        &self.cache
    }
}

impl Drop for VideoDecoderPool {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_pool_dispatch_and_cache() {
        let mut pool = VideoDecoderPool::new(2, 10, 50);
        let asset = AssetId(1);
        let pts = FrameTime(2_000_000);

        // 首次获取未命中
        let frame1 = pool.request_frame(asset, pts, 640, 480);
        assert!(frame1.is_none());

        // 等待后台解码线程完成
        thread::sleep(std::time::Duration::from_millis(100));

        // 收集已完成帧
        let polled = pool.poll_completed();
        assert!(polled > 0);

        // 再次获取应直接命中缓存
        let frame2 = pool.request_frame(asset, pts, 640, 480);
        assert!(frame2.is_some());
        assert_eq!(frame2.unwrap().pts, pts);
    }
}
