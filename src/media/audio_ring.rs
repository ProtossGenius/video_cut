use rtrb::{Consumer, Producer, RingBuffer};

/// 音频无锁环形队列封装（用于实时音频回调与重采样线程之间的单生产者-单消费者通信）
pub struct AudioRingChannel {
    pub producer: Producer<f32>,
    pub consumer: Consumer<f32>,
}

impl AudioRingChannel {
    pub fn new(capacity: usize) -> Self {
        let (producer, consumer) = RingBuffer::new(capacity);
        Self { producer, consumer }
    }

    /// 生产者写入音频样本块（非阻塞）
    pub fn push_chunk(&mut self, samples: &[f32]) -> usize {
        let mut written = 0;
        for &s in samples {
            if self.producer.push(s).is_ok() {
                written += 1;
            } else {
                break; // 队列满
            }
        }
        written
    }

    /// 消费者在 CPAL 实时声卡中断线程中拉取音频（绝不分配内存，绝不加锁）
    pub fn pop_into_buffer(consumer: &mut Consumer<f32>, output: &mut [f32]) -> usize {
        let mut read = 0;
        for out in output.iter_mut() {
            if let Ok(sample) = consumer.pop() {
                *out = sample;
                read += 1;
            } else {
                *out = 0.0; // 缓冲不足补静音
            }
        }
        read
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_ring_buffer_push_pop() {
        let mut channel = AudioRingChannel::new(1024);
        let samples = [0.1f32, 0.2, 0.3, 0.4, 0.5];
        let written = channel.push_chunk(&samples);
        assert_eq!(written, 5);

        let mut output = [0.0f32; 8];
        let read = AudioRingChannel::pop_into_buffer(&mut channel.consumer, &mut output);
        assert_eq!(read, 5);
        assert_eq!(&output[..5], &samples);
        assert_eq!(&output[5..], &[0.0, 0.0, 0.0]);
    }
}
