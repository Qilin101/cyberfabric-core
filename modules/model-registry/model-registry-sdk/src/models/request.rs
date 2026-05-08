// Created: 2026-05-06 by Constructor Tech
//! Transport-agnostic request DTOs for the Model Registry SDK.
//!
//! These are NOT REST DTOs — they sit on the SDK trait (`ModelRegistryClientV1`)
//! and are serialized into transport (REST/gRPC) by the module crate.

use crate::models::{
    ApprovalStatus, ContextWindow, DefaultInferenceParametersV1, LifecycleStatus,
    ModelCapabilities, ModelInfoV1, ModelPerformance, ProviderStatus, RawProviderSettings,
};

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
// CreateModelRequest (P1 — manual model management)
// ---------------------------------------------------------------------------

/// Request for manually creating a model in the catalog (P1 manual model
/// management; `cpt-cf-model-registry-fr-manual-model-management`).
///
/// The `canonical_id` is derived from `provider_slug` + `info.provider_model_id`
/// — both are immutable after creation. Provider must exist for the caller's
/// tenant (or be inherited from an ancestor).
///
/// **Phase semantics for `approval_status`**:
/// - **P1**: written directly to `ModelApproval` by Model Registry — defaults
///   to [`ApprovalStatus::Pending`]; admins can pass [`ApprovalStatus::Approved`]
///   to approve in the same call as a convenience.
/// - **P2 onward**: registered as an approvable resource with the Approval
///   Service; the `approval_status` field initiates the workflow rather than
///   writing directly.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateModelRequest {
    /// Provider slug (1-64 chars, lowercase alphanumeric + hyphen). Combined
    /// with `info.provider_model_id` to form the `canonical_id`.
    pub provider_slug: String,
    /// Lifecycle status (Production / Preview / Experimental / …).
    pub lifecycle_status: LifecycleStatus,
    /// Optional initial approval status. `None` ⇒ defaults to
    /// [`ApprovalStatus::Pending`].
    pub approval_status: Option<ApprovalStatus>,
    /// Model info — display, capabilities, limits, default parameters, and
    /// the provider-specific settings payload (raw JSON typed by
    /// `info.gts_type`).
    pub info: ModelInfoV1<RawProviderSettings>,
}

// ---------------------------------------------------------------------------
// UpdateModelRequest (P1 — manual model management; PATCH semantics)
// ---------------------------------------------------------------------------

/// Request for updating an existing model. Only non-`None` fields are applied.
///
/// **Immutable after creation** — these fields are NOT in this struct:
/// `canonical_id`, `provider_slug`, `info.provider_model_id`, `info.gts_type`.
/// To switch a model's provider settings shape, soft-delete and recreate.
///
/// **Approval status changes** also flow through this PATCH endpoint (see
/// `cpt-cf-model-registry-fr-manual-model-management` in DESIGN §1.2):
/// - **P1**: status writes go directly to `ModelApproval`.
/// - **P2 onward**: status writes route through the Approval Service; other
///   field updates remain direct DB writes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateModelRequest {
    // ── Status ────────────────────────────────────────────────────────
    /// Approval status (`approved` / `rejected` / `revoked` / `pending`).
    pub approval_status: Option<ApprovalStatus>,
    /// Lifecycle status (e.g. promote `Experimental` → `Production`, or mark
    /// `Sunset`). Setting to `Deprecated` here is equivalent to the soft-delete
    /// path; prefer [`crate::api::ModelRegistryClientV1::delete_model`].
    pub lifecycle_status: Option<LifecycleStatus>,

    // ── Display / discovery ───────────────────────────────────────────
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub family: Option<String>,
    pub vendor: Option<String>,
    pub region: Option<String>,
    pub hosted_by: Option<String>,
    pub reasoning_level: Option<String>,
    pub version: Option<String>,
    pub sort_order: Option<i32>,
    pub icon: Option<String>,
    pub multiplier_display: Option<String>,
    pub performance: Option<ModelPerformance>,

    // ── Capabilities & limits (full replacement) ──────────────────────
    /// Replace `info.capabilities` wholesale.
    pub capabilities: Option<ModelCapabilities>,
    /// Replace `info.disabled_capabilities` wholesale.
    pub disabled_capabilities: Option<ModelCapabilities>,
    /// Replace `info.context_window` wholesale.
    pub context_window: Option<ContextWindow>,

    // ── Defaults & override policy ────────────────────────────────────
    /// Replace `info.default_parameters` wholesale.
    pub default_parameters: Option<DefaultInferenceParametersV1>,
    pub allow_parameter_override: Option<bool>,
    pub allow_extra_params: Option<Vec<String>>,

    // ── Provider-specific payload ─────────────────────────────────────
    /// Replace `info.provider_settings` wholesale. The shape MUST validate
    /// against the model's existing `info.gts_type` (which is immutable).
    pub provider_settings: Option<RawProviderSettings>,
}

// ---------------------------------------------------------------------------
// CreateAliasRequest (P3)
// ---------------------------------------------------------------------------

