use ringbuf::traits::*;

type AudioSnippet = Vec<f32>;

pub struct AudioCollector;

impl AudioCollector {
    /// Pulls samples from the consumer until an `AudioSnippet` of target size has been fully collected.
    /// Then returns the `AudioSnippet`.
    pub fn collect<C: Consumer<Item = f32>>(consumer: &mut C, target_size: usize) -> AudioSnippet {
        let mut buffer = Vec::with_capacity(target_size);
        loop {
            let remaining = target_size - buffer.len();
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
            // audio snippet complete, return data
            if buffer.len() >= target_size {
                log::debug!("Collected {} samples, returning snippet", buffer.len());
                return buffer;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
