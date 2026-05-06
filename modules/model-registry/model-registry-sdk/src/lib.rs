// Created: 2026-04-17 by Constructor Tech
// Updated: 2026-05-07 by Constructor Tech
//! Model Registry SDK
//!
//! This crate provides the public API for the `model-registry` module:
//! - [`ModelRegistryClientV1`] trait for inter-module communication
//! - Generic [`Model<P>`] / [`ModelInfoV1<P>`] parameterized over a provider's
//!   typed settings (e.g. [`OpenAiSettingsV1`]); the set of shipped
//!   provider settings types is open-ended and lives in [`models::providers`]
//! - Default `P = `[`RawProviderSettings`] for heterogeneous lists; typed
//!   narrowing via `Model::try_into_typed::<P>()` resolves by GTS schema id
//!   (`info.gts_type` vs `P::SCHEMA_ID`)
//! - User-facing default inference parameters
//!   ([`DefaultInferenceParametersV1`]) plus supporting types
//!   ([`TextFormat`], [`ReasoningConfig`], [`ToolChoice`],
//!   [`TruncationStrategy`])
//! - Domain entities ([`Provider`], [`ProviderHealth`], [`Alias`], …)
//! - Error type ([`ModelRegistryError`])
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use model_registry_sdk::{ModelRegistryClientV1, OpenAiSettingsV1};
//!
//! let client = hub.get::<dyn ModelRegistryClientV1>()?;
//! let model = client.get_tenant_model(ctx, "openai::gpt-4o").await?;
//! // narrow to typed view if the caller knows the provider in advance
//! let typed = model.try_into_typed::<OpenAiSettingsV1>()?;
//! // flat access — no `connection.` / `parameters.` namespacing
//! let _ = typed.info.provider_settings.oagw_alias;
//! let _ = typed.info.provider_settings.temperature;
//! let _ = typed.info.default_parameters.temperature;
//! let _ = typed.info.allow_parameter_override;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod errors;
pub mod models;

pub use api::ModelRegistryClientV1;
pub use errors::ModelRegistryError;
pub use models::{
    // domain entities
    Alias,
    // anthropic provider — shipped today; additional providers can be added
    // in `models::providers` without touching the rest of the SDK
    AnthropicCost,
    AnthropicJsonOutputFormat,
    AnthropicOutputConfig,
    AnthropicOutputEffort,
    AnthropicServiceTier,
    AnthropicSettingsV1,
    AnthropicThinking,
    AnthropicThinkingDisplay,
    AnthropicToolChoice,
    ApprovalStatus,
    // common types
    ContextWindow,
    CreateAliasRequest,
    CreateProviderRequest,
    CreateProviderRequestBuilder,
    // default inference parameters (user-facing)
    DefaultInferenceParametersV1,
    DiscoveryResult,
    LifecycleStatus,
    // shared media-typed capability (vision, file_input, image_generation,
    // audio_input, audio_output)
    MediaCapability,
    Model,
    ModelCapabilities,
    // info / entities
    ModelInfoV1,
    ModelPerformance,
    // openai provider — shipped today
    OpenAiCost,
    OpenAiEmbeddingEncoding,
    OpenAiEndpoint,
    OpenAiPromptCacheRetention,
    OpenAiReasoningEffort,
    OpenAiResponseFormat,
    OpenAiServiceTier,
    OpenAiSettingsV1,
    Provider,
    ProviderHealth,
    ProviderHealthStatus,
    ProviderSchemaMismatch,
    // provider settings core
    ProviderSettings,
    ProviderStatus,
    // raw JSON carrier (default `P`)
    RawProviderSettings,
    ReasoningCapability,
    ReasoningConfig,
    ReasoningEffort,
    ReasoningSummary,
    ServiceTier,
    SupportedApi,
    TextFormat,
    TextFormatKind,
    TextVerbosity,
    ToolChoice,
    TruncationStrategy,
    UpdateProviderRequest,
    WebSearchCapability,
};
