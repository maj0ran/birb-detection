use crate::error::{BirdError, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, FrameCount, InputCallbackInfo, Stream, StreamConfig, SupportedBufferSize,
    default_host,
};
use ringbuf::{HeapRb, traits::*};

// Microphone sample rate.
// Our Birb classifier expects audio of this sample rate to correctly inference birb sounds.
pub const SAMPLE_RATE: u32 = 48000;
// size for the internal CPAL microphone buffer. This is required by the CPAL API and used for
// immediate microphone data capture. This is different to the ring buffer, which we use to safely
// send the data out of the microphone thread.
const AUDIO_BUFFER_SIZE: usize = 512;
// we want 3-second-snippets of audio, but let's make the ring buffer twice as large for good measure.
const RING_BUFFER_SIZE: usize = SAMPLE_RATE as usize * 3 * 2;

/// The `BirdMicrophone` is responsible for capturing audio from the microphone and storing it
/// in a ring buffer.
pub struct BirdMicrophone {
    pub rb_consumer: ringbuf::HeapCons<f32>,
    _stream: Stream,
}

impl BirdMicrophone {
    pub fn new() -> Result<BirdMicrophone> {
        // Microphone initialization hassle...
        let host = default_host();
        let input = host
            .default_input_device()
            .ok_or_else(|| BirdError::Microphone("no input device available".to_string()))?;

        log::info!(
            "Using input device: {}",
            input
                .description()
                .map_err(|e| BirdError::Microphone(e.to_string()))?
        );
        // Sanity checks if our platform supports the buffer size we want to use.
        let mut supported_configs_range = input.supported_input_configs()?;
        let supported_config = supported_configs_range
            .next()
            .ok_or_else(|| BirdError::Microphone("no supported config?!".to_string()))?
            .with_sample_rate(SAMPLE_RATE);

        match supported_config.buffer_size() {
            SupportedBufferSize::Range { min, max } => {
                if *min > AUDIO_BUFFER_SIZE as u32 || *max < AUDIO_BUFFER_SIZE as u32 {
                    return Err(BirdError::Microphone(
                        "Buffer size is out of range".to_string(),
                    ));
                }
            }
            SupportedBufferSize::Unknown => {
                // Platform doesn't expose buffer size control
                return Err(BirdError::Microphone(
                    "Buffer size cannot be queried on this platform".to_string(),
                ));
            }
        };
        let mut config: StreamConfig = supported_config.into();
        config.buffer_size = BufferSize::Fixed(AUDIO_BUFFER_SIZE as FrameCount);

        // Create a ring buffer to hold the audio samples and split it into a producer and consumer side.
        // The producer is the 'input' side and will be put into the mic listener thread. In this thread,
        // the mic will put its audio data into the ring buffer.
        // The consumer is the 'output' side and will be accessible from the outside. From there, we can
        // pull out the audio data again to process it further.
        let rb = HeapRb::<f32>::new(RING_BUFFER_SIZE);
        let (mut producer, consumer) = rb.split();

        // Initializes a thread that will periodically read from the microphone.
        // The `data_callback` is CPAL API and is called every time the `data` buffer with the given size is filled.
        // Using this callback, we will push the mic data into our ringbuffer where it will be accessible from the outside.
        let _stream = input.build_input_stream(
            &config,
            move |data: &[f32], _: &InputCallbackInfo| {
                let _ = producer.push_slice(data);
            },
            |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
            None,
        )?;
        // CPAL Docs says that this call might be needed on some systems,
        // while other systems play without it.
        _stream.play()?;

        log::info!("--- Microphone initialized ---");

        Ok(BirdMicrophone {
            rb_consumer: consumer,
            _stream, // we have to save this, because dropping the stream will stop the thread.
        })
    }
}
