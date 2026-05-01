use thiserror::Error;

#[derive(Error, Debug)]
pub enum EInkError {
    #[error("Screen error")]
    Init(#[from] std::io::Error),
    #[error("Plot error")]
    Plot(),
}

pub type Result<T> = std::result::Result<T, EInkError>;
