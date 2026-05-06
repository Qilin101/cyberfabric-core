// Created: 2026-05-06 by Constructor Tech
// Updated: 2026-05-07 by Constructor Tech
//! Provider-independent model types — enums, capability flags, the context
//! window, and performance characteristics.
//!
//! These types live on `ModelInfo<P>` directly (not on the per-provider
//! settings) because their shape is meaningful for every provider. The
//! per-model override policy is no longer a struct in this module — its
//! two fields (`allow_parameter_override`, `allow_extra_params`) are now
//! flat fields on [`crate::models::ModelInfoV1`].

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Lifecycle status of a model in the catalog.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum LifecycleStatus {
    Production,
    Preview,
    Experimental,
    Deprecated,
    Sunset,
}

/// Approval status of a model for a tenant.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Revoked,
}

/// Operational status of a provider.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum ProviderStatus {
    Active,
    Disabled,
}

/// Health status derived from discovery calls.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// API kind that a model exposes.
///
/// A model may expose multiple API surfaces (e.g. `[Completion, Batch]` for
/// a chat model that's also reachable via the asynchronous batch API). Each
/// variant corresponds to a distinct LLM Gateway endpoint family.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
pub enum SupportedApi {
    /// Synchronous chat / responses APIs.
    Completion,
    /// Embedding APIs.
    Embedding,
    /// Asynchronous batch API (see `gts.cf.llmgw.async.batch.v1~`). May
    /// coexist with `Completion` / `Embedding` on the same model.
    Batch,
}

/// Unified reasoning effort level used by `default_parameters.reasoning.effort`.
///
/// This enum is **neutral** — it exposes the levels every provider
/// understands. Provider-specific reasoning levels (e.g. `OpenAI`'s
/// `Minimal`, added with gpt-5) live on the per-provider settings as a
/// distinct enum (see `OpenAiReasoningEffort`), so adding an OpenAI-only
/// level doesn't perturb the shared enum.
///
/// Wire format is lowercase (`"none" | "low" | "medium" | "high" | "xhigh"`)
/// to match `gts.cf.llmgw.core.reasoning_config.v1~`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Low,
    Medium,
    High,
    XHigh,
}

/// Service tier in the unified two-variant shape used by `default_parameters`
/// (matches the Open Responses request schema).
///
/// Provider-specific tiers (e.g. `OpenAI`'s full `auto | default | flex |
/// priority` set) live on the per-provider settings as a separate enum (see
/// `OpenAiServiceTier`).
///
/// Wire format is lowercase (`"auto" | "default"`) to match
/// `gts.cf.llmgw.core.create_response_body.v1~`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Auto,
    Default,
}

// ---------------------------------------------------------------------------
// Capabilities
// ---------------------------------------------------------------------------

/// Reasoning sub-capabilities indicating which reasoning controls the model
/// accepts.
#[allow(clippy::struct_excessive_bools)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
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
#[allow(clippy::struct_excessive_bools)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct WebSearchCapability {
    /// Whether web search is available.
    pub enabled: bool,
    /// Whether the model supports configuring an allow-list of domains to
    /// restrict search to.
    pub allowed_domains: bool,
    /// Whether the model supports configuring a deny-list of domains to
    /// exclude from search.
    pub excluded_domains: bool,
}

/// Shared shape for media-typed capabilities — `vision`, `file_input`,
/// `image_generation`, `audio_input`, `audio_output`.
///
/// Captures both whether the capability is available and which media types
/// the model accepts (or produces). MIME types follow [RFC 6838][1] —
/// lowercased canonical spelling (e.g. `audio/mpeg`, not `audio/MP3`). An
/// empty `supported_mime_types` means "no per-type list surfaced by the
/// provider"; consumers should treat that as "best-effort, defer to the
/// provider's documented support".
///
/// In `ModelCapabilities::disabled_capabilities`, this struct is interpreted
/// **subtractively**: `enabled = true` disables the whole capability;
/// `supported_mime_types` lists media types disabled out of the parent
/// capability's enabled list.
///
/// [1]: https://datatracker.ietf.org/doc/html/rfc6838
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct MediaCapability {
    /// Whether the capability is available.
    pub enabled: bool,
    /// Accepted (or produced) media types — RFC 6838 names. Empty when
    /// `enabled` is `false` or the provider doesn't surface a per-type list.
    pub supported_mime_types: Vec<String>,
}

