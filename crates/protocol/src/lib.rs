#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

pub type ClinicId = Uuid;
pub type DeviceId = Uuid;
pub type UserId = Uuid;
pub type EntityId = Uuid;
pub type OperationId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub entity_type: String,
    pub entity_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub op_id: OperationId,
    pub clinic_id: ClinicId,
    pub device_id: DeviceId,
    pub user_id: UserId,
    pub entity: EntityRef,
    pub op_type: String,
    pub device_time: OffsetDateTime,
    pub device_seq: u64,
    pub schema_version: u32,
    pub payload: serde_json::Value,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OperationValidationError {
    #[error("op_type must not be empty")]
    EmptyOpType,
    #[error("entity_type must not be empty")]
    EmptyEntityType,
}

impl Operation {
    pub fn validate(&self) -> Result<(), OperationValidationError> {
        if self.op_type.trim().is_empty() {
            return Err(OperationValidationError::EmptyOpType);
        }
        if self.entity.entity_type.trim().is_empty() {
            return Err(OperationValidationError::EmptyEntityType);
        }
        Ok(())
    }
}

/// A server-issued monotonic cursor for `/sync/pull`.
///
/// Encoded as a string in JSON to avoid JS integer pitfalls.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor(pub u64);

impl Serialize for Cursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Cursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let n = s
            .parse::<u64>()
            .map_err(|_| serde::de::Error::custom("invalid cursor"))?;
        Ok(Cursor(n))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushRequest {
    pub ops: Vec<Operation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushResponse {
    pub accepted: u64,
    pub duplicate: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PullResponse {
    pub ops: Vec<Operation>,
    pub next_cursor: Option<Cursor>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn cursor_json_roundtrip() {
        let c = Cursor(123);
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"123\"");
        let decoded: Cursor = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, c);
    }

    #[test]
    fn operation_validation_rejects_empty_fields() {
        let op = Operation {
            op_id: Uuid::now_v7(),
            clinic_id: Uuid::now_v7(),
            device_id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            entity: EntityRef {
                entity_type: "  ".into(),
                entity_id: Uuid::now_v7(),
            },
            op_type: "".into(),
            device_time: OffsetDateTime::now_utc(),
            device_seq: 1,
            schema_version: 1,
            payload: serde_json::json!({}),
        };

        assert_eq!(op.validate(), Err(OperationValidationError::EmptyOpType));
    }

    proptest! {
        #[test]
        fn cursor_roundtrip_prop(n in any::<u64>()) {
            let c = Cursor(n);
            let json = serde_json::to_string(&c).unwrap();
            let decoded: Cursor = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(decoded, c);
        }
    }
}
