use thiserror::Error;

#[derive(Error, Debug)]
pub enum BirdError {
    #[error("Microphone error: {0}")]
    Microphone(String),

    #[error("CPAL supported configs error: {0}")]
    CpalSupportedConfigs(#[from] cpal::SupportedStreamConfigsError),

    #[error("CPAL stream error: {0}")]
    CpalStream(#[from] cpal::BuildStreamError),

    #[error("CPAL play error: {0}")]
    CpalPlay(#[from] cpal::PlayStreamError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, BirdError>;
