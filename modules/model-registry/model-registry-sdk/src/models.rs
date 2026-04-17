// Created: 2026-04-17 by Constructor Tech
//! Public models for the `model-registry` module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the `model-registry` module and its consumers. No serde derives
//! on domain entities — serialization concerns belong to the REST layer.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Lifecycle status of a model in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleStatus {
    Production,
    Preview,
    Experimental,
    Deprecated,
    Sunset,
}

/// Approval status of a model for a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Revoked,
}

/// Operational status of a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    Active,
    Disabled,
}

/// Health status derived from discovery calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// API type that a model supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedApi {
    Completion,
    Embedding,
}

/// Reasoning effort level (matches `OpenAI` Responses API).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningEffort {
    None,
    Low,
    Medium,
    High,
    XHigh,
}

/// Service tier for request routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceTier {
    Auto,
    Default,
    Flex,
    Priority,
}

// ---------------------------------------------------------------------------
// API Resolution
// ---------------------------------------------------------------------------

/// Routing and resolution information for calling the model via OAGW.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiResolution {
    /// Which API types this model supports (e.g. completion, embedding).
    pub supported_api: Vec<SupportedApi>,
    /// Provider plugin family that handles requests for this model
    /// (e.g. `"openai"`, `"anthropic"`, `"ollama"`). Determines which
    /// API adapter is used to build and parse requests.
    pub api_family: String,
    /// OAGW upstream alias that provides credentials and base URL routing.
    pub oagw_alias: String,
    /// Provider's model identifier — used both in `canonical_id`
    /// (`{provider_slug}::{provider_model_id}`) and sent to the provider
    /// in API requests. Users can create aliases to expose a different
    /// name that maps to this ID.
    pub provider_model_id: String,
}

// ---------------------------------------------------------------------------
// Capabilities
// ---------------------------------------------------------------------------

/// Reasoning sub-capabilities indicating which reasoning controls the model
/// accepts.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReasoningCapability {
    /// Supports `reasoning_effort` parameter (low/medium/high).
    pub effort: bool,
    /// Supports toggling reasoning on/off.
    pub toggle: bool,
    /// Supports resuming/continuing a reasoning chain.
    pub resume: bool,
    /// Supports explicit reasoning token budget.
    pub budget: bool,
}

/// Web search capability flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebSearchCapability {
    pub enabled: bool,
    /// Whether the model supports configuring allowed domains for web search.
    pub allowed_domains: bool,
}

/// Capability flags describing what the model can do.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Supports image/vision input.
    pub vision: bool,
    /// Reasoning controls (effort, toggle, resume, budget).
    pub reasoning: ReasoningCapability,
    /// Supports function/tool calling.
    pub function_calling: bool,
    /// Supports structured output via response schema (JSON schema).
    pub response_schema: bool,
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports file input (e.g. PDFs, documents).
    pub file_input: bool,
    /// Can generate images.
    pub image_generation: bool,
    /// Supports code interpreter / sandboxed execution.
    pub code_interpreter: bool,
    /// Web search capability.
    pub web_search: WebSearchCapability,
}

// ---------------------------------------------------------------------------
// Context Window
// ---------------------------------------------------------------------------

/// Token limits for the model's context window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextWindow {
    pub max_input_tokens: u32,
    /// Maximum output tokens. `None` for embedding models that produce
    /// vectors instead of token sequences.
    pub max_output_tokens: Option<u32>,
    /// Output vector dimensionality for embedding models.
    pub output_vector_size: Option<u32>,
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Default inference parameters and override policy.
///
/// These are the defaults sent to the provider when the caller does not
/// specify them. The `allow_parameter_override` and `allow_extra_params`
/// fields control what callers are permitted to override per-request.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelParameters {
    pub temperature: Option<f64>,
    /// Default reasoning effort for reasoning models.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Default max output tokens. Distinct from `ContextWindow::max_output_tokens`
    /// which is the model's hard limit — this is the default per-request cap.
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    /// Stop sequences.
    pub stop: Option<Vec<String>>,
    /// Service tier for request routing.
    pub service_tier: Option<ServiceTier>,
    /// Provider-specific extra body parameters (e.g. vLLM `top_k`,
    /// `chat_template_kwargs`).
    pub extra_body: Option<serde_json::Value>,
    /// Whether users are allowed to override the set parameters per-request.
    pub allow_parameter_override: bool,
    /// Which extra parameter names users are allowed to pass in the request body.
    pub allow_extra_params: Vec<String>,
}

