// Created: 2026-05-06 by Constructor Tech
//! Transport-agnostic request DTOs for the Model Registry SDK.
//!
//! These are NOT REST DTOs — they sit on the SDK trait (`ModelRegistryClientV1`)
//! and are serialized into transport (REST/gRPC) by the module crate.

use crate::models::ProviderStatus;

// ---------------------------------------------------------------------------
// CreateProviderRequest (builder pattern)
// ---------------------------------------------------------------------------

/// Request for registering a new provider. Construct via
/// [`CreateProviderRequest::builder`].
#[derive(Debug, Clone, PartialEq)]
pub struct CreateProviderRequest {
    slug: String,
    name: String,
    gts_type: String,
    oagw_alias: String,
    managed: bool,
    metadata: Option<serde_json::Value>,
    discovery_enabled: bool,
    discovery_interval_seconds: Option<u32>,
}

impl CreateProviderRequest {
    /// Start building a new request. All four fields are required.
    #[must_use]
    pub fn builder(
        slug: impl Into<String>,
        name: impl Into<String>,
        gts_type: impl Into<String>,
        oagw_alias: impl Into<String>,
    ) -> CreateProviderRequestBuilder {
        CreateProviderRequestBuilder {
            slug: slug.into(),
            name: name.into(),
            gts_type: gts_type.into(),
            oagw_alias: oagw_alias.into(),
            managed: false,
            metadata: None,
            discovery_enabled: false,
            discovery_interval_seconds: None,
        }
    }

    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn gts_type(&self) -> &str {
        &self.gts_type
    }

    #[must_use]
    pub fn oagw_alias(&self) -> &str {
        &self.oagw_alias
    }

    #[must_use]
    pub fn managed(&self) -> bool {
        self.managed
    }

    #[must_use]
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    #[must_use]
    pub fn discovery_enabled(&self) -> bool {
        self.discovery_enabled
    }

    #[must_use]
    pub fn discovery_interval_seconds(&self) -> Option<u32> {
        self.discovery_interval_seconds
    }
}

#[derive(Debug, Clone)]
pub struct CreateProviderRequestBuilder {
    slug: String,
    name: String,
    gts_type: String,
    oagw_alias: String,
    managed: bool,
    metadata: Option<serde_json::Value>,
    discovery_enabled: bool,
    discovery_interval_seconds: Option<u32>,
}

impl CreateProviderRequestBuilder {
    #[must_use]
    pub fn managed(mut self, managed: bool) -> Self {
        self.managed = managed;
        self
    }

    #[must_use]
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    #[must_use]
    pub fn discovery_enabled(mut self, enabled: bool) -> Self {
        self.discovery_enabled = enabled;
        self
    }

    #[must_use]
    pub fn discovery_interval_seconds(mut self, seconds: u32) -> Self {
        self.discovery_interval_seconds = Some(seconds);
        self
    }

    #[must_use]
    pub fn build(self) -> CreateProviderRequest {
        CreateProviderRequest {
            slug: self.slug,
            name: self.name,
            gts_type: self.gts_type,
            oagw_alias: self.oagw_alias,
            managed: self.managed,
            metadata: self.metadata,
            discovery_enabled: self.discovery_enabled,
            discovery_interval_seconds: self.discovery_interval_seconds,
        }
    }
}

// ---------------------------------------------------------------------------
// UpdateProviderRequest (PATCH semantics)
// ---------------------------------------------------------------------------

/// Request for updating a provider (PATCH semantics). Only non-`None` fields
/// are applied.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub oagw_alias: Option<String>,
    pub status: Option<ProviderStatus>,
    pub managed: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub discovery_enabled: Option<bool>,
    pub discovery_interval_seconds: Option<u32>,
}

// ---------------------------------------------------------------------------
// CreateAliasRequest (P2)
// ---------------------------------------------------------------------------

/// Request for creating a model alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAliasRequest {
    pub name: String,
    pub canonical_id: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_provider_request_builder() {
        let req = CreateProviderRequest::builder(
            "openai",
            "OpenAI",
            "gts.cf.genai.models.provider.v1~cf.genai._.openai.v1~",
            "openai-prod",
        )
        .managed(false)
        .discovery_enabled(true)
        .discovery_interval_seconds(3600)
        .build();

        assert_eq!(req.slug(), "openai");
        assert_eq!(req.name(), "OpenAI");
        assert_eq!(req.oagw_alias(), "openai-prod");
        assert!(req.discovery_enabled());
        assert_eq!(req.discovery_interval_seconds(), Some(3600));
        assert!(!req.managed());
    }

    #[test]
    fn create_provider_request_defaults() {
        let req = CreateProviderRequest::builder(
            "ollama",
            "Ollama Local",
            "gts.cf.genai.models.provider.v1~cf.genai.local.provider.v1~",
            "ollama-local",
        )
        .build();

        assert!(!req.managed());
        assert!(!req.discovery_enabled());
        assert_eq!(req.discovery_interval_seconds(), None);
        assert!(req.metadata().is_none());
    }

    #[test]
    fn update_provider_request_default_is_empty() {
        let req = UpdateProviderRequest::default();
        assert!(req.name.is_none());
        assert!(req.oagw_alias.is_none());
        assert!(req.status.is_none());
        assert!(req.managed.is_none());
        assert!(req.metadata.is_none());
        assert!(req.discovery_enabled.is_none());
        assert!(req.discovery_interval_seconds.is_none());
    }

    #[test]
    fn create_alias_request() {
        let req = CreateAliasRequest {
            name: "gpt4".into(),
            canonical_id: "openai::gpt-4o".into(),
        };
        assert_eq!(req.name, "gpt4");
        assert_eq!(req.canonical_id, "openai::gpt-4o");
    }
}
