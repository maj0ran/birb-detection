use crate::error::Result;
use birdnet_onnx::{Classifier, InferenceOptions};
use image::{DynamicImage, ImageReader};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read};

// These items are also baked in the ML model, but we exclude them because of
// their un-birby nature.
const EXCLUDED_ITEMS: [&str; 10] = [
    "Human non-vocal",
    "Human vocal",
    "Human whistle",
    "Engine",
    "Dog",
    "Noise",
    "Siren",
    "Environmental",
    "Fireworks",
    "Gun",
];

#[derive(Debug)]
pub struct BirdData {
    pub name: String,
    pub translation: String,
    pub desc: Option<String>,
    pub image: Option<DynamicImage>,
}

pub struct BirdClassifier {
    classifier: Classifier,
    translations: HashMap<String, String>,
}

impl BirdClassifier {
    pub fn new() -> Result<BirdClassifier> {
        // Init the ML Classifier for birb sounds.
        // We use a top_k of 1, so we only get one result back.
        let classifier = Classifier::builder()
            .model_path("model_files/birdnet_v24.onnx".to_string())
            .labels_path("model_files/labels/en_us.txt".to_string())
            .top_k(1)
            .min_confidence(0.1)
            .build()?;

        let mut translations = HashMap::new();
        let translation_file = File::open("model_files/labels/de.txt")?;
        let lines = io::BufReader::new(translation_file).lines();
        for l in lines {
            match l {
                Ok(l) => {
                    let split = l.splitn(2, ":").collect::<Vec<&str>>();
                    if split.len() == 2 {
                        translations
                            .insert(split[0].trim().to_string(), split[1].trim().to_string());
                    }
                }
                Err(e) => {
                    log::error!("Error reading label file: {}", e);
                    continue;
                }
            }
        }

        Ok(BirdClassifier {
            classifier,
            translations,
        })
    }

    pub fn predict(&self, audio: &[f32]) -> Option<BirdData> {
        // Get the prediction from the classifier (or `None` if the classifier failed).
        // Note: This doesn't return `None` if the classifier didn't detect a birb. In this
        // case, it will return a prediction-list of length 0.
        let prediction = if let Some(result) = self
            .classifier
            .predict(audio, &InferenceOptions::default())
            .ok()
        {
            result
        } else {
            return None;
        };

        // We have top_k=1. The classifier will return 0 or 1 predictions; 0 when nothing
        // is detected, but the classifier didn't fail for any other reason.
        // If we have != 0 predictions, we just take the first and only.
        if !prediction.predictions.is_empty() {
            let pred = &prediction.predictions[0];

            // Exclude non-birbs.
            if EXCLUDED_ITEMS.contains(&pred.species.as_str()) {
                return None;
            }

            Some(self.create_bird_data(&pred.species))
        } else {
            None
        }
    }

    fn create_bird_data(&self, name: &str) -> BirdData {
        let desc_file = File::open(format!("encyclopedia/{}/description.txt", name));
        let img_file_path = format!("encyclopedia/{}/image.jpg", name);

        let image = ImageReader::open(img_file_path)
            .ok()
            .and_then(|reader| reader.decode().ok());

        let translation = self
            .translations
            .get(name)
            .cloned()
            .unwrap_or(name.to_string());

        let desc = desc_file.ok().and_then(|mut f| {
            let mut text_buf = String::new();
            f.read_to_string(&mut text_buf).ok().map(|_| text_buf)
        });

        BirdData {
            name: name.to_string(),
            translation,
            desc,
            image,
        }
    }
}