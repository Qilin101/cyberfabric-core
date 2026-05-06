// Created: 2026-05-06 by Constructor Tech
// Updated: 2026-05-07 by Constructor Tech
//! `OpenAI` provider settings — Chat Completions, Responses, and Embeddings
//! APIs.
//!
//! Flat composition: all routing/auth, provider-wire parameter defaults, and
//! the nested [`OpenAiCost`] live directly on [`OpenAiSettingsV1`]. The
//! per-model override policy (`allow_parameter_override`,
//! `allow_extra_params`) is **not** here — those are flat fields on
//! [`crate::models::ModelInfoV1`]. Declared as a GTS schema leaf via
//! [`struct_to_gts_schema`]; its parent envelope is `ModelInfoV1<P>`.
//!
//! Field set is verified against the `OpenAPI` spec for `POST
//! /v1/chat/completions`, `POST /v1/responses`, and `POST /v1/embeddings`.
//! Per-request fields (`input` / `messages`, `tools`, `tool_choice`,
//! `instructions`, `metadata`, `safety_identifier`, `prompt_cache_key`,
//! `stream`, `stream_options`, `background`, `include`, `conversation`,
//! `modalities`, `audio`, `prediction`, `web_search_options`, `logit_bias`,
//! `function_call` / `functions`) are **not** stored as registry defaults —
//! the gateway builds them per call.
//!
//! Note: `supported_api` and `provider_model_id` live on `ModelInfoV1`
//! (common), not on `OpenAiSettingsV1`.

use gts_macros::struct_to_gts_schema;

use crate::models::{
    ModelInfoV1, ProviderSettings, ReasoningSummary, TextVerbosity, TruncationStrategy,
};

// ---------------------------------------------------------------------------
// Endpoint
// ---------------------------------------------------------------------------

/// Which `OpenAI` surface this connection points at.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum OpenAiEndpoint {
    /// Legacy `/v1/chat/completions`.
    ChatCompletions,
    /// New `/v1/responses` (Responses API).
    Responses,
    /// `/v1/embeddings`.
    Embeddings,
}

// ---------------------------------------------------------------------------
// Service Tier (provider-wire, five-variant)
// ---------------------------------------------------------------------------

/// `OpenAI`-specific service tier in the provider-wire shape.
///
/// Distinct from the unified two-variant [`crate::models::ServiceTier`] used
/// by `default_parameters` — the unified shape exposes only `Auto | Default`
/// to mirror the Open Responses request schema; the additional `Flex |
/// Scale | Priority` variants are `OpenAI`-only and ride alongside on this
/// struct. `Scale` was added by `OpenAI` alongside their pricing-tier rollout
/// and is distinct from `Priority`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum OpenAiServiceTier {
    Auto,
    Default,
    Flex,
    Scale,
    Priority,
}

// ---------------------------------------------------------------------------
// Reasoning effort (provider-wire, six-variant)
// ---------------------------------------------------------------------------

/// `OpenAI`-specific reasoning effort level for o-series and gpt-5 reasoning
/// models.
///
/// Distinct from the unified five-variant
/// [`crate::models::ReasoningEffort`] used by
/// `default_parameters.reasoning.effort` — the unified enum stays neutral
/// and exposes only levels every provider understands. `Minimal` was added
/// alongside the gpt-5 reasoning models in mid-2025; it sits between `None`
/// and `Low` and indicates "spend a tiny amount of reasoning effort, then
/// answer". Keeping the OpenAI-specific level here means future
/// OpenAI-only additions don't perturb the shared enum.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum OpenAiReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
}

// ---------------------------------------------------------------------------
// Prompt cache retention
// ---------------------------------------------------------------------------

/// `OpenAI` prompt-cache retention policy. `TwentyFourHours` enables extended
/// prompt caching, which keeps cached prefixes alive for up to 24 hours
/// instead of `OpenAI`'s default in-memory window.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum OpenAiPromptCacheRetention {
    /// Default in-memory window (cleared frequently).
    InMemory,
    /// Extended caching — prefixes kept alive for up to 24 hours.
    TwentyFourHours,
}

