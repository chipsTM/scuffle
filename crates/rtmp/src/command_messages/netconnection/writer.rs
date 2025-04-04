//! Writing [`NetConnectionCommand`].

use std::io;

use scuffle_amf0::encoder::Amf0Encoder;
use scuffle_amf0::{Amf0Object, Amf0Value};

use super::{NetConnectionCommand, NetConnectionCommandConnectResult};
use crate::command_messages::error::CommandError;

impl NetConnectionCommand<'_> {
    /// Writes a [`NetConnectionCommand`] to the given writer.
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        let mut encoder = Amf0Encoder::new(buf);

        match self {
            Self::ConnectResult(NetConnectionCommandConnectResult {
                fmsver,
                capabilities,
                level,
                code,
                description,
                encoding,
            }) => {
                encoder.encode_string("_result")?;
                encoder.encode_number(transaction_id)?;
                let object: Amf0Object = [
                    ("fmsVer".into(), Amf0Value::String(fmsver)),
                    ("capabilities".into(), Amf0Value::Number(capabilities)),
                ]
                .into_iter()
                .collect();
                encoder.encode_object(&object)?;

                let parameters: Amf0Object = [
                    ("level".into(), Amf0Value::String(level.as_ref().into())),
                    ("code".into(), Amf0Value::String(code.0.into())),
                    ("description".into(), Amf0Value::String(description)),
                    ("objectEncoding".into(), Amf0Value::Number(encoding)),
                ]
                .into_iter()
                .collect();
                encoder.encode_object(&parameters)?;
            }
            Self::CreateStreamResult { stream_id } => {
                encoder.encode_string("_result")?;
                encoder.encode_number(transaction_id)?;
                encoder.encode_null()?;
                encoder.encode_number(stream_id)?;
            }
            Self::Connect { .. } | Self::Call { .. } | Self::Close | Self::CreateStream => {
                return Err(CommandError::NoClientImplementation);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, BytesMut};
    use scuffle_amf0::decoder::Amf0Decoder;

    use super::*;
    use crate::command_messages::CommandResultLevel;

    #[test]
    fn test_netconnection_connect_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::ConnectResult(NetConnectionCommandConnectResult {
            fmsver: "flashver".into(),
            capabilities: 31.0,
            level: CommandResultLevel::Status,
            code: "idk".into(),
            description: "description".into(),
            encoding: 0.0,
        })
        .write(&mut (&mut buf).writer(), 1.0)
        .expect("write");

        let mut deserializer = Amf0Decoder::new(buf.freeze());
        let values = deserializer.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(
            values[2],
            Amf0Value::Object(
                [
                    ("fmsVer".into(), Amf0Value::String("flashver".into())),
                    ("capabilities".into(), Amf0Value::Number(31.0)),
                ]
                .into_iter()
                .collect()
            )
        ); // command object
        assert_eq!(
            values[3],
            Amf0Value::Object(
                [
                    ("level".into(), Amf0Value::String("status".into())),
                    ("code".into(), Amf0Value::String("idk".into())),
                    ("description".into(), Amf0Value::String("description".into())),
                    ("objectEncoding".into(), Amf0Value::Number(0.0)),
                ]
                .into_iter()
                .collect()
            )
        ); // info object
    }

    #[test]
    fn test_netconnection_create_stream_response() {
        let mut buf = BytesMut::new();

        NetConnectionCommand::CreateStreamResult { stream_id: 1.0 }
            .write(&mut (&mut buf).writer(), 1.0)
            .expect("write");

        let mut deserializer = Amf0Decoder::new(buf.freeze());
        let values = deserializer.decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("_result".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(values[2], Amf0Value::Null); // command object
        assert_eq!(values[3], Amf0Value::Number(1.0)); // stream id
    }
}
