// Created: 2026-05-06 by Constructor Tech
//! [`ProviderSettings`] marker trait for typed provider-settings types
//! (e.g. `OpenAiSettingsV1`; the shipped set lives in
//! [`crate::models::providers`] and is open-ended), [`RawProviderSettings`]
//! (the `serde_json::Value` carrier used by the default `Model` /
//! `ModelInfoV1`), and the [`ProviderSchemaMismatch`] error returned by
//! typed-narrowing helpers.
//!
//! There is **no** tagged-enum carrier here — `Model<P>` defaults to
//! `Model<RawProviderSettings>` (a transparent newtype around
//! `serde_json::Value`), i.e. the provider settings ride as a raw JSON
//! blob until the consumer narrows to a typed view via
//! [`crate::models::Model::try_into_typed`]. Resolution is by GTS schema
//! id: each typed settings type's `GtsSchema::SCHEMA_ID` is matched
//! against the model's `info.gts_type` before the JSON value is
//! deserialized into the typed shape.

use gts::GtsSchema;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Marker trait implemented by every typed provider-settings type shipped
/// in [`crate::models::providers`] (e.g. `OpenAiSettingsV1`).
///
/// Acts purely as documentation that a type is a typed provider-settings
/// payload — there are no required methods. Provider family identification
/// is done via `ModelInfoV1::gts_type` (a [`gts::GtsSchemaId`]).
pub trait ProviderSettings:
    std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static + GtsSchema
{
}

// ---------------------------------------------------------------------------
// Raw JSON carrier (default `P` for `Model<P>` / `ModelInfoV1<P>`)
// ---------------------------------------------------------------------------

/// Transparent newtype around [`serde_json::Value`] used as the default
/// payload type for [`crate::models::Model`] / [`crate::models::ModelInfoV1`].
///
/// The wrapper is purely there to satisfy the `gts::GtsSchema` bound the
/// `struct_to_gts_schema` macro injects on `ModelInfoV1<P>` — semantically
/// it's just a `serde_json::Value`. Serialization is `#[serde(transparent)]`
/// so on-the-wire the field is a bare JSON value (no envelope).
///
/// Consumers narrow to a typed `Model<OpenAiSettingsV1>` etc. via
/// [`crate::models::Model::try_into_typed`].
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct RawProviderSettings(pub serde_json::Value);

impl RawProviderSettings {
    /// Borrow the inner JSON value.
    #[must_use]
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    /// Consume the wrapper and return the inner JSON value.
    #[must_use]
    pub fn into_value(self) -> serde_json::Value {
        self.0
    }
}

impl From<serde_json::Value> for RawProviderSettings {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl From<RawProviderSettings> for serde_json::Value {
    fn from(raw: RawProviderSettings) -> Self {
        raw.0
    }
}

impl GtsSchema for RawProviderSettings {
    const SCHEMA_ID: &'static str = "";

    fn gts_schema_with_refs() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "description": "Raw provider settings JSON; the concrete schema \
                            is identified by ModelInfoV1::gts_type."
        })
    }
}

// ---------------------------------------------------------------------------
// Typed-narrowing error
// ---------------------------------------------------------------------------

/// Returned by [`crate::models::Model::try_into_typed`] when the GTS schema
/// id on the source doesn't match the target type's `SCHEMA_ID`, or when the
/// JSON payload fails to deserialize into the target shape.
#[derive(Debug, thiserror::Error)]
pub enum ProviderSchemaMismatch {
    /// `info.gts_type` doesn't match the target type's `GtsSchema::SCHEMA_ID`.
    #[error("provider schema id mismatch: expected `{expected}`, got `{actual}`")]
    SchemaId {
        /// `<TargetSettings>::SCHEMA_ID`.
        expected: String,
        /// The schema id we read off `info.gts_type`.
        actual: String,
    },

    /// JSON payload couldn't be deserialized into the target shape.
    #[error("provider settings deserialization failed: {0}")]
    Deserialize(#[from] serde_json::Error),
}
