use std::io;

use bytes::BytesMut;
use itertools::{Itertools, Position};
use tokio_util::codec::Encoder as _;
use vector_lib::codecs::encoding::Framer;
use vector_lib::request_metadata::GroupedCountByteSize;
use vector_lib::{config::telemetry, EstimatedJsonEncodedSizeOf};

use crate::{codecs::Transformer, event::Event, internal_events::EncoderWriteError};

pub trait Encoder<T> {
    /// Encodes the input into the provided writer.
    ///
    /// # Errors
    ///
    /// If an I/O error is encountered while encoding the input, an error variant will be returned.
    fn encode_input(
        &self,
        input: T,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, GroupedCountByteSize)>;
}

impl Encoder<Vec<Event>> for (Transformer, crate::codecs::Encoder<Framer>) {
    fn encode_input(
        &self,
        events: Vec<Event>,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, GroupedCountByteSize)> {
        let mut encoder = self.1.clone();
        let mut bytes_written = 0;
        let mut n_events_pending = events.len();
        let is_empty = events.is_empty();
        let batch_prefix = encoder.batch_prefix();
        write_all(writer, n_events_pending, batch_prefix)?;
        bytes_written += batch_prefix.len();

        let mut byte_size = telemetry().create_request_count_byte_size();

        for (position, mut event) in events.into_iter().with_position() {
            self.0.transform(&mut event);

            // Ensure the json size is calculated after any fields have been removed
            // by the transformer.
            byte_size.add_event(&event, event.estimated_json_encoded_size_of());

            let mut bytes = BytesMut::new();
            match (position, encoder.framer()) {
                (
                    Position::Last | Position::Only,
                    Framer::CharacterDelimited(_) | Framer::NewlineDelimited(_),
                ) => {
                    encoder
                        .serialize(event, &mut bytes)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                }
                _ => {
                    encoder
                        .encode(event, &mut bytes)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                }
            }
            write_all(writer, n_events_pending, &bytes)?;
            bytes_written += bytes.len();
            n_events_pending -= 1;
        }

        let batch_suffix = encoder.batch_suffix(is_empty);
        assert!(n_events_pending == 0);
        write_all(writer, 0, batch_suffix)?;
        bytes_written += batch_suffix.len();

        Ok((bytes_written, byte_size))
    }
}

impl Encoder<Event> for (Transformer, crate::codecs::Encoder<()>) {
    fn encode_input(
        &self,
        mut event: Event,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, GroupedCountByteSize)> {
        let mut encoder = self.1.clone();
        self.0.transform(&mut event);

        let mut byte_size = telemetry().create_request_count_byte_size();
        byte_size.add_event(&event, event.estimated_json_encoded_size_of());

        let mut bytes = BytesMut::new();
        encoder
            .serialize(event, &mut bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        write_all(writer, 1, &bytes)?;
        Ok((bytes.len(), byte_size))
    }
}

/// Write the buffer to the writer. If the operation fails, emit an internal event which complies with the
/// instrumentation spec- as this necessitates both an Error and EventsDropped event.
///
/// # Arguments
///
/// * `writer`           - The object implementing io::Write to write data to.
/// * `n_events_pending` - The number of events that are dropped if this write fails.
/// * `buf`              - The buffer to write.
pub fn write_all(
    writer: &mut dyn io::Write,
    n_events_pending: usize,
    buf: &[u8],
) -> io::Result<()> {
    writer.write_all(buf).inspect_err(|error| {
        emit!(EncoderWriteError {
            error,
            count: n_events_pending,
        });
    })
}

pub fn as_tracked_write<F, I, E>(inner: &mut dyn io::Write, input: I, f: F) -> io::Result<usize>
where
    F: FnOnce(&mut dyn io::Write, I) -> Result<(), E>,
    E: Into<io::Error> + 'static,
{
    struct Tracked<'inner> {
        count: usize,
        inner: &'inner mut dyn io::Write,
    }

    impl io::Write for Tracked<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            #[allow(clippy::disallowed_methods)] // We pass on the result of `write` to the caller.
            let n = self.inner.write(buf)?;
            self.count += n;
            Ok(n)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    let mut tracked = Tracked { count: 0, inner };
    f(&mut tracked, input).map_err(|e| e.into())?;
    Ok(tracked.count)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::path::PathBuf;

    use bytes::{BufMut, Bytes};
    use vector_lib::codecs::encoding::{ProtobufSerializerConfig, ProtobufSerializerOptions};
    use vector_lib::codecs::{
        CharacterDelimitedEncoder, JsonSerializerConfig, LengthDelimitedEncoder,
        NewlineDelimitedEncoder, TextSerializerConfig,
    };
    use vector_lib::event::LogEvent;
    use vector_lib::{internal_event::CountByteSize, json_size::JsonSize};
    use vrl::value::{KeyString, Value};

    use super::*;

