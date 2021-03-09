use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    /*
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    */
    /*
    #[error("invalid header (platform {platform:?}, found {found:?})")]
    InvalidHeader {
        platform: String,
        found: String,
    },
    */
    /*
    #[error("unknown data store error")]
    Unknown,
    */
    
    #[error("Credentials not found for ${platform:?}")]
    CredentialsNotFound {
        platform: String,
    }    
}