/// Request for creating a model alias (P3 alias management;
/// `cpt-cf-model-registry-fr-alias-management`).
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

    // ---- Manual model management (P1) ----

    use std::collections::HashMap;

    use crate::models::{
        ContextWindow, DefaultInferenceParametersV1, MediaCapability, ModelCapabilities,
        ModelInfoV1, ModelPerformance, ReasoningCapability, SupportedApi, WebSearchCapability,
    };

    fn empty_capabilities() -> ModelCapabilities {
        ModelCapabilities {
            vision: MediaCapability::default(),
            reasoning: ReasoningCapability {
                effort: false,
                toggle: false,
                resume: false,
                budget: false,
            },
            function_calling: false,
            response_schema: false,
            streaming: false,
            file_input: MediaCapability::default(),
            image_generation: MediaCapability::default(),
            audio_input: MediaCapability::default(),
            audio_output: MediaCapability::default(),
            code_interpreter: false,
            web_search: WebSearchCapability {
                enabled: false,
                allowed_domains: false,
                excluded_domains: false,
            },
        }
    }

    fn raw_info(provider_model_id: &str, gts_type: &str) -> ModelInfoV1<RawProviderSettings> {
        ModelInfoV1 {
            gts_type: gts::GtsSchemaId::new(gts_type),
            display_name: format!("Model {provider_model_id}"),
            description: None,
            family: None,
            vendor: None,
            region: None,
            hosted_by: None,
            last_release_at: None,
            reasoning_level: None,
            version: None,
            sort_order: None,
            icon: None,
            multiplier_display: None,
            performance: ModelPerformance {
                response_latency_ms: None,
                tokens_per_second: None,
            },
            additional_info: HashMap::new(),
            supported_api: vec![SupportedApi::Completion],
            provider_model_id: provider_model_id.into(),
            capabilities: empty_capabilities(),
            disabled_capabilities: empty_capabilities(),
            context_window: ContextWindow {
                max_input_tokens: 8192,
                max_output_tokens: Some(4096),
                output_vector_size: None,
            },
            default_parameters: DefaultInferenceParametersV1::default(),
            allow_parameter_override: false,
            allow_extra_params: Vec::new(),
            provider_settings: RawProviderSettings(serde_json::json!({})),
        }
    }

    #[test]
    fn create_model_request_defaults_status_to_none() {
        let req = CreateModelRequest {
            provider_slug: "openai".into(),
            lifecycle_status: LifecycleStatus::Production,
            approval_status: None,
            info: raw_info("gpt-4o", "gts.cf.genai.model.info.v1~cf.genai._.openai.v1~"),
        };
        assert_eq!(req.provider_slug, "openai");
        assert_eq!(req.info.provider_model_id, "gpt-4o");
        assert!(matches!(req.lifecycle_status, LifecycleStatus::Production));
        // None ⇒ server defaults to Pending; explicit Approved is the
        // admin-convenience path.
        assert!(req.approval_status.is_none());
    }

    #[test]
    fn create_model_request_with_initial_approval() {
        let req = CreateModelRequest {
            provider_slug: "anthropic".into(),
            lifecycle_status: LifecycleStatus::Preview,
            approval_status: Some(ApprovalStatus::Approved),
            info: raw_info(
                "claude-sonnet-4-6",
                "gts.cf.genai.model.info.v1~cf.genai._.anthropic.v1~",
            ),
        };
        assert_eq!(req.approval_status, Some(ApprovalStatus::Approved));
    }

    #[test]
    fn update_model_request_default_is_empty() {
        let req = UpdateModelRequest::default();
        assert!(req.approval_status.is_none());
        assert!(req.lifecycle_status.is_none());
        assert!(req.display_name.is_none());
        assert!(req.capabilities.is_none());
        assert!(req.context_window.is_none());
        assert!(req.default_parameters.is_none());
        assert!(req.allow_parameter_override.is_none());
        assert!(req.allow_extra_params.is_none());
        assert!(req.provider_settings.is_none());
    }

    #[test]
    fn update_model_request_status_only_patch() {
        // Common case: admin only flips approval status. P1 writes directly
        // to ModelApproval; P2 routes through Approval Service. Both phases
        // accept this same struct.
        let req = UpdateModelRequest {
            approval_status: Some(ApprovalStatus::Revoked),
            ..Default::default()
        };
        assert_eq!(req.approval_status, Some(ApprovalStatus::Revoked));
        assert!(req.lifecycle_status.is_none());
        assert!(req.display_name.is_none());
        assert!(req.provider_settings.is_none());
    }

    #[test]
    fn update_model_request_capabilities_full_replacement() {
        let mut caps = empty_capabilities();
        caps.streaming = true;
        let req = UpdateModelRequest {
            capabilities: Some(caps),
            ..Default::default()
        };
        assert!(req.capabilities.unwrap().streaming);
    }

    #[test]
    fn update_model_request_provider_settings_replacement() {
        let req = UpdateModelRequest {
            provider_settings: Some(RawProviderSettings(serde_json::json!({
                "oagw_alias": "openai-prod-replaced",
                "temperature": 0.2,
            }))),
            ..Default::default()
        };
        let payload = req.provider_settings.unwrap();
        assert_eq!(
            payload
                .as_value()
                .get("oagw_alias")
                .and_then(|v| v.as_str()),
            Some("openai-prod-replaced")
        );
    }
}
