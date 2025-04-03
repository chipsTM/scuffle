//! Reading [`NetStreamCommand`].

use scuffle_amf0::Amf0Object;
use scuffle_bytes_util::StringCow;
use serde::Deserialize;

use super::{NetStreamCommand, NetStreamCommandPublishPublishingType};
use crate::command_messages::error::CommandError;

impl NetStreamCommand<'_> {
    /// Reads a [`NetStreamCommand`] from the given decoder.
    ///
    /// Returns `Ok(None)` if the `command_name` is not recognized.
    pub fn read(command_name: &str, deserializer: &mut scuffle_amf0::Deserializer) -> Result<Option<Self>, CommandError> {
        match command_name {
            "play" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let values = deserializer.deserialize_all()?;
                Ok(Some(Self::Play { values }))
            }
            "play2" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let parameters = Amf0Object::deserialize(deserializer)?;
                Ok(Some(Self::Play2 { parameters }))
            }
            "deleteStream" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let stream_id = f64::deserialize(deserializer)?;
                Ok(Some(Self::DeleteStream { stream_id }))
            }
            "closeStream" => Ok(Some(Self::CloseStream)),
            "receiveAudio" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let receive_audio = bool::deserialize(deserializer)?;
                Ok(Some(Self::ReceiveAudio { receive_audio }))
            }
            "receiveVideo" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let receive_video = bool::deserialize(deserializer)?;
                Ok(Some(Self::ReceiveVideo { receive_video }))
            }
            "publish" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let publishing_name = StringCow::deserialize(&mut *deserializer)?;
                let publishing_type = NetStreamCommandPublishPublishingType::deserialize(deserializer)?;

                Ok(Some(Self::Publish {
                    publishing_name,
                    publishing_type,
                }))
            }
            "seek" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let milliseconds = f64::deserialize(deserializer)?;
                Ok(Some(Self::Seek { milliseconds }))
            }
            "pause" => {
                // skip command object
                <()>::deserialize(&mut *deserializer)?;

                let pause = bool::deserialize(&mut *deserializer)?;
                let milliseconds = f64::deserialize(deserializer)?;
                Ok(Some(Self::Pause { pause, milliseconds }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;
    use scuffle_amf0::{Amf0Marker, Amf0Object};
    use scuffle_bytes_util::StringCow;
    use serde::Serialize;

    use super::NetStreamCommandPublishPublishingType;
    use crate::command_messages::netstream::NetStreamCommand;

    #[test]
    fn test_command_no_payload() {
        let command = NetStreamCommand::read("closeStream", &mut scuffle_amf0::Deserializer::new(Bytes::new()))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::CloseStream);
    }

    #[test]
    fn play_command() {
        let mut payload = Vec::new();

        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        0.0f64.serialize(&mut serializer).unwrap();
        "test".serialize(&mut serializer).unwrap();

        let command = NetStreamCommand::read("play", &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();

        assert_eq!(
            command,
            NetStreamCommand::Play {
                values: vec![0.0f64.into(), StringCow::from("test").into(),],
            }
        );
    }

    #[test]
    fn play2_command() {
        let mut payload = Vec::new();

        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        let mut object = Amf0Object::new();
        object.insert("name".into(), StringCow::from("test").into());
        object.insert("value".into(), 0.0f64.into());
        object.serialize(&mut serializer).unwrap();

        let command = NetStreamCommand::read("play2", &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();

        assert_eq!(command, NetStreamCommand::Play2 { parameters: object });
    }

    #[test]
    fn receive_audio() {
        let mut payload = Vec::new();
        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        true.serialize(&mut serializer).unwrap();

        let command = NetStreamCommand::read(
            "receiveAudio",
            &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)),
        )
        .unwrap()
        .unwrap();
        assert_eq!(command, NetStreamCommand::ReceiveAudio { receive_audio: true });
    }

    #[test]
    fn receive_video() {
        let mut payload = Vec::new();
        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        true.serialize(&mut serializer).unwrap();

        let command = NetStreamCommand::read(
            "receiveVideo",
            &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)),
        )
        .unwrap()
        .unwrap();
        assert_eq!(command, NetStreamCommand::ReceiveVideo { receive_video: true });
    }

    #[test]
    fn delete_stream() {
        let mut payload = vec![Amf0Marker::Null as u8, Amf0Marker::Number as u8];
        payload.extend_from_slice(0.0f64.to_be_bytes().as_ref());

        let command = NetStreamCommand::read(
            "deleteStream",
            &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)),
        )
        .unwrap()
        .unwrap();
        assert_eq!(command, NetStreamCommand::DeleteStream { stream_id: 0.0 });
    }

    #[test]
    fn publish() {
        let mut payload = Vec::new();

        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        "live".serialize(&mut serializer).unwrap();
        NetStreamCommandPublishPublishingType::Record
            .serialize(&mut serializer)
            .unwrap();

        let command = NetStreamCommand::read("publish", &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)))
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
    fn seek() {
        let mut payload = Vec::new();
        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        0.0f64.serialize(&mut serializer).unwrap();

        let command = NetStreamCommand::read("seek", &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(command, NetStreamCommand::Seek { milliseconds: 0.0 });
    }

    #[test]
    fn pause() {
        let mut payload = Vec::new();
        let mut serializer = scuffle_amf0::Serializer::new(&mut payload);
        ().serialize(&mut serializer).unwrap();
        true.serialize(&mut serializer).unwrap();
        0.0f64.serialize(&mut serializer).unwrap();

        let command = NetStreamCommand::read("pause", &mut scuffle_amf0::Deserializer::new(Bytes::from_owner(payload)))
            .unwrap()
            .unwrap();
        assert_eq!(
            command,
            NetStreamCommand::Pause {
                pause: true,
                milliseconds: 0.0
            }
        );
    }
}