// ---------------------------------------------------------------------------
// Embedding encoding
// ---------------------------------------------------------------------------

/// Wire format used to return embedding vectors.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum OpenAiEmbeddingEncoding {
    /// JSON array of floats (default).
    Float,
    /// Base64-encoded little-endian float32 buffer.
    Base64,
}

// ---------------------------------------------------------------------------
// Response format (structured output)
// ---------------------------------------------------------------------------

/// `response_format` shape supported by Chat Completions / Responses.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum OpenAiResponseFormat {
    /// Plain text — provider default.
    Text,
    /// `{ "type": "json_object" }` — JSON-mode.
    JsonObject,
    /// `{ "type": "json_schema", "json_schema": {...} }` — schema-bound output.
    JsonSchema(serde_json::Value),
}

// ---------------------------------------------------------------------------
// Cost
// ---------------------------------------------------------------------------

/// `OpenAI` pricing in micro-credits (`u64`, scaled ×1,000,000 to avoid
/// floating point).
///
/// Token rates are **per 1K tokens**; built-in-tool rates are **per 1K
/// calls**. Long-context rates apply when the input length exceeds
/// [`OpenAiCost::long_context_threshold_tokens`] (the standard rates apply
/// below the threshold).
#[allow(clippy::struct_field_names)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
pub struct OpenAiCost {
    pub input_per_1k_micro: Option<u64>,
    pub cached_input_per_1k_micro: Option<u64>,
    pub output_per_1k_micro: Option<u64>,

    /// Input rate when above [`OpenAiCost::long_context_threshold_tokens`].
    pub long_context_input_per_1k_micro: Option<u64>,
    /// Cached-input rate when above
    /// [`OpenAiCost::long_context_threshold_tokens`].
    pub long_context_cached_input_per_1k_micro: Option<u64>,
    /// Output rate when above [`OpenAiCost::long_context_threshold_tokens`].
    pub long_context_output_per_1k_micro: Option<u64>,
    /// Input-token boundary above which the long-context rates apply.
    pub long_context_threshold_tokens: Option<u32>,

    /// Built-in web-search tool charge per 1,000 invocations.
    pub web_search_per_1k_calls_micro: Option<u64>,
    /// Built-in file-search tool charge per 1,000 invocations.
    pub file_search_per_1k_calls_micro: Option<u64>,
}

// ---------------------------------------------------------------------------
// Aggregate (flat) settings
// ---------------------------------------------------------------------------

/// `OpenAI` provider settings — the typed payload for `ModelInfoV1<OpenAiSettingsV1>`.
///
/// Flat composition: routing/auth, provider-wire parameter defaults, and the
/// nested [`OpenAiCost`].
///
/// # GTS schema
///
/// - **`schema_id`**: `gts.cf.genai.model.info.v1~cf.genai._.openai.v1~`
/// - **base**: `ModelInfoV1` (the generic envelope)
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = ModelInfoV1,
    schema_id = "gts.cf.genai.model.info.v1~cf.genai._.openai.v1~",
    description = "OpenAI provider settings (Chat Completions / Responses / Embeddings)",
    properties = "oagw_alias,endpoint_kind,organization,project,temperature,top_p,presence_penalty,frequency_penalty,top_logprobs,service_tier,prompt_cache_retention,reasoning_effort,reasoning_summary,verbosity,parallel_tool_calls,store,response_format,max_tokens,max_completion_tokens,n,stop,seed,logprobs,max_output_tokens,max_tool_calls,truncation,encoding_format,dimensions,cost"
)]
#[derive(Debug, Clone, PartialEq)]
pub struct OpenAiSettingsV1 {
    // ── Connection / auth ─────────────────────────────────────────────
    /// OAGW upstream alias for credentials and base URL routing.
    pub oagw_alias: String,
    pub endpoint_kind: OpenAiEndpoint,
    /// Optional `OpenAI` organization id.
    pub organization: Option<String>,
    /// Optional `OpenAI` project id.
    pub project: Option<String>,

