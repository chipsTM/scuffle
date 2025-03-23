//! Defines types for handling session events.

use bytes::Bytes;

use super::error::ServerSessionError;

/// Data received from a session.
#[derive(Debug, Clone)]
pub enum SessionData {
    /// Video data.
    Video {
        /// Timestamp of the data.
        timestamp: u32,
        /// Data.
        data: Bytes,
    },
    /// Audio data.
    Audio {
        /// Timestamp of the data.
        timestamp: u32,
        /// Data.
        data: Bytes,
    },
    /// Metadata.
    Amf0 {
        /// Timestamp of the data.
        timestamp: u32,
        /// Data.
        data: Bytes,
    },
}

/// Handler for session events.
pub trait SessionHandler {
    /// Called when a stream is published.
    fn on_publish(
        &mut self,
        stream_id: u32,
        app_name: &str,
        stream_name: &str,
    ) -> impl std::future::Future<Output = Result<(), ServerSessionError>> + Send;

    /// Called when a stream is unpublished.
    fn on_unpublish(&mut self, stream_id: u32) -> impl std::future::Future<Output = Result<(), ServerSessionError>> + Send;

    /// Called when data is received.
    fn on_data(
        &mut self,
        stream_id: u32,
        data: SessionData,
    ) -> impl std::future::Future<Output = Result<(), ServerSessionError>> + Send;
}
