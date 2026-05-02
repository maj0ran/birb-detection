use serde::Serialize;

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

#[derive(Serialize)]
pub struct BirdData {
    pub name: String,
}

pub struct BirdParser {}

impl BirdParser {
    pub fn create_bird_data(name: &str) -> Option<BirdData> {
        // the ML model is designed to output birb names as {latin}_{english-common}.
        // we only want the latin name as these are the names in our encyclopedia.
        let latin_name = name.split("_").next().unwrap();

        if EXCLUDED_ITEMS.contains(&latin_name) {
            return None;
        }

        Some(BirdData {
            name: latin_name.into(),
        })
    }
}
