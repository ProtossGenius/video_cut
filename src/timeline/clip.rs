use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::anchor::AnchorPoint;

/// 唯一标识媒体资产
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

/// 唯一标识一个切片
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClipId(pub u64);

/// 时间单位（帧）。为了避免浮点数精度问题，建议底层使用帧或采样点数计算。
/// 考虑到音频主时钟，这里可以作为一个相对时钟的微秒或帧数。
/// 这里暂时使用毫秒(ms)或者微秒(us)，为了适配高精度，使用微秒 i64。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FrameTime(pub i64);

impl FrameTime {
    pub const ZERO: FrameTime = FrameTime(0);
}

/// 切片数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: ClipId,
    pub name: String,
    
    /// 素材在原始文件中的起始时间
    pub source_in: FrameTime,
    /// 素材在原始文件中的结束时间
    pub source_out: FrameTime,
    /// 素材在时间线上的摆放起始时间
    pub timeline_start: FrameTime,
    
    /// 引用的源资产
    pub source: AssetId,
    
    /// 局部锚点
    pub anchors: HashMap<String, AnchorPoint>,
    
    // TODO: 特效属性列表 (暂时略过具体类型，留作扩展)
    // pub properties: Vec<Box<dyn EffectProperty>>,
}

impl Clip {
    pub fn new(id: ClipId, name: String, source: AssetId, timeline_start: FrameTime, duration: FrameTime) -> Self {
        Self {
            id,
            name,
            source,
            source_in: FrameTime::ZERO,
            source_out: duration,
            timeline_start,
            anchors: HashMap::new(),
        }
    }
    
    /// 计算切片在时间线上的时长
    pub fn duration(&self) -> FrameTime {
        // 注意：后续加入了加速/减速特性后，这里需要乘以速度倍数
        FrameTime(self.source_out.0 - self.source_in.0)
    }
    
    /// 计算切片在时间线上的结束时间
    pub fn timeline_end(&self) -> FrameTime {
        FrameTime(self.timeline_start.0 + self.duration().0)
    }
}
