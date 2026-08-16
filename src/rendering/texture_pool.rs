/// 纹理规格描述（宽、高、通道格式）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format_id: u32, // 1: RGBA8, 2: BGRA8, 3: NV12 Y, 4: NV12 UV
}

/// 纹理槽位，包含是否被当前帧租借标记
#[derive(Debug)]
pub struct TextureSlot {
    pub slot_id: u64,
    pub desc: TextureDescriptor,
    pub in_use: bool,
}

/// GPU 纹理对象缓存池，避免每帧高频分配/释放显存
pub struct TexturePool {
    slots: Vec<TextureSlot>,
    next_id: u64,
    max_cached_textures: usize,
}

impl Default for TexturePool {
    fn default() -> Self {
        Self::new(32)
    }
}

impl TexturePool {
    pub fn new(max_cached_textures: usize) -> Self {
        Self {
            slots: Vec::new(),
            next_id: 1,
            max_cached_textures,
        }
    }

    /// 租借一个满足规格的纹理槽位
    pub fn acquire(&mut self, desc: TextureDescriptor) -> u64 {
        // 先寻找空闲的匹配槽位
        for slot in &mut self.slots {
            if !slot.in_use && slot.desc == desc {
                slot.in_use = true;
                return slot.slot_id;
            }
        }

        // 如果没有，创建新的槽位
        let id = self.next_id;
        self.next_id += 1;
        self.slots.push(TextureSlot {
            slot_id: id,
            desc,
            in_use: true,
        });

        id
    }

    /// 归还纹理槽位供后续复用
    pub fn release(&mut self, slot_id: u64) {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.slot_id == slot_id) {
            slot.in_use = false;
        }
    }

    /// 获取当前池中分配的总纹理数与正处于使用状态的纹理数
    pub fn stats(&self) -> (usize, usize) {
        let in_use_count = self.slots.iter().filter(|s| s.in_use).count();
        (self.slots.len(), in_use_count)
    }

    /// 垃圾回收修剪超出容量的空闲纹理
    pub fn trim_excess(&mut self) {
        if self.slots.len() > self.max_cached_textures {
            self.slots.retain(|s| s.in_use);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_pool_acquire_and_reuse() {
        let mut pool = TexturePool::new(10);
        let desc = TextureDescriptor {
            width: 1920,
            height: 1080,
            format_id: 1,
        };

        // 第一次申请
        let tex1 = pool.acquire(desc);
        assert_eq!(pool.stats(), (1, 1));

        // 释放
        pool.release(tex1);
        assert_eq!(pool.stats(), (1, 0));

        // 再次申请相同尺寸，应该复用同一个 ID
        let tex2 = pool.acquire(desc);
        assert_eq!(tex1, tex2);
        assert_eq!(pool.stats(), (1, 1));
    }
}
