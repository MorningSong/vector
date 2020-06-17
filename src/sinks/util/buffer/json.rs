use crate::sinks::util::{Batch, BatchSettings, PushResult};
use serde_json::value::{to_raw_value, RawValue, Value};

pub type BoxedRawValue = Box<RawValue>;

/// A `batch` implementation for storing an array of json
/// values.
#[derive(Debug)]
pub struct JsonArrayBuffer {
    buffer: Vec<BoxedRawValue>,
    total_bytes: usize,
    max_size: usize,
}

impl JsonArrayBuffer {
    pub fn new(settings: BatchSettings) -> Self {
        Self {
            buffer: Vec::new(),
            total_bytes: 0,
            max_size: settings.size,
        }
    }
}

impl Batch for JsonArrayBuffer {
    type Input = Value;
    type Output = Vec<BoxedRawValue>;

    fn push(&mut self, item: Self::Input) -> PushResult<Self::Input> {
        let raw_item = to_raw_value(&item).expect("Value should be valid json");
        let new_len = self.total_bytes + raw_item.get().len();
        if new_len > self.max_size {
            PushResult::Full(item)
        } else {
            self.total_bytes += new_len;
            self.buffer.push(raw_item);
            PushResult::Ok
        }
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    fn fresh(&self) -> Self {
        Self {
            buffer: Vec::new(),
            total_bytes: 0,
            max_size: self.max_size,
        }
    }

    fn finish(self) -> Self::Output {
        self.buffer
    }

    fn num_items(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::super::PushResult;
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn multi_object_array() {
        let batch = BatchSettings {
            size: 2,
            timeout: Duration::from_secs(9999),
        };
        let mut buffer = JsonArrayBuffer::new(batch);

        assert_eq!(
            buffer.push(json!({
                "key1": "value1"
            })),
            PushResult::Ok
        );

        assert_eq!(
            buffer.push(json!({
                "key2": "value2"
            })),
            PushResult::Ok
        );

        assert!(matches!(buffer.push(json!({})), PushResult::Full(_)));

        assert_eq!(buffer.num_items(), 2);
        assert_eq!(buffer.total_bytes, 34);

        let json = buffer.finish();

        let wrapped = serde_json::to_string(&json!({
            "arr": json,
        }))
        .unwrap();

        let expected = serde_json::to_string(&json!({
            "arr": [
                {
                    "key1": "value1"
                },
                {
                    "key2": "value2"
                },
            ]
        }))
        .unwrap();

        assert_eq!(wrapped, expected);
    }
}