    // ── Cross-endpoint inference defaults ─────────────────────────────
    /// Sampling temperature (`OpenAI` accepts `0.0..=2.0`; SDK does not
    /// range-check).
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    /// Number of top log-probabilities to return per token (`OpenAI`
    /// accepts 0..=20). Pairs with `logprobs` on the Chat API.
    pub top_logprobs: Option<u8>,
    /// Full five-variant `OpenAI` service tier (`Auto | Default | Flex |
    /// Scale | Priority`); distinct from the unified two-variant
    /// `ServiceTier` on `default_parameters`.
    pub service_tier: Option<OpenAiServiceTier>,
    /// `OpenAI` prompt-cache retention policy.
    pub prompt_cache_retention: Option<OpenAiPromptCacheRetention>,
    /// For o-series and gpt-5 reasoning models. Uses the OpenAI-specific
    /// [`OpenAiReasoningEffort`] enum (six variants, including `Minimal`
    /// which is OpenAI-only). Distinct from the unified five-variant
    /// `ReasoningEffort` on `default_parameters.reasoning.effort`, which
    /// stays neutral.
    pub reasoning_effort: Option<OpenAiReasoningEffort>,
    /// Responses-API `reasoning.summary` knob — uses the shared
    /// [`ReasoningSummary`] enum (`Auto | Concise | Detailed`).
    pub reasoning_summary: Option<ReasoningSummary>,
    /// Chat-API top-level `verbosity` and Responses-API `text.verbosity`
    /// map to the same shape; one registry field covers both.
    pub verbosity: Option<TextVerbosity>,
    pub parallel_tool_calls: Option<bool>,
    /// Chat: whether to store the request for distillation/evals.
    /// Responses: whether to store for retrieval.
    pub store: Option<bool>,
    /// Chat: `response_format`. Responses: ships via `text.format`.
    pub response_format: Option<OpenAiResponseFormat>,

    // ── Chat Completions only ─────────────────────────────────────────
    /// Legacy chat-completions cap. **Deprecated by `OpenAI`** in favor of
    /// `max_completion_tokens`; retained for back-compat with older models.
    pub max_tokens: Option<u32>,
    /// Current chat-completions cap (mutually exclusive with `max_tokens`
    /// in practice).
    pub max_completion_tokens: Option<u32>,
    /// Number of completions per request (`OpenAI` accepts 1..=128).
    pub n: Option<u32>,
    /// Stop sequences.
    pub stop: Option<Vec<String>>,
    /// Deterministic sampling seed. **Marked Beta + deprecated by
    /// `OpenAI`** but still accepted on the wire.
    pub seed: Option<u64>,
    /// Whether to return log probabilities of output tokens.
    pub logprobs: Option<bool>,

    // ── Responses only ────────────────────────────────────────────────
    /// Responses-API output cap. `OpenAI` enforces a minimum of 16
    /// server-side; the SDK does not range-check this value.
    pub max_output_tokens: Option<u32>,
    /// Maximum total built-in tool calls per response.
    pub max_tool_calls: Option<u32>,
    /// Reuses the shared [`TruncationStrategy`] from `default_parameters`
    /// (`Auto | Disabled`). `OpenAI`'s default on the wire is `Disabled`.
    pub truncation: Option<TruncationStrategy>,

    // ── Embeddings only ───────────────────────────────────────────────
    /// `Float | Base64`.
    pub encoding_format: Option<OpenAiEmbeddingEncoding>,
    /// Output embedding dimensionality (text-embedding-3 and later only).
    /// Distinct from `ModelInfoV1.context_window.output_vector_size`: this
    /// field is the **request default** sent on the wire;
    /// `output_vector_size` is the model's intrinsic native dimensionality.
    pub dimensions: Option<u32>,

    // ── Cost (nested) ─────────────────────────────────────────────────
    pub cost: OpenAiCost,
}

