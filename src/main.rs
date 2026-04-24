mod audio_processor;
mod bird_detection;
mod error;
mod microphone;
mod page_builder;

use crate::audio_processor::{AudioCollector, SNIPPET_SAMPLES};
use crate::bird_detection::BirdClassifier;
use crate::error::Result;
use crate::microphone::BirdMicrophone;
use smol_macros::main;
use std::fs::File;
use std::io::Write;

main! {
    async fn main() -> Result<()> {
        env_logger::init();

        // Init the ML Classifier for birb sounds.
        let classifier = BirdClassifier::new()?;

        // Init the microphone. After this call, the microphone will collect data that
        // can be accessed from its ringbuffer via `rb_consumer`.
        let mut mic = BirdMicrophone::new()?;

        // Init the `AudioCollector` to collect snippets of SNIPPET_SAMPLES size.
        // (sample_rate * length)
        let mut collector = AudioCollector::new(SNIPPET_SAMPLES);

        // run the collector with the mic's ringbuffer consumer.
            loop {
                // collect an audio snippet from the mic.
                let sample = collector.collect(&mut mic.rb_consumer);

                let prediction = classifier.predict(&sample);

                if let Some(birb) = prediction {
                    let page = page_builder::HtmlPage::new(&birb);
                    log::info!("Detected bird: {}. Path to info: {}", birb.name, page.path);

                    let mut f = File::create("index.html")?;
                    f.write_all(format!(r#"<html><head><meta http-equiv="refresh" content="0; url={}" /></head></html>"#, page.path).as_bytes())?;
                }
            }
    }
}
