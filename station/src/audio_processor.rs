use crate::microphone::SAMPLE_RATE;
use ringbuf::traits::*;

pub const SNIPPET_DURATION_SECS: usize = 3;
pub const SNIPPET_SAMPLES: usize = SAMPLE_RATE as usize * SNIPPET_DURATION_SECS;

type AudioSnippet = Vec<f32>;

pub struct AudioCollector {
    target_size: usize,
}

impl AudioCollector {
    pub fn new(target_size: usize) -> Self {
        Self { target_size }
    }
    /// Pulls samples from the consumer until an `AudioSnippet` of target size has been fully collected.
    /// Then returns the `AudioSnippet`.
    pub fn collect<C: Consumer<Item = f32>>(&mut self, consumer: &mut C) -> AudioSnippet {
        let mut buffer = Vec::with_capacity(self.target_size);
        loop {
            let remaining = self.target_size - buffer.len();
            let available = consumer.occupied_len();
            let to_read = available.min(remaining);

            if to_read > 0 {
                let start_idx = buffer.len();
                buffer.resize(start_idx + to_read, 0.0);
                let actual_read = consumer.pop_slice(&mut buffer[start_idx..]);

                if actual_read < to_read {
                    buffer.truncate(start_idx + actual_read);
                }
            }

            if buffer.len() >= self.target_size {
                log::debug!("Collected {} samples, returning snippet", buffer.len());
                return buffer;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
