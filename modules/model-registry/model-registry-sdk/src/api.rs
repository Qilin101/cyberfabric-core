// Created: 2026-04-17 by Constructor Tech
//! Public API trait for the `model-registry` module.
//!
//! [`ModelRegistryClientV1`] is registered in `ClientHub` by the module:
//! ```ignore
//! let mr = hub.get::<dyn ModelRegistryClientV1>()?;
//! ```

use async_trait::async_trait;
use modkit_security::SecurityContext;
use uuid::Uuid;

#[cfg(feature = "odata")]
use modkit_odata::{ODataQuery, Page};

use crate::errors::ModelRegistryError;
use crate::models::{
    Alias, CreateAliasRequest, CreateProviderRequest, DiscoveryResult, Model, Provider,
    ProviderHealth, UpdateProviderRequest,
};

/// Public API trait for the Model Registry (Version 1).
///
/// This trait is registered in `ClientHub` by the model-registry module:
/// ```ignore
/// let mr = hub.get::<dyn ModelRegistryClientV1>()?;
/// let model = mr.get_tenant_model(ctx, "openai::gpt-4o").await?;
/// ```
///
/// All methods require `SecurityContext` for tenant scoping and authorization.
#[async_trait]
pub trait ModelRegistryClientV1: Send + Sync {
    // ==================== Models (P1) ====================

    /// Get a model by canonical ID within the caller's tenant context.
    ///
    /// Returns the model with its approval status resolved via Approval
    /// Service. Uses cache-first lookup with DB fallback.
    async fn get_tenant_model(
        &self,
        ctx: &SecurityContext,
        canonical_id: &str,
    ) -> Result<Model, ModelRegistryError>;

    /// List models available to the caller's tenant with `OData` filtering.
    ///
    /// Supports `$filter` on: `lifecycle_status`, `approval_status`,
    /// `info.provider_settings.kind`, `info.supported_api`,
    /// `info.provider_model_id`, `info.capabilities.*` (e.g. `vision`,
    /// `function_calling`, `streaming`, `reasoning.effort`), `info.vendor`,
    /// `info.family`. Per-provider parameter and cost fields are not
    /// filterable in v1 — see `docs/DESIGN.md` §3.3.
    ///
    /// Returns `Model<AnyProviderSettings>`. Consumers narrowed to a
    /// specific provider (e.g. when they've already filtered on
    /// `info.provider_settings.kind eq 'openai'`) can call
    /// [`Model::try_into_typed`] on each result.
    #[cfg(feature = "odata")]
    async fn list_tenant_models(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Model>, ModelRegistryError>;

    // ==================== Providers (P1) ====================

    /// Get a provider by ID.
    async fn get_provider(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Provider, ModelRegistryError>;

    /// List providers for the caller's tenant with `OData` filtering.
    #[cfg(feature = "odata")]
    async fn list_providers(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Provider>, ModelRegistryError>;

    /// Register a new provider for the caller's tenant.
    async fn create_provider(
        &self,
        ctx: &SecurityContext,
        req: CreateProviderRequest,
    ) -> Result<Provider, ModelRegistryError>;

    /// Update a provider (PATCH semantics).
    async fn update_provider(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        req: UpdateProviderRequest,
    ) -> Result<Provider, ModelRegistryError>;

    /// Delete a provider by ID.
    async fn delete_provider(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<(), ModelRegistryError>;

    /// Trigger model discovery for a provider via OAGW.
    async fn trigger_discovery(
        &self,
        ctx: &SecurityContext,
        provider_id: Uuid,
    ) -> Result<DiscoveryResult, ModelRegistryError>;

    // ==================== Provider Health (P2) ====================

    /// Get health status for a provider's discovery endpoint.
    async fn get_provider_health(
        &self,
        ctx: &SecurityContext,
        provider_id: Uuid,
    ) -> Result<ProviderHealth, ModelRegistryError>;

    // ==================== Aliases (P2) ====================

    /// List aliases for the caller's tenant with `OData` filtering.
    #[cfg(feature = "odata")]
    async fn list_aliases(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Alias>, ModelRegistryError>;

    /// Create a model alias.
    async fn create_alias(
        &self,
        ctx: &SecurityContext,
        req: CreateAliasRequest,
    ) -> Result<Alias, ModelRegistryError>;

    /// Delete a model alias by name.
    async fn delete_alias(
        &self,
        ctx: &SecurityContext,
        name: &str,
    ) -> Result<(), ModelRegistryError>;
}
