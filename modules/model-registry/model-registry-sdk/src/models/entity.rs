// Created: 2026-05-06 by Constructor Tech
// Updated: 2026-05-07 by Constructor Tech
//! Domain entities — generic [`Model<P>`] plus the non-generic [`Provider`],
//! [`ProviderHealth`], [`Alias`], and [`DiscoveryResult`].

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{
    ApprovalStatus, LifecycleStatus, ModelInfoV1, ProviderHealthStatus, ProviderSchemaMismatch,
    ProviderStatus, RawProviderSettings,
};

// ---------------------------------------------------------------------------
// Model<P>
// ---------------------------------------------------------------------------

/// An AI model in the catalog, scoped to a tenant.
///
/// Generic over `P: gts::GtsSchema` so that consumers narrowed to a
/// specific provider can carry `Model<OpenAiSettingsV1>` etc. The default
/// `P = RawProviderSettings` (a transparent newtype around
/// `serde_json::Value`) is what the public
/// [`crate::api::ModelRegistryClientV1`] trait returns — the provider
/// settings ride as opaque JSON until the consumer narrows via
/// [`Model::try_into_typed`], which reads `info.gts_type` for dispatch.
#[derive(Debug, Clone, PartialEq)]
pub struct Model<P: gts::GtsSchema = RawProviderSettings> {
    pub id: Uuid,
    /// Format: `{provider_slug}::{provider_model_id}`.
    pub canonical_id: String,
    pub lifecycle_status: LifecycleStatus,
    pub approval_status: ApprovalStatus,
    /// All model information: display, capabilities, the promoted identity
    /// (`gts_type`, `supported_api`, `provider_model_id`), and the
    /// `provider_settings: P` payload.
    pub info: ModelInfoV1<P>,
}

