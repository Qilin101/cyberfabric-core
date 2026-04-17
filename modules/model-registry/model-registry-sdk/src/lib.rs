// Created: 2026-04-17 by Constructor Tech
//! Model Registry SDK
//!
//! This crate provides the public API for the `model-registry` module:
//! - [`ModelRegistryClientV1`] trait for inter-module communication
//! - Model types ([`Provider`], [`Model`], etc.)
//! - Error type ([`ModelRegistryError`])
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use model_registry_sdk::ModelRegistryClientV1;
//!
//! let client = hub.get::<dyn ModelRegistryClientV1>()?;
//! let model = client.get_tenant_model(ctx, "openai::gpt-4o").await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod errors;
pub mod models;

pub use api::ModelRegistryClientV1;
pub use errors::ModelRegistryError;
pub use models::{
    Alias, ApiResolution, ApprovalStatus, ContextWindow, CreateAliasRequest, CreateProviderRequest,
    CreateProviderRequestBuilder, DiscoveryResult, LifecycleStatus, Model, ModelCapabilities,
    ModelCost, ModelInfo, ModelParameters, ModelPerformance, Provider, ProviderHealth,
    ProviderHealthStatus, ProviderStatus, ReasoningCapability, ReasoningEffort, ServiceTier,
    SupportedApi, UpdateProviderRequest, WebSearchCapability,
};
