//! NetStream command messages.

use std::borrow::Cow;

pub mod reader;

/// NetStream commands as defined in 7.2.2.
#[derive(Debug, Clone, PartialEq)]
pub enum NetStreamCommand<'a> {
    /// Play command.
    ///
    /// Command object processing is not implemented for this command.
    Play,
    /// Play2 command.
    ///
    /// Command object processing is not implemented for this command.
    Play2,
    /// Delete stream command.
    DeleteStream {
        /// ID of the stream to delete.
        stream_id: f64,
    },
    /// Close stream command.
    ///
    /// Command object processing is not implemented for this command.
    CloseStream,
    /// Receive audio command.
    ///
    /// Command object processing is not implemented for this command.
    ReceiveAudio,
    /// Receive video command.
    ///
    /// Command object processing is not implemented for this command.
    ReceiveVideo,
    /// Publish command.
    Publish {
        /// Name with which the stream is published.
        publishing_name: Cow<'a, str>,
        /// Type of publishing.
        publishing_type: NetStreamCommandPublishPublishingType,
    },
    /// Seek command.
    ///
    /// Command object processing is not implemented for this command.
    Seek,
    /// Pause command.
    ///
    /// Command object processing is not implemented for this command.
    Pause,
}

/// Type of publishing.
///
/// Appears as part of the [`NetStreamCommand::Publish`] command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetStreamCommandPublishPublishingType {
    /// Citing the legacy RTMP spec, page 46:
    /// Live data is published without recording it in a file.
    Live,
    /// Citing the legacy RTMP spec, page 46:
    /// > The stream is published and the
    /// > data is recorded to a new file. The file
    /// > is stored on the server in a
    /// > subdirectory within the directory that
    /// > contains the server application. If the
    /// > file already exists, it is overwritten.
    Record,
    /// Citing the legacy RTMP spec, page 46:
    /// The stream is published and the
    /// data is appended to a file. If no file
    /// is found, it is created.
    Append,
    /// Any other value.
    Unknown(String),
}