impl Model<RawProviderSettings> {
    /// Narrow a raw-JSON-payload model to a typed view by validating
    /// `info.gts_type` against `Q::SCHEMA_ID` and deserializing
    /// `info.provider_settings` into `Q`.
    ///
    /// ```ignore
    /// use model_registry_sdk::{Model, OpenAiSettingsV1};
    ///
    /// let model: Model = client.get_tenant_model(&ctx, "openai::gpt-4o").await?;
    /// let typed: Model<OpenAiSettingsV1> = model.try_into_typed()?;
    /// // now `typed.info.provider_settings.parameters.temperature` is typed
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ProviderSchemaMismatch::SchemaId`] when `info.gts_type` doesn't
    ///   match `Q::SCHEMA_ID`.
    /// - [`ProviderSchemaMismatch::Deserialize`] when the JSON payload
    ///   can't be deserialized into `Q`.
    pub fn try_into_typed<Q>(self) -> Result<Model<Q>, ProviderSchemaMismatch>
    where
        Q: gts::GtsSchema,
        for<'de> Q: gts::GtsDeserialize<'de>,
    {
        let actual = self.info.gts_type.as_ref();
        let expected = <Q as gts::GtsSchema>::SCHEMA_ID;
        if actual != expected {
            return Err(ProviderSchemaMismatch::SchemaId {
                expected: expected.to_owned(),
                actual: actual.to_owned(),
            });
        }
        // Leaf settings (`base = ModelInfoV1`) impl `GtsDeserialize`, not
        // `serde::Deserialize` directly — the wrapper bridges the two.
        let wrapper: gts::GtsDeserializeWrapper<Q> =
            serde_json::from_value(self.info.provider_settings.into_value())?;
        let typed_settings = wrapper.0;
        Ok(Model {
            id: self.id,
            canonical_id: self.canonical_id,
            lifecycle_status: self.lifecycle_status,
            approval_status: self.approval_status,
            info: ModelInfoV1 {
                gts_type: self.info.gts_type,
                display_name: self.info.display_name,
                description: self.info.description,
                family: self.info.family,
                vendor: self.info.vendor,
                region: self.info.region,
                hosted_by: self.info.hosted_by,
                last_release_at: self.info.last_release_at,
                reasoning_level: self.info.reasoning_level,
                version: self.info.version,
                sort_order: self.info.sort_order,
                icon: self.info.icon,
                multiplier_display: self.info.multiplier_display,
                performance: self.info.performance,
                additional_info: self.info.additional_info,
                supported_api: self.info.supported_api,
                provider_model_id: self.info.provider_model_id,
                capabilities: self.info.capabilities,
                disabled_capabilities: self.info.disabled_capabilities,
                context_window: self.info.context_window,
                default_parameters: self.info.default_parameters,
                allow_parameter_override: self.info.allow_parameter_override,
                allow_extra_params: self.info.allow_extra_params,
                provider_settings: typed_settings,
            },
        })
    }
}

// ---------------------------------------------------------------------------
// Provider, ProviderHealth, Alias, DiscoveryResult
// ---------------------------------------------------------------------------

/// A configured AI provider instance for a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct Provider {
    pub id: Uuid,
    /// Human-readable identifier (immutable after creation).
    /// Format: 1-64 chars, lowercase alphanumeric + hyphen.
    pub slug: String,
    pub name: String,
    /// GTS type identifier for the provider.
    pub gts_type: String,
    /// OAGW upstream alias for provider API access (credentials, routing).
    pub oagw_alias: String,
    pub status: ProviderStatus,
    /// Whether the platform can manage this provider (e.g. install/unload
    /// models on ollama, `lm_studio`).
    pub managed: bool,
    /// Provider-specific metadata, GTS-typed.
    pub metadata: Option<serde_json::Value>,
    pub discovery_enabled: bool,
    pub discovery_interval_seconds: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Provider discovery health status (P3 —
/// `cpt-cf-model-registry-fr-health-monitoring`).
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderHealth {
    pub provider_id: Uuid,
    pub status: ProviderHealthStatus,
    pub latency_p50_ms: Option<u32>,
    pub latency_p99_ms: Option<u32>,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub last_check_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Human-friendly name mapping to a canonical model ID (P3 —
/// `cpt-cf-model-registry-fr-alias-management`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub id: Uuid,
    pub name: String,
    pub canonical_id: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Result of a model discovery run against a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryResult {
    pub provider_id: Uuid,
    pub models_discovered: u32,
    pub models_added: u32,
    pub models_updated: u32,
    pub models_deprecated: u32,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use gts::GtsSchema;

    use crate::models::{
        ContextWindow, DefaultInferenceParametersV1, MediaCapability, ModelCapabilities,
        ModelPerformance, OpenAiSettingsV1, ReasoningCapability, SupportedApi, WebSearchCapability,
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

    /// Build a raw-JSON `Model` for the given GTS schema id and provider
    /// JSON payload. This is the shape consumers see coming out of
    /// `list_tenant_models`.
    fn raw_model(gts_type: &str, provider_settings_json: serde_json::Value) -> Model {
        Model {
            id: Uuid::nil(),
            canonical_id: "openai::gpt-4o".into(),
            lifecycle_status: LifecycleStatus::Production,
            approval_status: ApprovalStatus::Approved,
            info: ModelInfoV1 {
                gts_type: gts::GtsSchemaId::new(gts_type),
                display_name: "Sample".into(),
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
                provider_model_id: "gpt-4o".into(),
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
                provider_settings: RawProviderSettings(provider_settings_json),
            },
        }
    }

    /// Sample `OpenAI` settings as a JSON value in the flat shape that
    /// would arrive via JSONB + `serde_json` from the DB or a wire payload.
    fn openai_payload() -> serde_json::Value {
        serde_json::json!({
            // Connection / auth
            "oagw_alias": "openai-prod",
            "endpoint_kind": "ChatCompletions",
            "organization": null,
            "project": null,
            // Cross-endpoint inference defaults
            "temperature": 0.7,
            "top_p": null,
            "presence_penalty": null,
            "frequency_penalty": null,
            "top_logprobs": null,
            "service_tier": null,
            "prompt_cache_retention": null,
            "reasoning_effort": null,
            "reasoning_summary": null,
            "verbosity": null,
            "parallel_tool_calls": null,
            "store": null,
            "response_format": null,
            // Chat-only
            "max_tokens": 4096,
            "max_completion_tokens": null,
            "n": null,
            "stop": null,
            "seed": null,
            "logprobs": null,
            // Responses-only
            "max_output_tokens": null,
            "max_tool_calls": null,
            "truncation": null,
            // Embeddings-only
            "encoding_format": null,
            "dimensions": null,
            // Cost (nested)
            "cost": {
                "input_per_1k_micro": null,
                "cached_input_per_1k_micro": null,
                "output_per_1k_micro": null,
                "long_context_input_per_1k_micro": null,
                "long_context_cached_input_per_1k_micro": null,
                "long_context_output_per_1k_micro": null,
                "long_context_threshold_tokens": null,
                "web_search_per_1k_calls_micro": null,
                "file_search_per_1k_calls_micro": null,
            },
        })
    }

    #[test]
    fn try_into_typed_succeeds_on_matching_schema_id() {
        let m = raw_model(OpenAiSettingsV1::SCHEMA_ID, openai_payload());
        let typed: Model<OpenAiSettingsV1> = m.try_into_typed().expect("openai schema matches");
        assert_eq!(typed.canonical_id, "openai::gpt-4o");
        assert_eq!(typed.info.provider_model_id, "gpt-4o");
        // Flat access: no `.connection.` / `.parameters.` namespacing.
        assert_eq!(typed.info.provider_settings.oagw_alias, "openai-prod");
        assert_eq!(typed.info.provider_settings.temperature, Some(0.7));
        assert_eq!(typed.info.provider_settings.max_tokens, Some(4096));
    }

    #[test]
    fn try_into_typed_fails_on_schema_id_mismatch() {
        // gts_type points at a different provider leaf (Anthropic), but
        // caller asks for OpenAi — typed-narrowing must surface a SchemaId
        // error rather than silently deserializing the wrong shape.
        let m = raw_model(
            "gts.cf.genai.model.info.v1~cf.genai._.anthropic.v1~",
            openai_payload(),
        );
        let err = m
            .try_into_typed::<OpenAiSettingsV1>()
            .expect_err("schema id mismatch");
        match err {
            ProviderSchemaMismatch::SchemaId { expected, actual } => {
                assert_eq!(expected, OpenAiSettingsV1::SCHEMA_ID);
                assert_eq!(
                    actual,
                    "gts.cf.genai.model.info.v1~cf.genai._.anthropic.v1~"
                );
            }
            other @ ProviderSchemaMismatch::Deserialize(_) => {
                panic!("expected SchemaId variant, got {other:?}")
            }
        }
    }

    #[test]
    fn try_into_typed_fails_on_unknown_gts_type() {
        // Unknown / unmodeled provider — base envelope schema with no leaf.
        let m = raw_model(
            "gts.cf.genai.model.info.v1~",
            serde_json::json!({ "anything": "goes" }),
        );
        let err = m
            .try_into_typed::<OpenAiSettingsV1>()
            .expect_err("base schema doesn't match openai leaf");
        assert!(matches!(err, ProviderSchemaMismatch::SchemaId { .. }));
    }

    #[test]
    fn try_into_typed_fails_on_malformed_payload() {
        // Schema id matches OpenAi, but payload is missing required fields.
        let m = raw_model(
            OpenAiSettingsV1::SCHEMA_ID,
            serde_json::json!({ "oagw_alias": 12345 }),
        );
        let err = m
            .try_into_typed::<OpenAiSettingsV1>()
            .expect_err("deserialization should fail");
        assert!(matches!(err, ProviderSchemaMismatch::Deserialize(_)));
    }

    #[test]
    fn typed_narrowing_preserves_common_fields() {
        let mut m = raw_model(OpenAiSettingsV1::SCHEMA_ID, openai_payload());
        // Add an entry to `additional_info` and confirm it survives narrowing.
        m.info.additional_info.insert(
            "architecture".into(),
            serde_json::Value::String("transformer".into()),
        );
        let typed: Model<OpenAiSettingsV1> = m.try_into_typed().expect("openai matches");
        assert_eq!(
            typed.info.additional_info.get("architecture"),
            Some(&serde_json::Value::String("transformer".into()))
        );
        // gts_type carries through to the typed view.
        assert_eq!(typed.info.gts_type.as_ref(), OpenAiSettingsV1::SCHEMA_ID);
    }

    #[test]
    fn raw_provider_settings_is_transparent_for_serde() {
        // RawProviderSettings serializes/deserializes as the bare inner value.
        let raw = RawProviderSettings(serde_json::json!({ "tier": "fast" }));
        let serialized = serde_json::to_value(&raw).unwrap();
        assert_eq!(serialized, serde_json::json!({ "tier": "fast" }));
        let round_tripped: RawProviderSettings = serde_json::from_value(serialized).unwrap();
        assert_eq!(round_tripped, raw);
    }

    #[test]
    fn discovery_result_fields() {
        let result = DiscoveryResult {
            provider_id: Uuid::nil(),
            models_discovered: 10,
            models_added: 3,
            models_updated: 5,
            models_deprecated: 2,
        };
        assert_eq!(result.models_discovered, 10);
        assert_eq!(result.models_added, 3);
    }
}
