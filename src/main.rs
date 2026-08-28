mod audio_processor;
mod bird_parser;
mod error;
mod microphone;
mod transmitter;
mod trmnl;

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
    // Start the Python scripts that contains the ML models. This script also sets up
    // the socket for communication between rust and python.
    let _ = Command::new("python3")
        .arg("classifier.py")
        .spawn()
        .expect("failed to execute process");

    // Connect to the birb socket. We are sending our microphone data into this socket.
    // The socket is created by the Python script on the other side, which will receive
    // the data and put it into its ML voodoo. After it inferred a birb, it will send back
    // the classification to us.
    // Since we started the Python script just before, we will probably fail when trying
    // to connect immediately, so we repeat until it succeeds. This is a bit dirty, as we
    // could block here indefinitely when something goes on fire, but let's just say
    // this is fine.
    let mut socket = UnixStream::connect("/tmp/birb_socket");
    while let Err(_) = socket {
        socket = UnixStream::connect("/tmp/birb_socket")
    }
    let mut socket = socket.unwrap();

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

    tokio::spawn(async move {
        crate::trmnl::run().await;
    });

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
        let mut predictions = Vec::new();
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
            predictions.push(prediction);
        }

        // We are now only interested in the top prediction. Send it to the clients
        // when we are confident enough about the birb.
        if !predictions.is_empty() && predictions[0].confidence > 0.3 {
            let bird = BirdParser::create_bird_data(&predictions[0].name);
            match bird {
                Some(bird) => {
                    log::info!(
                        "Detected bird: {}. (Confidence: {})",
                        bird.name,
                        predictions[0].confidence
                    );
                    let _ = tx.send(bird);
                }
                None => {}
            }
        }
    }

    Ok(())
}
