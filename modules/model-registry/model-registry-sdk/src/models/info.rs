// Created: 2026-05-06 by Constructor Tech
// Updated: 2026-05-07 by Constructor Tech
//! [`ModelInfoV1<P>`] — provider-independent fields, the user-facing
//! [`DefaultInferenceParametersV1`], the flat per-model override fields
//! (`allow_parameter_override`, `allow_extra_params`), and the typed
//! `provider_settings: P` payload.
//!
//! Declared as the GTS base schema via [`struct_to_gts_schema`]; concrete
//! provider-settings types (e.g. `OpenAiSettingsV1`, shipped in
//! [`crate::models::providers`]) declare themselves as GTS leaves with
//! `base = ModelInfoV1`. The set of provider leaves is open-ended.
//!
//! `P` defaults to [`crate::models::RawProviderSettings`] (a transparent
//! newtype around `serde_json::Value`) so heterogeneous lists carry an
//! opaque JSON blob; consumers route on [`ModelInfoV1::gts_type`] and
//! narrow to a typed view via [`crate::models::Model::try_into_typed`].

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use gts_macros::struct_to_gts_schema;

use crate::models::{
    ContextWindow, DefaultInferenceParametersV1, ModelCapabilities, ModelPerformance,
    RawProviderSettings, SupportedApi,
};