/// Capability flags describing what the model can do.
///
/// `Copy` is intentionally not derived: [`MediaCapability`] carries a
/// `Vec<String>`, so the struct is `Clone` only.
#[allow(clippy::struct_excessive_bools)]
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ModelCapabilities {
    /// Supports image/vision input.
    pub vision: MediaCapability,
    /// Reasoning controls (effort, toggle, resume, budget).
    pub reasoning: ReasoningCapability,
    /// Supports function/tool calling.
    pub function_calling: bool,
    /// Supports structured output via response schema (JSON schema).
    pub response_schema: bool,
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports file input (e.g. PDFs, documents).
    pub file_input: MediaCapability,
    /// Can generate images.
    pub image_generation: MediaCapability,
    /// Accepts audio input (speech-to-text, audio understanding).
    pub audio_input: MediaCapability,
    /// Produces audio output (text-to-speech).
    pub audio_output: MediaCapability,
    /// Supports code interpreter / sandboxed execution.
    pub code_interpreter: bool,
    /// Web search capability.
    pub web_search: WebSearchCapability,
}

// ---------------------------------------------------------------------------
// Context Window
// ---------------------------------------------------------------------------

/// Token limits for the model's context window.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ContextWindow {
    pub max_input_tokens: u32,
    /// Maximum output tokens. `None` for embedding models that produce
    /// vectors instead of token sequences.
    pub max_output_tokens: Option<u32>,
    /// Output vector dimensionality for embedding models.
    pub output_vector_size: Option<u32>,
}

// ---------------------------------------------------------------------------
// Performance
// ---------------------------------------------------------------------------