// ---------------------------------------------------------------------------
// Cost
// ---------------------------------------------------------------------------

/// Token pricing in micro-credits per 1K tokens.
///
/// Values are scaled by 1,000,000 to avoid floating point
/// (e.g. `1_000_000` = 1.0x multiplier).
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelCost {
    /// Micro-credits per 1K input tokens.
    pub input_token_cost_micro: Option<u64>,
    /// Micro-credits per 1K output tokens.
    pub output_token_cost_micro: Option<u64>,
    /// Micro-credits per 1K cached input tokens.
    pub cached_input_token_cost_micro: Option<u64>,
}

// ---------------------------------------------------------------------------
// Information
// ---------------------------------------------------------------------------

/// Estimated performance characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPerformance {
    /// Expected response latency in milliseconds.
    pub response_latency_ms: Option<u32>,
    /// Expected generation speed in tokens per second.
    pub tokens_per_second: Option<u32>,
}

/// Complete model information: descriptive metadata, routing, capabilities,
/// parameters, cost.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInfo {
    /// Display name shown in UI.
    pub display_name: String,
    pub description: Option<String>,
    /// Model family (e.g. `"gpt-4"`, `"claude"`, `"llama"`).
    pub family: Option<String>,
    /// Model vendor (e.g. `"OpenAI"`, `"Anthropic"`, `"Meta"`).
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
    /// Arbitrary key-value metadata for provider- or deployment-specific info.
    pub additional_info: HashMap<String, serde_json::Value>,
    /// API routing and provider resolution.
    pub api_resolution: ApiResolution,
    /// What the model can do.
    pub capabilities: ModelCapabilities,
    /// Capabilities that are administratively disabled. Flags set to `true`
    /// indicate the corresponding capability is disabled for this model.
    pub disabled_capabilities: ModelCapabilities,
    /// Token limits.
    pub context_window: ContextWindow,
    /// Default inference parameters and override policy.
    pub parameters: ModelParameters,
    /// Token pricing.
    pub cost: ModelCost,
}

// ---------------------------------------------------------------------------
// Domain entities
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

/// An AI model in the catalog, scoped to a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub id: Uuid,
    /// Format: `{provider_slug}::{provider_model_id}`.
    pub canonical_id: String,
    pub lifecycle_status: LifecycleStatus,
    pub approval_status: ApprovalStatus,
    /// All model information: routing, capabilities, parameters, cost, and
    /// descriptive metadata.
    pub info: ModelInfo,
}

/// Provider discovery health status (P2).
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