/// Complete model information: provider-independent metadata, capabilities,
/// the context window, performance, the user-facing default inference
/// parameters, the flat per-model override fields, and the
/// provider-specific `provider_settings` payload.
///
/// `P` defaults to [`RawProviderSettings`] (a transparent newtype around
/// `serde_json::Value`) so heterogeneous lists (e.g. `list_tenant_models`)
/// carry an opaque JSON blob; consumers route on [`ModelInfoV1::gts_type`]
/// and narrow to a typed view via [`crate::models::Model::try_into_typed`].
///
/// # GTS schema
///
/// - **`schema_id`**: `gts.cf.genai.model.info.v1~`
/// - **base**: yes (root envelope; provider-specific leaves chain off it)
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.cf.genai.model.info.v1~",
    description = "Generic model info envelope: provider-independent metadata + provider_settings JSON payload",
    properties = "gts_type,display_name,family,vendor,supported_api,provider_model_id,capabilities,context_window,default_parameters"
)]
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInfoV1<P: gts::GtsSchema = RawProviderSettings> {
    // ── GTS schema identity ───────────────────────────────────────────
    /// Full GTS schema chain identifying this model's settings shape — e.g.
    /// `gts.cf.genai.model.info.v1~cf.genai._.openai.v1~`. Mirrors
    /// `Provider.gts_type` and is the canonical key for resolving the
    /// concrete shape of `provider_settings` (which is a raw JSON blob by
    /// default — see [`crate::models::RawProviderSettings`]).
    pub gts_type: gts::GtsSchemaId,

    // ── Display / discovery ────────────────────────────────────────────
    /// Display name shown in UI.
    pub display_name: String,
    pub description: Option<String>,
    /// Model family (e.g. `"gpt-4"`, `"claude"`, `"llama"`).
    pub family: Option<String>,
    /// Model vendor — the organization that produced the model weights
    /// (e.g. `"OpenAI"`, `"Meta"`). Free-form string; independent of which
    /// provider serves the model.
    pub vendor: Option<String>,
    /// Deployment region (e.g. `"us-east-1"`, `"eu-west-1"`).
    pub region: Option<String>,
    /// Infrastructure host (e.g. `"Azure"`, `"AWS Bedrock"`, `"self-hosted"`).
    pub hosted_by: Option<String>,
    /// When the model version was last released by the vendor.
    pub last_release_at: Option<DateTime<Utc>>,
    /// Informational reasoning level label (e.g. `"high"`, `"medium"`).
    /// Display-only — not used for routing or parameter selection.
    pub reasoning_level: Option<String>,
    /// Model version string.
    pub version: Option<String>,
    /// Display order in model picker / lists.
    pub sort_order: Option<i32>,
    /// URL to model icon.
    pub icon: Option<String>,
    /// Human-readable cost multiplier label (e.g. `"1x"`, `"3x"`).
    pub multiplier_display: Option<String>,
    /// Estimated performance characteristics.
    pub performance: ModelPerformance,
    /// Last-resort escape hatch for deployment-specific metadata; typed
    /// fields on `provider_settings` are preferred.
    pub additional_info: HashMap<String, serde_json::Value>,

    // ── Promoted from the old `ApiResolution` ─────────────────────────
    /// Which API kinds this model exposes (completion, embedding).
    /// Promoted to common so consumers can filter on completion vs
    /// embedding without unwrapping the variant.
    pub supported_api: Vec<SupportedApi>,
    /// Provider's model identifier — used both in `canonical_id`
    /// (`{provider_slug}::{provider_model_id}`) and sent to the provider in
    /// API requests. Promoted to common so the catalog UI / alias logic
    /// doesn't have to reach into `provider_settings`.
    pub provider_model_id: String,

    // ── Capabilities ───────────────────────────────────────────────────
    /// What the model can do.
    pub capabilities: ModelCapabilities,
    /// Capabilities that are administratively disabled. Flags set to `true`
    /// indicate the corresponding capability is disabled for this model.
    pub disabled_capabilities: ModelCapabilities,
    /// Token limits.
    pub context_window: ContextWindow,

    // ── User-facing defaults & override policy ────────────────────────
    /// User-facing default inference parameters; mirrors the inference-knob
    /// subset of the Open Responses request schema
    /// (`gts.cf.llmgw.core.create_response_body.v1~`). Distinct from any
    /// provider-wire defaults living on `provider_settings`.
    pub default_parameters: DefaultInferenceParametersV1,
    /// Whether callers may override `default_parameters` per-request. Flat
    /// field on the envelope (no wrapper struct).
    pub allow_parameter_override: bool,
    /// Which extra (non-default) parameter names callers may pass alongside
    /// the request. Flat field on the envelope.
    pub allow_extra_params: Vec<String>,

    // ── Provider-specific payload ──────────────────────────────────────
    /// Provider-specific connection routing, provider-wire default
    /// parameters, and token pricing.
    pub provider_settings: P,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        MediaCapability, OpenAiCost, OpenAiEndpoint, OpenAiReasoningEffort, OpenAiServiceTier,
        OpenAiSettingsV1, RawProviderSettings, ReasoningCapability, WebSearchCapability,
    };

    fn sample_capabilities() -> ModelCapabilities {
        ModelCapabilities {
            vision: MediaCapability {
                enabled: true,
                supported_mime_types: vec!["image/png".into(), "image/jpeg".into()],
            },
            reasoning: ReasoningCapability {
                effort: false,
                toggle: false,
                resume: false,
                budget: false,
            },
            function_calling: true,
            response_schema: true,
            streaming: true,
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

    fn sample_no_capabilities() -> ModelCapabilities {
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

    fn sample_openai() -> OpenAiSettingsV1 {
        OpenAiSettingsV1 {
            oagw_alias: "openai-prod".into(),
            endpoint_kind: OpenAiEndpoint::ChatCompletions,
            organization: None,
            project: None,
            temperature: Some(0.7),
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            top_logprobs: None,
            service_tier: Some(OpenAiServiceTier::Default),
            prompt_cache_retention: None,
            reasoning_effort: Some(OpenAiReasoningEffort::Medium),
            reasoning_summary: None,
            verbosity: None,
            parallel_tool_calls: None,
            store: None,
            response_format: None,
            max_tokens: Some(4096),
            max_completion_tokens: None,
            n: None,
            stop: None,
            seed: None,
            logprobs: None,
            max_output_tokens: None,
            max_tool_calls: None,
            truncation: None,
            encoding_format: None,
            dimensions: None,
            cost: OpenAiCost {
                input_per_1k_micro: Some(2_500_000),
                cached_input_per_1k_micro: Some(1_250_000),
                output_per_1k_micro: Some(10_000_000),
                long_context_input_per_1k_micro: None,
                long_context_cached_input_per_1k_micro: None,
                long_context_output_per_1k_micro: None,
                long_context_threshold_tokens: None,
                web_search_per_1k_calls_micro: None,
                file_search_per_1k_calls_micro: None,
            },
        }
    }

    fn typed_info() -> ModelInfoV1<OpenAiSettingsV1> {
        ModelInfoV1 {
            gts_type: gts::GtsSchemaId::new("gts.cf.genai.model.info.v1~cf.genai._.openai.v1~"),
            display_name: "GPT-4o".into(),
            description: Some("Multimodal flagship model".into()),
            family: Some("gpt-4".into()),
            vendor: Some("OpenAI".into()),
            region: None,
            hosted_by: None,
            last_release_at: None,
            reasoning_level: None,
            version: Some("2024-08-06".into()),
            sort_order: Some(1),
            icon: Some("https://example.com/gpt4o.svg".into()),
            multiplier_display: Some("1x".into()),
            performance: ModelPerformance {
                response_latency_ms: Some(500),
                tokens_per_second: Some(100),
            },
            additional_info: HashMap::new(),
            supported_api: vec![SupportedApi::Completion],
            provider_model_id: "gpt-4o".into(),
            capabilities: sample_capabilities(),
            disabled_capabilities: sample_no_capabilities(),
            context_window: ContextWindow {
                max_input_tokens: 128_000,
                max_output_tokens: Some(16_384),
                output_vector_size: None,
            },
            default_parameters: DefaultInferenceParametersV1::default(),
            allow_parameter_override: false,
            allow_extra_params: Vec::new(),
            provider_settings: sample_openai(),
        }
    }

    #[test]
    fn typed_info_has_promoted_fields() {
        let info = typed_info();
        assert_eq!(info.supported_api, vec![SupportedApi::Completion]);
        assert_eq!(info.provider_model_id, "gpt-4o");
    }

    #[test]
    fn typed_info_carries_flat_provider_settings() {
        let info = typed_info();
        assert_eq!(info.provider_settings.oagw_alias, "openai-prod");
        assert_eq!(info.provider_settings.max_tokens, Some(4096));
        assert_eq!(info.provider_settings.temperature, Some(0.7));
        // Provider family is identified via gts_type, not via a kind() method.
        assert_eq!(
            info.gts_type.as_ref(),
            "gts.cf.genai.model.info.v1~cf.genai._.openai.v1~"
        );
    }

    #[test]
    fn typed_info_carries_default_parameters_and_overrides() {
        let info = typed_info();
        // Default parameters default to all-None.
        assert!(info.default_parameters.temperature.is_none());
        assert!(info.default_parameters.text.is_none());
        // Override fields are flat.
        assert!(!info.allow_parameter_override);
        assert!(info.allow_extra_params.is_empty());
    }

    #[test]
    fn override_fields_can_be_populated() {
        let mut info = typed_info();
        info.allow_parameter_override = true;
        info.allow_extra_params = vec!["top_k".into(), "repetition_penalty".into()];
        assert!(info.allow_parameter_override);
        assert_eq!(info.allow_extra_params.len(), 2);
    }

    #[test]
    fn default_carrier_holds_raw_json() {
        // A `ModelInfoV1` (no type arg) defaults to RawProviderSettings —
        // a transparent newtype over `serde_json::Value`. Heterogeneous
        // catalogs ride this shape; consumers route on `gts_type`. This
        // case uses a *hypothetical* provider GTS id to demonstrate the
        // carrier's forward-compatibility with providers the SDK doesn't
        // ship today: operators can wire up routing without an SDK release.
        let raw_info: ModelInfoV1 = ModelInfoV1 {
            gts_type: gts::GtsSchemaId::new(
                "gts.cf.genai.model.info.v1~example.genai._.experimental.v1~",
            ),
            display_name: "Experimental Provider Model".into(),
            description: None,
            family: None,
            vendor: Some("Example Org".into()),
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
            provider_model_id: "experimental-1".into(),
            capabilities: sample_no_capabilities(),
            disabled_capabilities: sample_no_capabilities(),
            context_window: ContextWindow {
                max_input_tokens: 32_000,
                max_output_tokens: Some(4096),
                output_vector_size: None,
            },
            default_parameters: DefaultInferenceParametersV1::default(),
            allow_parameter_override: false,
            allow_extra_params: Vec::new(),
            provider_settings: RawProviderSettings(serde_json::json!({
                "oagw_alias": "example-prod",
                "custom_routing_key": "experimental-1",
                "max_tokens": 4096,
                "cost": {},
            })),
        };
        assert_eq!(
            raw_info.gts_type.as_ref(),
            "gts.cf.genai.model.info.v1~example.genai._.experimental.v1~"
        );
        assert_eq!(raw_info.provider_model_id, "experimental-1");
        // The raw payload is just JSON; consumers narrow via try_into_typed
        // (covered in entity.rs tests).
        assert!(raw_info.provider_settings.as_value().is_object());
    }

    #[test]
    fn additional_info_is_escape_hatch() {
        let mut info = typed_info();
        info.additional_info.insert(
            "architecture".into(),
            serde_json::Value::String("transformer".into()),
        );
        assert_eq!(info.additional_info.len(), 1);
    }

    #[test]
    fn embedding_model_no_output_tokens() {
        let mut info = typed_info();
        info.supported_api = vec![SupportedApi::Embedding];
        info.context_window.max_output_tokens = None;
        info.context_window.output_vector_size = Some(3072);
        assert!(info.context_window.max_output_tokens.is_none());
        assert_eq!(info.context_window.output_vector_size, Some(3072));
    }

    #[test]
    fn ui_fields_round_trip() {
        let info = typed_info();
        assert_eq!(info.sort_order, Some(1));
        assert_eq!(info.icon.as_deref(), Some("https://example.com/gpt4o.svg"));
        assert_eq!(info.multiplier_display.as_deref(), Some("1x"));
    }

    #[test]
    fn disabled_capabilities_independent_from_capabilities() {
        let mut info = typed_info();
        info.disabled_capabilities.web_search.enabled = true;
        info.disabled_capabilities.code_interpreter = true;
        assert!(info.disabled_capabilities.web_search.enabled);
        assert!(info.disabled_capabilities.code_interpreter);
        assert!(!info.disabled_capabilities.vision.enabled);
        // capabilities unaffected
        assert!(info.capabilities.vision.enabled);
    }

    #[test]
    fn disabled_capabilities_subtractive_mime_types() {
        // For MediaCapability fields, populating disabled_capabilities'
        // supported_mime_types means "the model supports these but the admin
        // has disabled them" — used as a subtractive overlay over the parent
        // capability's enabled list.
        let mut info = typed_info();
        info.capabilities.image_generation = MediaCapability {
            enabled: true,
            supported_mime_types: vec!["image/png".into(), "image/svg+xml".into()],
        };
        info.disabled_capabilities.image_generation = MediaCapability {
            enabled: false,
            supported_mime_types: vec!["image/svg+xml".into()],
        };
        assert!(info.capabilities.image_generation.enabled);
        // The capability itself is not disabled outright …
        assert!(!info.disabled_capabilities.image_generation.enabled);
        // … but SVG is on the deny-list.
        assert_eq!(
            info.disabled_capabilities
                .image_generation
                .supported_mime_types,
            vec!["image/svg+xml"]
        );
    }
}
