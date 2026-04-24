use crate::bird_detection::BirdData;

pub struct HtmlPage {
    pub path: String,
}

impl HtmlPage {
    pub fn new(bird: &BirdData) -> Self {
        let path = format!("encyclopedia/{}/index.html", bird.name);
        HtmlPage { path }
    }
}