/// Estimated performance characteristics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ModelPerformance {
    /// Expected response latency in milliseconds.
    pub response_latency_ms: Option<u32>,
    /// Expected generation speed in tokens per second.
    pub tokens_per_second: Option<u32>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_status_equality() {
        assert_eq!(LifecycleStatus::Production, LifecycleStatus::Production);
        assert_ne!(LifecycleStatus::Production, LifecycleStatus::Deprecated);
    }

    #[test]
    fn reasoning_effort_wire_format_is_lowercase() {
        // Pinned to match gts.cf.llmgw.core.reasoning_config.v1~ — gateway
        // schema expects lowercase strings.
        for (variant, expected) in [
            (ReasoningEffort::None, "\"none\""),
            (ReasoningEffort::Low, "\"low\""),
            (ReasoningEffort::Medium, "\"medium\""),
            (ReasoningEffort::High, "\"high\""),
            (ReasoningEffort::XHigh, "\"xhigh\""),
        ] {
            let s = serde_json::to_string(&variant).unwrap();
            assert_eq!(s, expected, "wire format drift on {variant:?}");
            let back: ReasoningEffort = serde_json::from_str(&s).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn service_tier_wire_format_is_lowercase() {
        // Pinned to match gts.cf.llmgw.core.create_response_body.v1~.
        for (variant, expected) in [
            (ServiceTier::Auto, "\"auto\""),
            (ServiceTier::Default, "\"default\""),
        ] {
            let s = serde_json::to_string(&variant).unwrap();
            assert_eq!(s, expected, "wire format drift on {variant:?}");
            let back: ServiceTier = serde_json::from_str(&s).unwrap();
            assert_eq!(back, variant);
        }
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
        assert_ne!(SupportedApi::Completion, SupportedApi::Batch);
        assert_ne!(SupportedApi::Embedding, SupportedApi::Batch);

        let mut set = std::collections::HashSet::new();
        set.insert(SupportedApi::Completion);
        set.insert(SupportedApi::Embedding);
        set.insert(SupportedApi::Batch);
        set.insert(SupportedApi::Completion);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn supported_api_batch_can_coexist() {
        // A model that supports both synchronous chat and the batch API
        // carries both variants in `supported_api`.
        let api = [SupportedApi::Completion, SupportedApi::Batch];
        assert!(api.contains(&SupportedApi::Completion));
        assert!(api.contains(&SupportedApi::Batch));
        assert!(!api.contains(&SupportedApi::Embedding));
    }

    #[test]
    fn reasoning_effort_equality() {
        assert_eq!(ReasoningEffort::High, ReasoningEffort::High);
        assert_ne!(ReasoningEffort::Low, ReasoningEffort::XHigh);
        assert_ne!(ReasoningEffort::None, ReasoningEffort::Medium);
    }

    #[test]
    fn service_tier_equality() {
        assert_eq!(ServiceTier::Auto, ServiceTier::Auto);
        assert_eq!(ServiceTier::Default, ServiceTier::Default);
        assert_ne!(ServiceTier::Auto, ServiceTier::Default);
    }

    #[test]
    fn service_tier_only_two_variants() {
        // The unified ServiceTier is intentionally narrowed to Auto | Default;
        // provider-specific tiers (OpenAI flex/priority) live on per-provider
        // settings as a distinct enum.
        let auto = ServiceTier::Auto;
        let default = ServiceTier::Default;
        // Exhaustive match — adding a variant would force this test to fail.
        match auto {
            ServiceTier::Auto | ServiceTier::Default => {}
        }
        assert_ne!(auto, default);
    }

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
    fn web_search_with_domain_support() {
        let ws = WebSearchCapability {
            enabled: true,
            allowed_domains: true,
            excluded_domains: true,
        };
        assert!(ws.enabled);
        assert!(ws.allowed_domains);
        assert!(ws.excluded_domains);
    }

    #[test]
    fn web_search_no_domain_support() {
        let ws = WebSearchCapability {
            enabled: true,
            allowed_domains: false,
            excluded_domains: false,
        };
        assert!(ws.enabled);
        assert!(!ws.allowed_domains);
        assert!(!ws.excluded_domains);
    }

    #[test]
    fn web_search_allow_only() {
        // A model may support allow-lists but not deny-lists (or vice versa);
        // the two flags are independent.
        let ws = WebSearchCapability {
            enabled: true,
            allowed_domains: true,
            excluded_domains: false,
        };
        assert!(ws.allowed_domains);
        assert!(!ws.excluded_domains);
    }

    #[test]
    fn media_capability_default_is_disabled_and_empty() {
        let m = MediaCapability::default();
        assert!(!m.enabled);
        assert!(m.supported_mime_types.is_empty());
    }

    #[test]
    fn media_capability_round_trip_serde() {
        let m = MediaCapability {
            enabled: true,
            supported_mime_types: vec!["image/png".into(), "image/jpeg".into()],
        };
        let v = serde_json::to_value(&m).unwrap();
        let back: MediaCapability = serde_json::from_value(v).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn model_capabilities_holds_media_typed_fields() {
        // Smoke test that the new MediaCapability fields can be set per-shape.
        let caps = ModelCapabilities {
            vision: MediaCapability {
                enabled: true,
                supported_mime_types: vec!["image/png".into()],
            },
            reasoning: ReasoningCapability {
                effort: false,
                toggle: false,
                resume: false,
                budget: false,
            },
            function_calling: true,
            response_schema: false,
            streaming: true,
            file_input: MediaCapability {
                enabled: true,
                supported_mime_types: vec!["application/pdf".into()],
            },
            image_generation: MediaCapability::default(),
            audio_input: MediaCapability {
                enabled: true,
                supported_mime_types: vec!["audio/mpeg".into(), "audio/wav".into()],
            },
            audio_output: MediaCapability::default(),
            code_interpreter: false,
            web_search: WebSearchCapability {
                enabled: true,
                allowed_domains: true,
                excluded_domains: false,
            },
        };
        assert!(caps.vision.enabled);
        assert_eq!(caps.audio_input.supported_mime_types.len(), 2);
        assert!(!caps.image_generation.enabled);
        assert!(caps.image_generation.supported_mime_types.is_empty());
    }

    #[test]
    fn context_window_completion_model() {
        let cw = ContextWindow {
            max_input_tokens: 128_000,
            max_output_tokens: Some(16_384),
            output_vector_size: None,
        };
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
}
