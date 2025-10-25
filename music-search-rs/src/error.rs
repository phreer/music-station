use thiserror::Error;

#[derive(Error, Debug)]
pub enum MusicSearchError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Requires login")]
    RequiresLogin,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Other error: {0}")]
    Other(String),

}

impl From<quick_xml::Error> for MusicSearchError {
    fn from(err: quick_xml::Error) -> Self {
        MusicSearchError::XmlParse(err.to_string())
    }
}

impl From<quick_xml::escape::EscapeError> for MusicSearchError {
    fn from(err: quick_xml::escape::EscapeError) -> Self {
        MusicSearchError::XmlParse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MusicSearchError>;
