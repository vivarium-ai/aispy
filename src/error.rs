use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("entity resolution failed")]
    EntityResolution,

    #[error("failed to load CUPTI: {message}")]
    CuptiLoad {
        message: String,
    },

    #[error("CUPTI call {function} failed with result {result}")]
    CuptiCall {
        function: &'static str,
        result: i32,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
