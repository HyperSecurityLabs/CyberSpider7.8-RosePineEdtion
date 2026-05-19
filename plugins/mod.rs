pub mod loader;
pub mod registry;
pub mod example;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::plugins::registry::PluginRegistry;
use crate::plugins::loader::PluginLoader;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn plugin_info(&self) -> PluginInfo;
    async fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    async fn execute(&mut self, context: &PluginContext) -> Result<PluginResult>;
    async fn cleanup(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
}

// FFI-safe version for C interface
#[repr(C)]
pub struct PluginInfoFFI {
    pub name: *const std::ffi::c_char,
    pub version: *const std::ffi::c_char,
    pub description: *const std::ffi::c_char,
    pub author: *const std::ffi::c_char,
    pub plugin_type: PluginType,
    pub dependencies: *const std::ffi::c_char,
    pub permissions: *const std::ffi::c_char,
}

impl PluginInfoFFI {
    // Helper to create from Rust strings
    pub fn from_info(info: &PluginInfo) -> Self {
        Self {
            name: std::ffi::CString::new(info.name.as_str()).unwrap().into_raw(),
            version: std::ffi::CString::new(info.version.as_str()).unwrap().into_raw(),
            description: std::ffi::CString::new(info.description.as_str()).unwrap().into_raw(),
            author: std::ffi::CString::new(info.author.as_str()).unwrap().into_raw(),
            plugin_type: info.plugin_type.clone(),
            dependencies: std::ffi::CString::new(info.dependencies.join(",")).unwrap().into_raw(),
            permissions: std::ffi::CString::new(info.permissions.join(",")).unwrap().into_raw(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[repr(C)]
pub enum PluginType {
    Detector,
    Processor,
    Output,
    Filter,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginContext {
    pub url: Option<String>,
    pub content: Option<String>,
    pub metadata: HashMap<String, String>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    registry: PluginRegistry,
    loader: PluginLoader,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            registry: PluginRegistry::new(),
            loader: PluginLoader::new(),
        }
    }

    pub async fn load_plugins_from_dir<P: AsRef<std::path::Path>>(&mut self, dir: P) -> Result<usize> {
        let dir_path = dir.as_ref();
        if !dir_path.exists() {
            return Ok(0);
        }

        let mut loaded_count = 0;
        
        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("so") || 
               path.extension().and_then(|s| s.to_str()) == Some("dll") ||
               path.extension().and_then(|s| s.to_str()) == Some("dylib") {
                
                if let Ok(plugin) = self.loader.load_dynamic_plugin(&path).await {
                    let info = plugin.plugin_info();
                    self.plugins.insert(info.name.clone(), plugin);
                    self.registry.register_plugin(info);
                    loaded_count += 1;
                }
            }
        }
        
        Ok(loaded_count)
    }

    pub async fn execute_plugin(&mut self, name: &str, context: &PluginContext) -> Result<PluginResult> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.execute(context).await
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }

    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.registry.get_plugin_info(name)
    }

    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.registry.list_plugins()
    }

    pub async fn initialize_all(&mut self, config: &HashMap<String, PluginConfig>) -> Result<()> {
        for (name, plugin) in &mut self.plugins {
            if let Some(plugin_config) = config.get(name) {
                if plugin_config.enabled {
                    plugin.initialize(plugin_config).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn cleanup_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values_mut() {
            plugin.cleanup().await?;
        }
        Ok(())
    }
}