/// Human-friendly name mapping to a canonical model ID (P2).
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
// Request DTOs — CreateProviderRequest (builder pattern)
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
// Request DTOs — UpdateProviderRequest (PATCH semantics)
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
// Request DTOs — CreateAliasRequest (P2)
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

    // ── Test helpers ──

    fn sample_api_resolution() -> ApiResolution {
        ApiResolution {
            supported_api: vec![SupportedApi::Completion],
            api_family: "openai".into(),
            oagw_alias: "openai-prod".into(),
            provider_model_id: "gpt-4o".into(),
        }
    }

    fn sample_capabilities() -> ModelCapabilities {
        ModelCapabilities {
            vision: true,
            reasoning: ReasoningCapability {
                effort: false,
                toggle: false,
                resume: false,
                budget: false,
            },
            function_calling: true,
            response_schema: true,
            streaming: true,
            file_input: false,
            image_generation: false,
            code_interpreter: false,
            web_search: WebSearchCapability {
                enabled: false,
                allowed_domains: false,
            },
        }
    }

    fn sample_no_capabilities() -> ModelCapabilities {
        ModelCapabilities {
            vision: false,
            reasoning: ReasoningCapability {
                effort: false,
                toggle: false,
                resume: false,
                budget: false,
            },
            function_calling: false,
            response_schema: false,
            streaming: false,
            file_input: false,
            image_generation: false,
            code_interpreter: false,
            web_search: WebSearchCapability {
                enabled: false,
                allowed_domains: false,
            },
        }
    }

    fn sample_context_window() -> ContextWindow {
        ContextWindow {
            max_input_tokens: 128_000,
            max_output_tokens: Some(16_384),
            output_vector_size: None,
        }
    }

    fn sample_parameters() -> ModelParameters {
        ModelParameters {
            temperature: Some(0.7),
            reasoning_effort: None,
            max_tokens: Some(4096),
            top_p: Some(1.0),
            stop: None,
            service_tier: None,
            extra_body: None,
            allow_parameter_override: true,
            allow_extra_params: vec![],
        }
    }

    fn sample_cost() -> ModelCost {
        ModelCost {
            input_token_cost_micro: Some(2_500_000),
            output_token_cost_micro: Some(10_000_000),
            cached_input_token_cost_micro: Some(1_250_000),
        }
    }

    fn sample_info() -> ModelInfo {
        ModelInfo {
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
            api_resolution: sample_api_resolution(),
            capabilities: sample_capabilities(),
            disabled_capabilities: sample_no_capabilities(),
            context_window: sample_context_window(),
            parameters: sample_parameters(),
            cost: sample_cost(),
        }
    }

    fn sample_model() -> Model {
        Model {
            id: Uuid::nil(),
            canonical_id: "openai::gpt-4o".into(),
            lifecycle_status: LifecycleStatus::Production,
            approval_status: ApprovalStatus::Approved,
            info: sample_info(),
        }
    }

    // ── Enum equality ──

    #[test]
    fn lifecycle_status_equality() {
        assert_eq!(LifecycleStatus::Production, LifecycleStatus::Production);
        assert_ne!(LifecycleStatus::Production, LifecycleStatus::Deprecated);
    }

    #[test]
    fn approval_status_equality() {
        assert_eq!(ApprovalStatus::Approved, ApprovalStatus::Approved);
        assert_ne!(ApprovalStatus::Pending, ApprovalStatus::Rejected);
    }

    #[test]
    fn provider_status_equality() {
        assert_eq!(ProviderStatus::Active, ProviderStatus::Active);
        assert_ne!(ProviderStatus::Active, ProviderStatus::Disabled);
    }

    #[test]
    fn provider_health_status_equality() {
        assert_eq!(ProviderHealthStatus::Healthy, ProviderHealthStatus::Healthy);
        assert_ne!(
            ProviderHealthStatus::Healthy,
            ProviderHealthStatus::Degraded
        );
    }

    #[test]
    fn supported_api_equality_and_hash() {
        assert_eq!(SupportedApi::Completion, SupportedApi::Completion);
        assert_ne!(SupportedApi::Completion, SupportedApi::Embedding);

        let mut set = std::collections::HashSet::new();
        set.insert(SupportedApi::Completion);
        set.insert(SupportedApi::Embedding);
        set.insert(SupportedApi::Completion);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn reasoning_effort_equality() {
        assert_eq!(ReasoningEffort::High, ReasoningEffort::High);
        assert_ne!(ReasoningEffort::Low, ReasoningEffort::XHigh);
        assert_ne!(ReasoningEffort::None, ReasoningEffort::Medium);
    }

    #[test]
    fn service_tier_equality() {
        assert_eq!(ServiceTier::Default, ServiceTier::Default);
        assert_ne!(ServiceTier::Flex, ServiceTier::Priority);
    }

    // ── API Resolution ──

    #[test]
    fn api_resolution_fields() {
        let ar = sample_api_resolution();
        assert_eq!(ar.supported_api, vec![SupportedApi::Completion]);
        assert_eq!(ar.api_family, "openai");
        assert_eq!(ar.oagw_alias, "openai-prod");
        assert_eq!(ar.provider_model_id, "gpt-4o");
    }

    #[test]
    fn api_resolution_multiple_apis() {
        let ar = ApiResolution {
            supported_api: vec![SupportedApi::Completion, SupportedApi::Embedding],
            api_family: "openai".into(),
            oagw_alias: "openai-prod".into(),
            provider_model_id: "text-embedding-3-large".into(),
        };
        assert_eq!(ar.supported_api.len(), 2);
    }

    // ── Capabilities ──

    #[test]
    fn capabilities_reasoning_all_false() {
        let r = ReasoningCapability {
            effort: false,
            toggle: false,
            resume: false,
            budget: false,
        };
        assert!(!r.effort);
        assert!(!r.toggle);
        assert!(!r.resume);
        assert!(!r.budget);
    }

    #[test]
    fn capabilities_reasoning_model() {
        let caps = ModelCapabilities {
            vision: false,
            reasoning: ReasoningCapability {
                effort: true,
                toggle: true,
                resume: false,
                budget: true,
            },
            function_calling: true,
            response_schema: true,
            streaming: true,
            file_input: false,
            image_generation: false,
            code_interpreter: false,
            web_search: WebSearchCapability {
                enabled: false,
                allowed_domains: false,
            },
        };
        assert!(caps.reasoning.effort);
        assert!(caps.reasoning.toggle);
        assert!(caps.reasoning.budget);
    }

    #[test]
    fn web_search_with_domain_support() {
        let ws = WebSearchCapability {
            enabled: true,
            allowed_domains: true,
        };
        assert!(ws.enabled);
        assert!(ws.allowed_domains);
    }

    #[test]
    fn web_search_no_domain_support() {
        let ws = WebSearchCapability {
            enabled: true,
            allowed_domains: false,
        };
        assert!(ws.enabled);
        assert!(!ws.allowed_domains);
    }

    // ── Context Window ──

    #[test]
    fn context_window_completion_model() {
        let cw = sample_context_window();
        assert_eq!(cw.max_input_tokens, 128_000);
        assert_eq!(cw.max_output_tokens, Some(16_384));
        assert!(cw.output_vector_size.is_none());
    }

    #[test]
    fn context_window_embedding_model() {
        let cw = ContextWindow {
            max_input_tokens: 8191,
            max_output_tokens: None,
            output_vector_size: Some(3072),
        };
        assert!(cw.max_output_tokens.is_none());
        assert_eq!(cw.output_vector_size, Some(3072));
    }

    // ── Parameters ──

    #[test]
    fn parameters_with_overrides_allowed() {
        let p = sample_parameters();
        assert!(p.allow_parameter_override);
        assert!(p.allow_extra_params.is_empty());
    }

    #[test]
    fn parameters_locked_down() {
        let p = ModelParameters {
            temperature: Some(0.0),
            reasoning_effort: Some(ReasoningEffort::High),
            max_tokens: Some(8192),
            top_p: None,
            stop: None,
            service_tier: Some(ServiceTier::Default),
            extra_body: None,
            allow_parameter_override: false,
            allow_extra_params: vec![],
        };
        assert!(!p.allow_parameter_override);
        assert_eq!(p.reasoning_effort, Some(ReasoningEffort::High));
        assert_eq!(p.service_tier, Some(ServiceTier::Default));
    }

    #[test]
    fn parameters_with_selective_extra_params() {
        let p = ModelParameters {
            temperature: Some(0.7),
            reasoning_effort: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            service_tier: Some(ServiceTier::Flex),
            extra_body: None,
            allow_parameter_override: true,
            allow_extra_params: vec!["top_k".into(), "repetition_penalty".into()],
        };
        assert_eq!(p.allow_extra_params.len(), 2);
        assert_eq!(p.service_tier, Some(ServiceTier::Flex));
    }

    // ── Cost ──

    #[test]
    fn model_cost_all_present() {
        let cost = sample_cost();
        assert_eq!(cost.input_token_cost_micro, Some(2_500_000));
        assert_eq!(cost.output_token_cost_micro, Some(10_000_000));
        assert_eq!(cost.cached_input_token_cost_micro, Some(1_250_000));
    }

    #[test]
    fn model_cost_free_model() {
        let cost = ModelCost {
            input_token_cost_micro: None,
            output_token_cost_micro: None,
            cached_input_token_cost_micro: None,
        };
        assert!(cost.input_token_cost_micro.is_none());
    }

    // ── ModelInfo ──

    #[test]
    fn model_info_fields() {
        let info = sample_info();
        assert_eq!(info.display_name, "GPT-4o");
        assert_eq!(info.vendor.as_deref(), Some("OpenAI"));
        assert_eq!(info.family.as_deref(), Some("gpt-4"));
        assert!(info.additional_info.is_empty());
        // Technical sections are accessible via info
        assert_eq!(info.api_resolution.provider_model_id, "gpt-4o");
        assert_eq!(info.context_window.max_input_tokens, 128_000);
        assert!(info.capabilities.vision);
        assert!(!info.disabled_capabilities.vision);
    }

    #[test]
    fn model_info_with_additional_info() {
        let mut info = sample_info();
        info.additional_info.insert(
            "architecture".into(),
            serde_json::Value::String("transformer".into()),
        );
        info.additional_info.insert(
            "size_bytes".into(),
            serde_json::Value::Number(serde_json::Number::from(70_000_000_000_i64)),
        );
        assert_eq!(info.additional_info.len(), 2);
    }

    #[test]
    fn model_ui_fields() {
        let info = sample_info();
        assert_eq!(info.sort_order, Some(1));
        assert_eq!(info.icon.as_deref(), Some("https://example.com/gpt4o.svg"));
        assert_eq!(info.multiplier_display.as_deref(), Some("1x"));
    }

    #[test]
    fn model_performance_fields() {
        let perf = ModelPerformance {
            response_latency_ms: Some(200),
            tokens_per_second: Some(150),
        };
        assert_eq!(perf.response_latency_ms, Some(200));
        assert_eq!(perf.tokens_per_second, Some(150));
    }

    // ── Model entity ──

    #[test]
    fn model_canonical_id() {
        let model = sample_model();
        assert_eq!(model.canonical_id, "openai::gpt-4o");
        assert_eq!(model.info.api_resolution.provider_model_id, "gpt-4o");
    }

    #[test]
    fn model_disabled_capabilities() {
        let mut model = sample_model();
        model.info.disabled_capabilities.web_search.enabled = true;
        model.info.disabled_capabilities.code_interpreter = true;
        assert!(model.info.disabled_capabilities.web_search.enabled);
        assert!(model.info.disabled_capabilities.code_interpreter);
        assert!(!model.info.disabled_capabilities.vision);
    }

    #[test]
    fn model_approval_status() {
        let model = sample_model();
        assert_eq!(model.approval_status, ApprovalStatus::Approved);
        assert_eq!(model.lifecycle_status, LifecycleStatus::Production);
        assert_eq!(model.info.display_name, "GPT-4o");
    }

    // ── Provider ──

    #[test]
    fn create_provider_request_builder() {
        let req = CreateProviderRequest::builder(
            "openai",
            "OpenAI",
            "gts.x.genai.models.provider.v1~x.genai._.openai.v1~",
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
            "gts.x.genai.models.provider.v1~x.genai.local.provider.v1~",
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