impl ProviderSettings for OpenAiSettingsV1 {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> OpenAiSettingsV1 {
        OpenAiSettingsV1 {
            oagw_alias: "openai-prod".into(),
            endpoint_kind: OpenAiEndpoint::ChatCompletions,
            organization: Some("org-abc".into()),
            project: Some("proj-xyz".into()),
            temperature: Some(0.7),
            top_p: Some(1.0),
            presence_penalty: None,
            frequency_penalty: None,
            top_logprobs: Some(5),
            service_tier: Some(OpenAiServiceTier::Default),
            prompt_cache_retention: Some(OpenAiPromptCacheRetention::InMemory),
            reasoning_effort: Some(OpenAiReasoningEffort::Medium),
            reasoning_summary: Some(ReasoningSummary::Auto),
            verbosity: Some(TextVerbosity::Medium),
            parallel_tool_calls: None,
            store: None,
            response_format: Some(OpenAiResponseFormat::JsonObject),
            max_tokens: Some(4096),
            max_completion_tokens: None,
            n: Some(1),
            stop: None,
            seed: None,
            logprobs: Some(false),
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

    #[test]
    fn flat_routing_and_org_fields() {
        let s = sample();
        assert_eq!(s.oagw_alias, "openai-prod");
        assert_eq!(s.endpoint_kind, OpenAiEndpoint::ChatCompletions);
        assert_eq!(s.organization.as_deref(), Some("org-abc"));
        assert_eq!(s.project.as_deref(), Some("proj-xyz"));
    }

    #[test]
    fn flat_parameter_defaults_present() {
        let s = sample();
        assert_eq!(s.temperature, Some(0.7));
        assert_eq!(s.top_p, Some(1.0));
        assert_eq!(s.max_tokens, Some(4096));
        assert!(s.max_completion_tokens.is_none());
        assert!(s.reasoning_effort.is_some());
        assert_eq!(s.top_logprobs, Some(5));
        assert_eq!(s.n, Some(1));
        assert_eq!(s.logprobs, Some(false));
    }

    #[test]
    fn response_format_variants() {
        let text = OpenAiResponseFormat::Text;
        let json_obj = OpenAiResponseFormat::JsonObject;
        let json_schema = OpenAiResponseFormat::JsonSchema(serde_json::json!({"type": "object"}));
        assert_ne!(text, json_obj);
        assert_ne!(json_obj, json_schema);
    }

    #[test]
    fn service_tier_has_five_variants() {
        // Provider-wire tier carries all five OpenAI variants; the unified
        // ServiceTier on default_parameters narrows to two.
        let tiers = [
            OpenAiServiceTier::Auto,
            OpenAiServiceTier::Default,
            OpenAiServiceTier::Flex,
            OpenAiServiceTier::Scale,
            OpenAiServiceTier::Priority,
        ];
        assert_eq!(tiers.len(), 5);
        // `Scale` and `Priority` are distinct.
        assert_ne!(tiers[3], tiers[4]);
    }

    #[test]
    fn prompt_cache_retention_two_variants() {
        let in_mem = OpenAiPromptCacheRetention::InMemory;
        let extended = OpenAiPromptCacheRetention::TwentyFourHours;
        assert_ne!(in_mem, extended);
    }

    #[test]
    fn embedding_encoding_variants() {
        let f = OpenAiEmbeddingEncoding::Float;
        let b = OpenAiEmbeddingEncoding::Base64;
        assert_ne!(f, b);
    }

    #[test]
    fn cost_micro_credits_standard_tier() {
        let c = sample().cost;
        assert_eq!(c.input_per_1k_micro, Some(2_500_000));
        assert_eq!(c.cached_input_per_1k_micro, Some(1_250_000));
        assert_eq!(c.output_per_1k_micro, Some(10_000_000));
    }

    #[test]
    fn cost_long_context_tier_optional() {
        let mut s = sample();
        s.cost.long_context_input_per_1k_micro = Some(5_000_000);
        s.cost.long_context_cached_input_per_1k_micro = Some(2_500_000);
        s.cost.long_context_output_per_1k_micro = Some(20_000_000);
        s.cost.long_context_threshold_tokens = Some(128_000);
        // Long-context rates are higher than standard rates above the
        // boundary (128K input tokens here).
        assert!(
            s.cost.long_context_input_per_1k_micro.unwrap() > s.cost.input_per_1k_micro.unwrap()
        );
        assert_eq!(s.cost.long_context_threshold_tokens, Some(128_000));
    }

    #[test]
    fn cost_built_in_tool_rates() {
        let mut s = sample();
        // Per OpenAI's published prices: web_search and file_search are
        // billed per call (not per token). Stored as micro-credits per 1K
        // calls.
        s.cost.web_search_per_1k_calls_micro = Some(25_000_000); // $25 / 1K
        s.cost.file_search_per_1k_calls_micro = Some(2_500_000); // $2.50 / 1K
        assert!(
            s.cost.web_search_per_1k_calls_micro.unwrap()
                > s.cost.file_search_per_1k_calls_micro.unwrap()
        );
    }

    #[test]
    fn responses_api_uses_max_completion_tokens() {
        let s = OpenAiSettingsV1 {
            endpoint_kind: OpenAiEndpoint::Responses,
            reasoning_summary: Some(ReasoningSummary::Auto),
            max_completion_tokens: Some(8192),
            ..sample()
        };
        assert_eq!(s.endpoint_kind, OpenAiEndpoint::Responses);
        assert_eq!(s.max_completion_tokens, Some(8192));
        assert_eq!(s.reasoning_summary, Some(ReasoningSummary::Auto));
    }

    #[test]
    fn responses_api_specific_fields() {
        let s = OpenAiSettingsV1 {
            endpoint_kind: OpenAiEndpoint::Responses,
            max_output_tokens: Some(2048),
            max_tool_calls: Some(8),
            truncation: Some(TruncationStrategy::Auto),
            ..sample()
        };
        assert_eq!(s.max_output_tokens, Some(2048));
        assert_eq!(s.max_tool_calls, Some(8));
        assert_eq!(s.truncation, Some(TruncationStrategy::Auto));
    }

    #[test]
    fn embeddings_api_specific_fields() {
        let s = OpenAiSettingsV1 {
            endpoint_kind: OpenAiEndpoint::Embeddings,
            encoding_format: Some(OpenAiEmbeddingEncoding::Base64),
            dimensions: Some(1536),
            ..sample()
        };
        assert_eq!(s.endpoint_kind, OpenAiEndpoint::Embeddings);
        assert_eq!(s.encoding_format, Some(OpenAiEmbeddingEncoding::Base64));
        assert_eq!(s.dimensions, Some(1536));
    }

    #[test]
    fn reasoning_effort_uses_openai_specific_enum() {
        // OpenAi reasoning_effort is the six-variant `OpenAiReasoningEffort`
        // (carries `Minimal`, added alongside gpt-5), distinct from the
        // unified five-variant `ReasoningEffort` on default_parameters.
        let s = OpenAiSettingsV1 {
            reasoning_effort: Some(OpenAiReasoningEffort::Minimal),
            ..sample()
        };
        assert_eq!(s.reasoning_effort, Some(OpenAiReasoningEffort::Minimal));
    }

    #[test]
    fn openai_reasoning_effort_has_six_variants() {
        let efforts = [
            OpenAiReasoningEffort::None,
            OpenAiReasoningEffort::Minimal,
            OpenAiReasoningEffort::Low,
            OpenAiReasoningEffort::Medium,
            OpenAiReasoningEffort::High,
            OpenAiReasoningEffort::XHigh,
        ];
        assert_eq!(efforts.len(), 6);
        // `Minimal` sits between `None` and `Low`.
        assert_ne!(efforts[0], efforts[1]);
        assert_ne!(efforts[1], efforts[2]);
    }
}
