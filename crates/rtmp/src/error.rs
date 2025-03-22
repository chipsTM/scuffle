use crate::chunk::error::ChunkReadError;
use crate::command_messages::error::CommandError;
use crate::handshake::complex::error::ComplexHandshakeError;
use crate::session::error::SessionError;

#[derive(Debug, thiserror::Error)]
pub enum RtmpError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("chunk read error: {0}")]
    ChunkRead(#[from] ChunkReadError),
    #[error("command error: {0}")]
    Command(#[from] CommandError),
    #[error("complex handshake error: {0}")]
    ComplexHandshake(#[from] ComplexHandshakeError),
    #[error("session error: {0}")]
    Session(#[from] SessionError),
}

impl RtmpError {
    pub fn is_client_closed(&self) -> bool {
        match self {
            Self::Io(err) => matches!(
                err.kind(),
                std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::UnexpectedEof
            ),
            Self::Session(SessionError::Timeout(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::future;
    use std::io::ErrorKind;
    use std::time::Duration;

    use crate::error::RtmpError;
    use crate::session::error::SessionError;

    #[tokio::test]
    async fn test_is_client_closed() {
        assert!(RtmpError::Io(std::io::Error::new(ErrorKind::ConnectionAborted, "test")).is_client_closed());
        assert!(RtmpError::Io(std::io::Error::new(ErrorKind::ConnectionReset, "test")).is_client_closed());
        assert!(RtmpError::Io(std::io::Error::new(ErrorKind::UnexpectedEof, "test")).is_client_closed());

        let elapsed = tokio::time::timeout(Duration::ZERO, future::pending::<()>())
            .await
            .unwrap_err();

        assert!(RtmpError::Session(SessionError::Timeout(elapsed)).is_client_closed());

        assert!(!RtmpError::Io(std::io::Error::other("test")).is_client_closed());
    }
}
