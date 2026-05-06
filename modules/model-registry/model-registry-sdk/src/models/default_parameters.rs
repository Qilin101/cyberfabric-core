// Created: 2026-05-07 by Constructor Tech
//! User-facing default inference parameters for a model.
//!
//! [`DefaultInferenceParametersV1`] mirrors the inference-knob subset of the
//! Open Responses request schema (`gts.cf.llmgw.core.create_response_body.v1~`)
//! so the LLM Gateway has a uniform input contract regardless of the
//! underlying provider. The provider-wire defaults that ride alongside this
//! shape (different naming, mutually-exclusive variants, provider-only knobs)
//! live on the per-provider settings payload (one typed struct per provider,
//! versioned independently from the envelope) and are intentionally kept
//! distinct: field names that look universal (`temperature`, `top_p`,
//! `max_output_tokens`) are duplicated on the two surfaces because they are
//! rarely 1:1 with provider-wire parameters in practice.
//!
//! The override policy itself (`allow_parameter_override`,
//! `allow_extra_params`) is **not** part of this struct — those are flat
//! fields on [`crate::models::ModelInfoV1`] applied uniformly across all
//! provider variants.

use crate::models::{ReasoningEffort, ServiceTier};

// ---------------------------------------------------------------------------
// DefaultInferenceParametersV1
// ---------------------------------------------------------------------------

/// Default inference parameters in the unified, user-facing shape.
///
/// All fields are optional — an absent field signals "no default" and the
/// caller's request value (or the provider-wire default on the per-provider
/// settings struct) wins at send time.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct DefaultInferenceParametersV1 {
    /// Sampling temperature. No min/max constraints — provider ranges differ.
    pub temperature: Option<f64>,
    /// Nucleus sampling.
    pub top_p: Option<f64>,
    /// Maximum number of output tokens.
    pub max_output_tokens: Option<u32>,
    /// Maximum number of tool calls per response.
    pub max_tool_calls: Option<u32>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    /// Top log-probabilities to return per token.
    pub top_logprobs: Option<u8>,
    /// Context-truncation strategy.
    pub truncation: Option<TruncationStrategy>,
    /// Service tier in the unified two-variant shape (`Auto | Default`).
    /// Provider-specific tiers (e.g. `OpenAI` `flex`/`priority`) are expressed
    /// at request time via the override-extras allowlist and live on the
    /// per-provider settings as `OpenAiServiceTier`.
    pub service_tier: Option<ServiceTier>,
    /// Whether the model may issue multiple tool calls in parallel.
    pub parallel_tool_calls: Option<bool>,
    /// Response text-format configuration.
    pub text: Option<TextFormat>,
    /// Reasoning controls.
    pub reasoning: Option<ReasoningConfig>,
    /// Tool-choice policy in the unified shape.
    pub tool_choice: Option<ToolChoice>,
    /// Whether to store the response for later retrieval.
    pub store: Option<bool>,
}

// ---------------------------------------------------------------------------
// TextFormat
// ---------------------------------------------------------------------------

/// Response text format configuration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TextFormat {
    pub format: TextFormatKind,
    pub verbosity: Option<TextVerbosity>,
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            format: TextFormatKind::Text,
            verbosity: None,
        }
    }
}

/// Concrete text-format variant.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum TextFormatKind {
    /// Plain text — provider default.
    #[default]
    Text,
    /// JSON-mode (no schema constraint).
    JsonObject,
    /// Schema-constrained JSON output.
    JsonSchema {
        name: String,
        description: Option<String>,
        schema: Option<serde_json::Value>,
        strict: bool,
    },
}

/// Verbosity level for the response text.
///
/// Wire format is lowercase (`"low" | "medium" | "high"`) to match
/// `gts.cf.llmgw.core.text_format.v1~`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum TextVerbosity {
    Low,
    Medium,
    High,
}

// ---------------------------------------------------------------------------
// ReasoningConfig
// ---------------------------------------------------------------------------

/// Reasoning controls in the unified shape — matches
/// `gts.cf.llmgw.core.reasoning_config.v1~`.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ReasoningConfig {
    /// Reasoning effort level.
    pub effort: Option<ReasoningEffort>,
    /// Reasoning summary mode.
    pub summary: Option<ReasoningSummary>,
}

/// Reasoning summary mode.
///
/// Wire format is lowercase (`"concise" | "detailed" | "auto"`) to match
/// `gts.cf.llmgw.core.reasoning_config.v1~`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    Concise,
    Detailed,
    Auto,
}

// ---------------------------------------------------------------------------
// ToolChoice
// ---------------------------------------------------------------------------

/// Tool-choice policy in the unified Open Responses shape.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub enum ToolChoice {
    /// Provider picks (default).
    Auto,
    /// Provider must call exactly one tool.
    Required,
    /// No tool calling.
    None,
    /// Force a specific tool by name.
    Function { name: String },
}

// ---------------------------------------------------------------------------
// TruncationStrategy
// ---------------------------------------------------------------------------