    #[test]
    fn test_encode_batch_json_empty() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                CharacterDelimitedEncoder::new(b',').into(),
                JsonSerializerConfig::default().build().into(),
            ),
        );

        let mut writer = Vec::new();
        let (written, json_size) = encoding.encode_input(vec![], &mut writer).unwrap();
        assert_eq!(written, 2);

        assert_eq!(String::from_utf8(writer).unwrap(), "[]");
        assert_eq!(
            CountByteSize(0, JsonSize::zero()),
            json_size.size().unwrap()
        );
    }

    #[test]
    fn test_encode_batch_json_single() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                CharacterDelimitedEncoder::new(b',').into(),
                JsonSerializerConfig::default().build().into(),
            ),
        );

        let mut writer = Vec::new();
        let input = vec![Event::Log(LogEvent::from(BTreeMap::from([(
            KeyString::from("key"),
            Value::from("value"),
        )])))];

        let input_json_size = input
            .iter()
            .map(|event| event.estimated_json_encoded_size_of())
            .sum::<JsonSize>();

        let (written, json_size) = encoding.encode_input(input, &mut writer).unwrap();
        assert_eq!(written, 17);

        assert_eq!(String::from_utf8(writer).unwrap(), r#"[{"key":"value"}]"#);
        assert_eq!(CountByteSize(1, input_json_size), json_size.size().unwrap());
    }

    #[test]
    fn test_encode_batch_json_multiple() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                CharacterDelimitedEncoder::new(b',').into(),
                JsonSerializerConfig::default().build().into(),
            ),
        );

        let input = vec![
            Event::Log(LogEvent::from(BTreeMap::from([(
                KeyString::from("key"),
                Value::from("value1"),
            )]))),
            Event::Log(LogEvent::from(BTreeMap::from([(
                KeyString::from("key"),
                Value::from("value2"),
            )]))),
            Event::Log(LogEvent::from(BTreeMap::from([(
                KeyString::from("key"),
                Value::from("value3"),
            )]))),
        ];

        let input_json_size = input
            .iter()
            .map(|event| event.estimated_json_encoded_size_of())
            .sum::<JsonSize>();

        let mut writer = Vec::new();
        let (written, json_size) = encoding.encode_input(input, &mut writer).unwrap();
        assert_eq!(written, 52);

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            r#"[{"key":"value1"},{"key":"value2"},{"key":"value3"}]"#
        );

        assert_eq!(CountByteSize(3, input_json_size), json_size.size().unwrap());
    }

    #[test]
    fn test_encode_batch_ndjson_empty() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                NewlineDelimitedEncoder::default().into(),
                JsonSerializerConfig::default().build().into(),
            ),
        );

        let mut writer = Vec::new();
        let (written, json_size) = encoding.encode_input(vec![], &mut writer).unwrap();
        assert_eq!(written, 0);

        assert_eq!(String::from_utf8(writer).unwrap(), "");
        assert_eq!(
            CountByteSize(0, JsonSize::zero()),
            json_size.size().unwrap()
        );
    }

    #[test]
    fn test_encode_batch_ndjson_single() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                NewlineDelimitedEncoder::default().into(),
                JsonSerializerConfig::default().build().into(),
            ),
        );

        let mut writer = Vec::new();
        let input = vec![Event::Log(LogEvent::from(BTreeMap::from([(
            KeyString::from("key"),
            Value::from("value"),
        )])))];
        let input_json_size = input
            .iter()
            .map(|event| event.estimated_json_encoded_size_of())
            .sum::<JsonSize>();

        let (written, json_size) = encoding.encode_input(input, &mut writer).unwrap();
        assert_eq!(written, 16);

        assert_eq!(String::from_utf8(writer).unwrap(), "{\"key\":\"value\"}\n");
        assert_eq!(CountByteSize(1, input_json_size), json_size.size().unwrap());
    }

    #[test]
    fn test_encode_batch_ndjson_multiple() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                NewlineDelimitedEncoder::default().into(),
                JsonSerializerConfig::default().build().into(),
            ),
        );

        let mut writer = Vec::new();
        let input = vec![
            Event::Log(LogEvent::from(BTreeMap::from([(
                KeyString::from("key"),
                Value::from("value1"),
            )]))),
            Event::Log(LogEvent::from(BTreeMap::from([(
                KeyString::from("key"),
                Value::from("value2"),
            )]))),
            Event::Log(LogEvent::from(BTreeMap::from([(
                KeyString::from("key"),
                Value::from("value3"),
            )]))),
        ];
        let input_json_size = input
            .iter()
            .map(|event| event.estimated_json_encoded_size_of())
            .sum::<JsonSize>();

        let (written, json_size) = encoding.encode_input(input, &mut writer).unwrap();
        assert_eq!(written, 51);

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "{\"key\":\"value1\"}\n{\"key\":\"value2\"}\n{\"key\":\"value3\"}\n"
        );
        assert_eq!(CountByteSize(3, input_json_size), json_size.size().unwrap());
    }

    #[test]
    fn test_encode_event_json() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<()>::new(JsonSerializerConfig::default().build().into()),
        );

        let mut writer = Vec::new();
        let input = Event::Log(LogEvent::from(BTreeMap::from([(
            KeyString::from("key"),
            Value::from("value"),
        )])));
        let input_json_size = input.estimated_json_encoded_size_of();

        let (written, json_size) = encoding.encode_input(input, &mut writer).unwrap();
        assert_eq!(written, 15);

        assert_eq!(String::from_utf8(writer).unwrap(), r#"{"key":"value"}"#);
        assert_eq!(CountByteSize(1, input_json_size), json_size.size().unwrap());
    }

    #[test]
    fn test_encode_event_text() {
        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<()>::new(TextSerializerConfig::default().build().into()),
        );

        let mut writer = Vec::new();
        let input = Event::Log(LogEvent::from(BTreeMap::from([(
            KeyString::from("message"),
            Value::from("value"),
        )])));
        let input_json_size = input.estimated_json_encoded_size_of();

        let (written, json_size) = encoding.encode_input(input, &mut writer).unwrap();
        assert_eq!(written, 5);

        assert_eq!(String::from_utf8(writer).unwrap(), r"value");
        assert_eq!(CountByteSize(1, input_json_size), json_size.size().unwrap());
    }

    fn test_data_dir() -> PathBuf {
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("tests/data/protobuf")
    }

    #[test]
    fn test_encode_batch_protobuf_single() {
        let message_raw = std::fs::read(test_data_dir().join("test_proto.pb")).unwrap();
        let input_proto_size = message_raw.len();

        // default LengthDelimitedCoderOptions.length_field_length is 4
        let mut buf = BytesMut::with_capacity(64);
        buf.reserve(4 + input_proto_size);
        buf.put_uint(input_proto_size as u64, 4);
        buf.extend_from_slice(&message_raw[..]);
        let expected_bytes = buf.freeze();

        let config = ProtobufSerializerConfig {
            protobuf: ProtobufSerializerOptions {
                desc_file: test_data_dir().join("test_proto.desc"),
                message_type: "test_proto.User".to_string(),
            },
        };

        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                LengthDelimitedEncoder::default().into(),
                config.build().unwrap().into(),
            ),
        );

        let mut writer = Vec::new();
        let input = vec![Event::Log(LogEvent::from(BTreeMap::from([
            (KeyString::from("id"), Value::from("123")),
            (KeyString::from("name"), Value::from("Alice")),
            (KeyString::from("age"), Value::from(30)),
            (
                KeyString::from("emails"),
                Value::from(vec!["alice@example.com", "alice@work.com"]),
            ),
        ])))];

        let input_json_size = input
            .iter()
            .map(|event| event.estimated_json_encoded_size_of())
            .sum::<JsonSize>();

        let (written, size) = encoding.encode_input(input, &mut writer).unwrap();

        assert_eq!(input_proto_size, 49);
        assert_eq!(written, input_proto_size + 4);
        assert_eq!(CountByteSize(1, input_json_size), size.size().unwrap());
        assert_eq!(Bytes::copy_from_slice(&writer), expected_bytes);
    }

    #[test]
    fn test_encode_batch_protobuf_multiple() {
        let message_raw = std::fs::read(test_data_dir().join("test_proto.pb")).unwrap();
        let messages = vec![message_raw.clone(), message_raw.clone()];
        let total_input_proto_size: usize = messages.iter().map(|m| m.len()).sum();

        let mut buf = BytesMut::with_capacity(128);
        for message in messages {
            // default LengthDelimitedCoderOptions.length_field_length is 4
            buf.reserve(4 + message.len());
            buf.put_uint(message.len() as u64, 4);
            buf.extend_from_slice(&message[..]);
        }
        let expected_bytes = buf.freeze();

        let config = ProtobufSerializerConfig {
            protobuf: ProtobufSerializerOptions {
                desc_file: test_data_dir().join("test_proto.desc"),
                message_type: "test_proto.User".to_string(),
            },
        };

        let encoding = (
            Transformer::default(),
            crate::codecs::Encoder::<Framer>::new(
                LengthDelimitedEncoder::default().into(),
                config.build().unwrap().into(),
            ),
        );

        let mut writer = Vec::new();
        let input = vec![
            Event::Log(LogEvent::from(BTreeMap::from([
                (KeyString::from("id"), Value::from("123")),
                (KeyString::from("name"), Value::from("Alice")),
                (KeyString::from("age"), Value::from(30)),
                (
                    KeyString::from("emails"),
                    Value::from(vec!["alice@example.com", "alice@work.com"]),
                ),
            ]))),
            Event::Log(LogEvent::from(BTreeMap::from([
                (KeyString::from("id"), Value::from("123")),
                (KeyString::from("name"), Value::from("Alice")),
                (KeyString::from("age"), Value::from(30)),
                (
                    KeyString::from("emails"),
                    Value::from(vec!["alice@example.com", "alice@work.com"]),
                ),
            ]))),
        ];

        let input_json_size: JsonSize = input
            .iter()
            .map(|event| event.estimated_json_encoded_size_of())
            .sum();

        let (written, size) = encoding.encode_input(input, &mut writer).unwrap();

        assert_eq!(total_input_proto_size, 49 * 2);
        assert_eq!(written, total_input_proto_size + 8);
        assert_eq!(CountByteSize(2, input_json_size), size.size().unwrap());
        assert_eq!(Bytes::copy_from_slice(&writer), expected_bytes);
    }
}
