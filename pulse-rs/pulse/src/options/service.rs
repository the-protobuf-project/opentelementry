//! Service configuration options.
//!
//! This module defines service metadata and deployment environment configuration.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Deployment environment types.
///
/// Represents different deployment environments for service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Jetson,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
            Environment::Jetson => write!(f, "jetson"),
        }
    }
}

/// Service configuration options.
///
/// Contains metadata about the service including name, version, and environment.
///
/// # Examples
///
/// ```no_run
/// use pulse::options::{ServiceOptions, Environment};
///
/// let opts = ServiceOptions::new("my-service", "1.0.0")
///     .with_description("My service description")
///     .with_environment(Environment::Production);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOptions {
    pub name: String,
    pub description: String,
    pub version: String,
    pub environment: Environment,
    /// Global attributes added to ALL telemetry (logs, metrics, traces).
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

impl ServiceOptions {
    /// Creates new service options with name and version.
    ///
    /// # Arguments
    ///
    /// * `name` - Service name
    /// * `version` - Service version
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            version: version.into(),
            environment: Environment::default(),
            attributes: HashMap::new(),
        }
    }

    /// Sets the service description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the deployment environment.
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Sets global attributes that will be added to all telemetry.
    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Adds a single attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}
