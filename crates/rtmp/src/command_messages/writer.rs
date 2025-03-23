//! Writing [`Command`].

use std::io;

use bytes::Bytes;

use super::error::CommandError;
use super::{Command, CommandResultLevel, CommandType};
use crate::chunk::writer::ChunkWriter;
use crate::chunk::{CHUNK_STREAM_ID_COMMAND, Chunk};
use crate::error::RtmpError;
use crate::messages::MessageType;

impl CommandResultLevel {
    /// Converts the [`CommandResultLevel`] to a `&str`.
    pub fn to_str(&self) -> &str {
        match self {
            CommandResultLevel::Warning => "warning",
            CommandResultLevel::Status => "status",
            CommandResultLevel::Error => "error",
            CommandResultLevel::Unknown(s) => s,
        }
    }

    /// Converts the [`CommandResultLevel`] to a [`String`] by taking ownership.
    pub fn into_string(self) -> String {
        match self {
            CommandResultLevel::Warning => "warning".to_string(),
            CommandResultLevel::Status => "status".to_string(),
            CommandResultLevel::Error => "error".to_string(),
            CommandResultLevel::Unknown(s) => s,
        }
    }
}

impl Command<'_> {
    fn write_amf0_chunk(io: &mut impl io::Write, writer: &ChunkWriter, payload: Bytes) -> io::Result<()> {
        writer.write_chunk(
            io,
            Chunk::new(CHUNK_STREAM_ID_COMMAND, 0, MessageType::CommandAMF0, 0, payload),
        )
    }

    /// Writes a [`Command`] to the given writer.
    ///
    /// Skips unknown commands.
    pub fn write(self, io: &mut impl io::Write, writer: &ChunkWriter) -> Result<(), RtmpError> {
        let mut buf = Vec::new();

        match self.command_type {
            CommandType::NetConnection(command) => {
                command.write(&mut buf, self.transaction_id)?;
            }
            CommandType::NetStream(_) => {
                return Err(RtmpError::from(CommandError::NoClientImplementation));
            }
            CommandType::OnStatus(command) => {
                command.write(&mut buf, self.transaction_id)?;
            }
            // don't write unknown commands
            CommandType::Unknown { .. } => {}
        }

        Self::write_amf0_chunk(io, writer, Bytes::from(buf))?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::super::{Command, CommandResultLevel};
    use crate::chunk::writer::ChunkWriter;
    use crate::command_messages::CommandType;
    use crate::command_messages::error::CommandError;
    use crate::command_messages::netstream::NetStreamCommand;
    use crate::error::RtmpError;

    #[test]
    fn command_result_level_to_str() {
        assert_eq!(CommandResultLevel::Warning.to_str(), "warning");
        assert_eq!(CommandResultLevel::Status.to_str(), "status");
        assert_eq!(CommandResultLevel::Error.to_str(), "error");
        assert_eq!(CommandResultLevel::Unknown("custom".to_string()).to_str(), "custom");
    }

    #[test]
    fn command_result_level_into_string() {
        assert_eq!(CommandResultLevel::Warning.into_string(), "warning");
        assert_eq!(CommandResultLevel::Status.into_string(), "status");
        assert_eq!(CommandResultLevel::Error.into_string(), "error");
        assert_eq!(CommandResultLevel::Unknown("custom".to_string()).into_string(), "custom");
    }

    #[test]
    fn netstream_command_write() {
        let mut buf = Vec::new();
        let writer = ChunkWriter::default();

        let err = Command {
            command_type: CommandType::NetStream(NetStreamCommand::Play),
            transaction_id: 1.0,
        }
        .write(&mut buf, &writer)
        .unwrap_err();

        assert!(matches!(err, RtmpError::Command(CommandError::NoClientImplementation)));
    }
}
