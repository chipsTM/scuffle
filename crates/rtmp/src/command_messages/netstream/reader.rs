//! Reading [`NetStreamCommand`].

use std::convert::Infallible;
use std::str::FromStr;

use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Value};

use super::{NetStreamCommand, NetStreamCommandPublishPublishingType};
use crate::command_messages::error::CommandError;

impl<'a> NetStreamCommand<'a> {
    /// Reads a [`NetStreamCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read(command_name: &str, decoder: &mut Amf0Decoder<'a>) -> Result<Option<Self>, CommandError> {
        match command_name {
            "play" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let values = decoder.decode_all()?;

                Ok(Some(Self::Play { values }))
            }
            "play2" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let Amf0Value::Object(parameters) = decoder.decode_with_type(Amf0Marker::Object)? else {
                    // TODO: CLOUD-91
                    unreachable!();
                };

                Ok(Some(Self::Play2 { parameters }))
            }
            "deleteStream" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let Amf0Value::Number(stream_id) = decoder.decode_with_type(Amf0Marker::Number)? else {
                    // TODO: CLOUD-91
                    unreachable!();
                };

                Ok(Some(Self::DeleteStream { stream_id }))
            }
            "closeStream" => Ok(Some(Self::CloseStream)),
            "receiveAudio" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let receive_audio = decoder.decode_with_type(Amf0Marker::Boolean)?.as_boolean()?;

                Ok(Some(Self::ReceiveAudio { receive_audio }))
            }
            "receiveVideo" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let receive_video = decoder.decode_with_type(Amf0Marker::Boolean)?.as_boolean()?;

                Ok(Some(Self::ReceiveVideo { receive_video }))
            }
            "publish" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let Amf0Value::String(publishing_name) = decoder.decode_with_type(Amf0Marker::String)? else {
                    // TODO: CLOUD-91
                    unreachable!();
                };
                let Amf0Value::String(publishing_type) = decoder.decode_with_type(Amf0Marker::String)? else {
                    // TODO: CLOUD-91
                    unreachable!();
                };
                // TODO: change expect to into_ok once stabliized
                // https://doc.rust-lang.org/std/result/enum.Result.html#method.into_ok
                let publishing_type =
                    NetStreamCommandPublishPublishingType::from_str(&publishing_type).expect("infalible error");

                Ok(Some(Self::Publish {
                    publishing_name,
                    publishing_type,
                }))
            }
            "seek" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let milliseconds = decoder.decode_with_type(Amf0Marker::Number)?.as_number()?;

                Ok(Some(Self::Seek { milliseconds }))
            }
            "pause" => {
                // skip command object
                decoder.decode_with_type(Amf0Marker::Null)?;

                let pause = decoder.decode_with_type(Amf0Marker::Boolean)?.as_boolean()?;
                let milliseconds = decoder.decode_with_type(Amf0Marker::Number)?.as_number()?;

                Ok(Some(Self::Pause { pause, milliseconds }))
            }
            _ => Ok(None),
        }
    }
}

impl FromStr for NetStreamCommandPublishPublishingType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "live" => Ok(Self::Live),
            "record" => Ok(Self::Record),
            "append" => Ok(Self::Append),
            _ => Ok(Self::Unknown(s.to_string())),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::str::FromStr;

    use scuffle_amf0::{Amf0Decoder, Amf0Encoder, Amf0Marker};

    use super::NetStreamCommandPublishPublishingType;
    use crate::command_messages::netstream::NetStreamCommand;

    #[test]
    fn test_command_no_payload() {
        let command = NetStreamCommand::read("closeStream", &mut Amf0Decoder::new(&[]))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::CloseStream);
    }

    #[test]
    fn test_command_delete_stream() {
        let mut payload = vec![Amf0Marker::Null as u8, Amf0Marker::Number as u8];
        payload.extend_from_slice(0.0f64.to_be_bytes().as_ref());

        let command = NetStreamCommand::read("deleteStream", &mut Amf0Decoder::new(&payload))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::DeleteStream { stream_id: 0.0 });
    }

    #[test]
    fn test_command_publish() {
        let mut payload = Vec::new();

        Amf0Encoder::encode_null(&mut payload).unwrap();
        Amf0Encoder::encode_string(&mut payload, "live").unwrap();
        Amf0Encoder::encode_string(&mut payload, "record").unwrap();

        let command = NetStreamCommand::read("publish", &mut Amf0Decoder::new(&payload))
            .unwrap()
            .unwrap();

        assert_eq!(
            command,
            NetStreamCommand::Publish {
                publishing_name: "live".into(),
                publishing_type: NetStreamCommandPublishPublishingType::Record
            }
        );
    }

    #[test]
    fn test_parse_publishing_type() {
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("live").unwrap(),
            NetStreamCommandPublishPublishingType::Live
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("record").unwrap(),
            NetStreamCommandPublishPublishingType::Record
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("append").unwrap(),
            NetStreamCommandPublishPublishingType::Append
        );
        assert_eq!(
            NetStreamCommandPublishPublishingType::from_str("unknown").unwrap(),
            NetStreamCommandPublishPublishingType::Unknown("unknown".to_string())
        );
    }
}