/// Context-truncation strategy.
///
/// Wire format is lowercase (`"auto" | "disabled"`) to match
/// `gts.cf.llmgw.core.create_response_body.v1~`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum TruncationStrategy {
    Auto,
    Disabled,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_inference_parameters_default_is_empty() {
        let p = DefaultInferenceParametersV1::default();
        assert!(p.temperature.is_none());
        assert!(p.top_p.is_none());
        assert!(p.max_output_tokens.is_none());
        assert!(p.max_tool_calls.is_none());
        assert!(p.presence_penalty.is_none());
        assert!(p.frequency_penalty.is_none());
        assert!(p.top_logprobs.is_none());
        assert!(p.truncation.is_none());
        assert!(p.service_tier.is_none());
        assert!(p.parallel_tool_calls.is_none());
        assert!(p.text.is_none());
        assert!(p.reasoning.is_none());
        assert!(p.tool_choice.is_none());
        assert!(p.store.is_none());
    }

    #[test]
    fn truncation_strategy_variants() {
        assert_eq!(TruncationStrategy::Auto, TruncationStrategy::Auto);
        assert_ne!(TruncationStrategy::Auto, TruncationStrategy::Disabled);
    }

    #[test]
    fn text_verbosity_variants() {
        assert_ne!(TextVerbosity::Low, TextVerbosity::High);
        assert_eq!(TextVerbosity::Medium, TextVerbosity::Medium);
    }

    #[test]
    fn unified_enums_wire_format_is_lowercase() {
        // Pinned wire format — must stay lowercase to match the
        // gateway-side gts.cf.llmgw.core.* schemas. If a future change
        // drops the `#[serde(rename_all = "lowercase")]` annotation the
        // gateway will reject our payloads.
        assert_eq!(
            serde_json::to_string(&TextVerbosity::Medium).unwrap(),
            "\"medium\""
        );
        assert_eq!(
            serde_json::to_string(&ReasoningSummary::Concise).unwrap(),
            "\"concise\""
        );
        assert_eq!(
            serde_json::to_string(&TruncationStrategy::Disabled).unwrap(),
            "\"disabled\""
        );

        // Round-trip back from lowercase.
        let v: TextVerbosity = serde_json::from_str("\"high\"").unwrap();
        assert_eq!(v, TextVerbosity::High);
        let r: ReasoningSummary = serde_json::from_str("\"auto\"").unwrap();
        assert_eq!(r, ReasoningSummary::Auto);
        let t: TruncationStrategy = serde_json::from_str("\"auto\"").unwrap();
        assert_eq!(t, TruncationStrategy::Auto);
    }

    #[test]
    fn text_format_kind_variants() {
        let txt = TextFormatKind::Text;
        let json = TextFormatKind::JsonObject;
        let schema = TextFormatKind::JsonSchema {
            name: "Person".into(),
            description: Some("A person record".into()),
            schema: Some(serde_json::json!({"type": "object"})),
            strict: true,
        };
        assert_ne!(txt, json);
        assert_ne!(json, schema);
        if let TextFormatKind::JsonSchema { name, strict, .. } = schema {
            assert_eq!(name, "Person");
            assert!(strict);
        }
    }

    #[test]
    fn text_format_default_is_text_no_verbosity() {
        let t = TextFormat::default();
        assert_eq!(t.format, TextFormatKind::Text);
        assert!(t.verbosity.is_none());
    }

    #[test]
    fn reasoning_summary_variants() {
        assert_ne!(ReasoningSummary::Concise, ReasoningSummary::Detailed);
        assert_eq!(ReasoningSummary::Auto, ReasoningSummary::Auto);
    }

    #[test]
    fn reasoning_config_default_is_empty() {
        let r = ReasoningConfig::default();
        assert!(r.effort.is_none());
        assert!(r.summary.is_none());
    }

    #[test]
    fn tool_choice_variants() {
        let auto = ToolChoice::Auto;
        let req = ToolChoice::Required;
        let none = ToolChoice::None;
        let fun = ToolChoice::Function {
            name: "search".into(),
        };
        assert_ne!(auto, req);
        assert_ne!(none, fun);
        if let ToolChoice::Function { name } = fun {
            assert_eq!(name, "search");
        }
    }

    #[test]
    fn populated_round_trip_serde() {
        let p = DefaultInferenceParametersV1 {
            temperature: Some(0.7),
            top_p: Some(0.95),
            max_output_tokens: Some(4096),
            max_tool_calls: Some(8),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.1),
            top_logprobs: Some(5),
            truncation: Some(TruncationStrategy::Auto),
            service_tier: Some(ServiceTier::Default),
            parallel_tool_calls: Some(true),
            text: Some(TextFormat {
                format: TextFormatKind::JsonObject,
                verbosity: Some(TextVerbosity::Medium),
            }),
            reasoning: Some(ReasoningConfig {
                effort: Some(ReasoningEffort::Medium),
                summary: Some(ReasoningSummary::Auto),
            }),
            tool_choice: Some(ToolChoice::Function {
                name: "lookup".into(),
            }),
            store: Some(false),
        };
        let serialized = serde_json::to_value(&p).expect("serialize");
        let round_tripped: DefaultInferenceParametersV1 =
            serde_json::from_value(serialized).expect("deserialize");
        assert_eq!(p, round_tripped);
    }
}
