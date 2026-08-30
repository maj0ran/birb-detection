mod audio_processor;
mod bird_parser;
mod error;
mod microphone;
mod transmitter;
mod trmnl;
mod bird_detection;

use crate::audio_processor::{AudioCollector, SNIPPET_SAMPLES};
use crate::bird_parser::BirdParser;
use crate::error::Result;
use crate::microphone::BirdMicrophone;
use crate::transmitter::Transmitter;
use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use image::EncodableLayout;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::process::Command;
use birdnet_onnx::{Classifier, InferenceOptions};
use crate::bird_detection::BirdClassifier;

struct ClassificationEntry {
    name: String,
    confidence: f32,
}

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


    log::info!("Starting birb-station");

    // Init the microphone. After this call, the microphone will collect data that
    // can be accessed from its ringbuffer via `rb_consumer`.
    let mut mic = BirdMicrophone::new()?;

    // start the ML classifier.
    let classifier = BirdClassifier::new()?;

    // Init the `AudioCollector`. This one collects the data from the microphone
    // and merges it into snippets of SNIPPET_SAMPLES size (=sample_rate * length)
    let mut collector = AudioCollector::new(SNIPPET_SAMPLES);

    // Start the transmitter server. This is a TCP stream that can be connected to from
    // outside clients, like our bird-display. When the bird-station detected a birb, it will
    // broadcast the information to all connected clients.
    // `tx` is the channel that we use to send the birbsies to the transmitter.
    let (transmitter, tx) = Transmitter::new();
    tokio::spawn(async move {
        if let Err(e) = transmitter.run().await {
            log::error!("Transmitter error: {}", e);
        }
    });

    loop {
        // collect an audio snippet from the mic and send it to the python ML code
        let sample = collector.collect(&mut mic.rb_consumer);
    //    socket.write_all(&sample.as_bytes())?;
        let pred = classifier.predict(&sample);
        log::info!("Predictions: {:?}", pred);

    }

    Ok(())
}
