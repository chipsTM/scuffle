//! Command error type.

/// Errors that can occur when processing command messages.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Amf0 read error.
    #[error("amf0 read: {0}")]
    Amf0Read(#[from] scuffle_amf0::Amf0ReadError),
    /// Amf0 write error.
    #[error("amf0 write: {0}")]
    Amf0Write(#[from] scuffle_amf0::Amf0WriteError),
    /// No app name of type string in connect command.
    #[error("no app name of type string in connect command")]
    NoAppName,
    /// Received an invalid onStatus info object.
    #[error("invalid onStatus info object")]
    InvalidOnStatusInfoObject,
    /// The RTMP client is not implemented yet.
    #[error("the rtmp client is not implemented yet")]
    NoClientImplementation,
}
