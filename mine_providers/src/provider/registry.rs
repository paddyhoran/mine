use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::RegistryError;

use super::ProviderTrait;

pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn ProviderTrait>>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, provider: Arc<dyn ProviderTrait>) -> Result<(), RegistryError> {
        let mut providers = self
            .providers
            .write()
            .map_err(|_| RegistryError::LockPoisoned)?;

        let id = provider.provider_id().to_string();

        if providers.contains_key(&id) {
            return Err(RegistryError::ProviderExists(id));
        }

        providers.insert(id, provider);
        Ok(())
    }

    pub fn get(&self, provider_id: &str) -> Result<Arc<dyn ProviderTrait>, RegistryError> {
        let providers = self
            .providers
            .read()
            .map_err(|_| RegistryError::LockPoisoned)?;

        providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| RegistryError::ProviderNotFound(provider_id.to_string()))
    }

    pub fn list(&self) -> Result<Vec<String>, RegistryError> {
        let providers = self
            .providers
            .read()
            .map_err(|_| RegistryError::LockPoisoned)?;

        Ok(providers.keys().cloned().collect())
    }

    pub fn unregister(&self, provider_id: &str) -> Result<(), RegistryError> {
        let mut providers = self
            .providers
            .write()
            .map_err(|_| RegistryError::LockPoisoned)?;

        providers
            .remove(provider_id)
            .ok_or_else(|| RegistryError::ProviderNotFound(provider_id.to_string()))?;

        Ok(())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_REGISTRY: once_cell::sync::Lazy<ProviderRegistry> =
    once_cell::sync::Lazy::new(ProviderRegistry::new);

pub fn global_registry() -> &'static ProviderRegistry {
    &GLOBAL_REGISTRY
}
