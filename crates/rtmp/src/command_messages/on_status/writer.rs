//! Writing [`OnStatus`].

use std::io;

use scuffle_amf0::encoder::Amf0Encoder;
use scuffle_amf0::{Amf0Object, Amf0Value};

use super::OnStatus;
use crate::command_messages::error::CommandError;

impl OnStatus<'_> {
    /// Writes an [`OnStatus`] command to the given writer.
    pub fn write(self, buf: &mut impl io::Write, transaction_id: f64) -> Result<(), CommandError> {
        let mut encoder = Amf0Encoder::new(buf);

        encoder.encode_string("onStatus")?;
        encoder.encode_number(transaction_id)?;
        // command object is null
        encoder.encode_null()?;

        let mut info_object = Amf0Object::new();

        info_object.insert("level".into(), Amf0Value::String(self.level.to_string().into()));
        info_object.insert("code".into(), Amf0Value::String(self.code.0.into()));

        if let Some(description) = self.description {
            info_object.insert("description".into(), Amf0Value::String(description));
        }

        if let Some(others) = self.others {
            info_object.extend(others);
        }

        encoder.encode_object(&info_object)?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::{BufMut, BytesMut};
    use scuffle_amf0::Amf0Value;
    use scuffle_amf0::decoder::Amf0Decoder;

    use crate::command_messages::CommandResultLevel;
    use crate::command_messages::on_status::OnStatus;

    #[test]
    fn test_write_on_status() {
        let mut buf = BytesMut::new();

        OnStatus {
            level: CommandResultLevel::Status,
            code: "idk".into(),
            description: Some("description".into()),
            others: Some(
                [("testkey".into(), Amf0Value::String("testvalue".into()))]
                    .into_iter()
                    .collect(),
            ),
        }
        .write(&mut (&mut buf).writer(), 1.0)
        .expect("write");

        let values = Amf0Decoder::new(buf.freeze()).decode_all().unwrap();

        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Amf0Value::String("onStatus".into())); // command name
        assert_eq!(values[1], Amf0Value::Number(1.0)); // transaction id
        assert_eq!(values[2], Amf0Value::Null); // command object
        assert_eq!(
            values[3],
            Amf0Value::Object(
                [
                    ("level".into(), Amf0Value::String("status".into())),
                    ("code".into(), Amf0Value::String("idk".into())),
                    ("description".into(), Amf0Value::String("description".into())),
                    ("testkey".into(), Amf0Value::String("testvalue".into())),
                ]
                .into_iter()
                .collect()
            )
        ); // info object
    }
}
