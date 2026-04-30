mod audio_processor;
mod bird_parser;
mod error;
mod microphone;

use crate::audio_processor::{AudioCollector, SNIPPET_SAMPLES};
use crate::bird_parser::BirdParser;
use crate::error::Result;
use crate::microphone::BirdMicrophone;
use byteorder::{LittleEndian, ReadBytesExt};
use image::EncodableLayout;
use smol_macros::main;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

struct ClassificationEntry {
    name: String,
    confidence: f32,
}

main! {
    async fn main() -> Result<()> {
        env_logger::init();
        // Init the birb socket. We are sending our microphone data into this socket.
        // the python script on the other side will receive the data and put it
        // into his ML voodoo. After it inferred a birb, it will send back
        // the classification to us.
        let mut socket = UnixStream::connect("/tmp/birb_socket")?;

        // Init the microphone. After this call, the microphone will collect data that
        // can be accessed from its ringbuffer via `rb_consumer`.
        let mut mic = BirdMicrophone::new()?;

        // Init the `AudioCollector`. This one collects the data from the microphone
        // and merges it into snippets of SNIPPET_SAMPLES size (=sample_rate * length)
        let mut collector = AudioCollector::new(SNIPPET_SAMPLES);

            loop {
                // collect an audio snippet from the mic and send it to the python ML code
                let sample = collector.collect(&mut mic.rb_consumer);
                socket.write_all(&sample.as_bytes())?;
                // then read back from the socket. We don't need any async shizzle here,
                // because for one audio snippet, we get one answer.

                // first we read the number of predictions that the ML model made
                // (we may get something like 60% Bird A, 30% Bird B, 10% Bird C)
                let num_items = socket.read_u32::<LittleEndian>()?;

                // then we read each prediction. Because birb-names happen to be of
                // variable length, our protocol is defined to first send the length
                // of the birb name, then the name itself, and finally the confidence value.
                for _ in 0..num_items {
                    let name_len = socket.read_u32::<LittleEndian>()? as usize;
                    let mut name_buf = vec![0u8; name_len];
                    socket.read_exact(&mut name_buf)?;
                    let name = String::from_utf8_lossy(&name_buf).into();
                    let confidence = socket.read_f32::<LittleEndian>()?;

                    // A ClassificationEntry also holds the confidence value.
                    // We are just undecided yet if we want to use this or not, so we
                    // havew ClassificationEntry and BirdData for now.
                    let prediction = ClassificationEntry { name, confidence };

                    let bird = BirdParser::create_bird_data(&prediction.name);
                    match bird {
                    Some(bird) => {
                        log::info!("Detected bird: {}. Path to info: {}", bird.name, bird.page);
                    },
                    None => {}

                    }
                }

            }
    }
}
