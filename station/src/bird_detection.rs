use crate::error::Result;
use birdnet_onnx::{Classifier, InferenceOptions};


pub type BirdName = String;
// These items are also baked in the ML model, but we exclude them because of
// their un-birby nature.
const EXCLUDED_ITEMS: [&str; 11] = [
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
    "Power Tools",
];


pub struct BirdClassifier {
    classifier: Classifier,
}


/// Classifier for birb sounds. This struct is responsible for accepting birb audio snippets
/// and infer birb names.
/// The pipeline goes as follows:
/// - new(): Init the classifier.
/// - predict(): Predict a birb name from a given audio snippet.
/// - create_bird_data(): Return the final birb data object. (latin name of a birb)
impl BirdClassifier {
    /// Init the classifier.
    pub fn new() -> Result<BirdClassifier> {
        // Init the ML Classifier for birb sounds.
        // We use a top_k of 1, so we only get one result back.
        let classifier = Classifier::builder()
            .model_path("model_files/birdnet_v24.onnx".to_string())
            .labels_path("model_files/labels/en_us.txt".to_string())
            .top_k(1)
            .min_confidence(0.5)
            .build()?;

        Ok(BirdClassifier { classifier })
    }

    /// Predict a birb entry in the labels file from a given audio snippet.
    pub fn predict(&self, audio: &[f32]) -> Option<BirdName> {
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

        // We have top_k=1. The classifier will return 0 or 1 predictions; 0 if nothing
        // is detected and the classifier didn't fail for any other reason.
        if !prediction.predictions.is_empty() {
            let pred: BirdName = prediction.predictions[0].species.clone();
            // Exclude non-birbs.
            if EXCLUDED_ITEMS.contains(&pred.as_str()) {
                return None;
            }
            return self.create_bird_data(&pred);
        }
        // now we return `None` if no birb has been detected.
        None
    }

    /// Create a final birb data object.
    /// The ML model used in `predict()` is designed to output birb names as {latin}_{english-common}.
    /// This method is responsible for post-processing the output from the ML model.
    /// Concretely, we are just extracting the latin name from the ML output and exclude non-birb items.
    /// But from an architecture standpoint, any post-processing can be done here.
    fn create_bird_data(&self, name: &str) -> Option<BirdName> {
        let latin_name: BirdName = name.split('_').next().unwrap().into();

        if EXCLUDED_ITEMS.contains(&latin_name.as_str()) {
            return None;
        }

        Some(latin_name)
    }
}
