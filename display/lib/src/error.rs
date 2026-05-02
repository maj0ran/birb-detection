use thiserror::Error;

#[derive(Error, Debug)]
pub enum EInkError {
    #[error("Screen error")]
    Init(#[from] std::io::Error),
    #[error("Plot error")]
    Plot(),
    #[error("Generic error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, EInkError>;
