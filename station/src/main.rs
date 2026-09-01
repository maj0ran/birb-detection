mod audio_processor;
mod bird_detection;
mod error;
mod microphone;
mod transmitter;

use crate::audio_processor::{AudioCollector, SNIPPET_SAMPLES};
use crate::bird_detection::BirdClassifier;
use crate::error::Result;
use crate::microphone::BirdMicrophone;
use crate::transmitter::Transmitter;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ip address of the birb-server
    #[arg(long)]
    ip: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("--- Starting birb-station ---");

    // Init the microphone. After this call, the microphone will collect data that
    // can be accessed from its ringbuffer via `rb_consumer`.
    let mut mic = BirdMicrophone::new()?;

    // Init the `AudioCollector`. This one collects the data from the microphone
    // and merges it into snippets of SNIPPET_SAMPLES size (=sample_rate * length)
    let mut collector = AudioCollector::new(SNIPPET_SAMPLES);

    // create the ML classifier.
    // Here we'll insert the audio snippets and the magic will output birbs.
    let classifier = BirdClassifier::new()?;

    // Start the transmitter server. This is a TCP stream that outside clients can connect to, like our bird-display.
    // When the bird-station detects a birb, it will broadcast the information to all connected clients.
    // `tx` is the channel that we use to send the birbsies to the transmitter.
    let transmitter = Transmitter::new().await?;

    loop {
        // collect an audio snippet from the mic and insert it into the ML classifier.
        let sample = collector.collect(&mut mic.rb_consumer);
        let pred = classifier.predict(&sample);
        log::info!("Prediction: {:?}", pred);

        match pred {
            Some(pred) => transmitter.send(pred)?,
            None => (),
        }
    }
}
