use std::collections::HashMap;
use crate::plugins::{PluginInfo, PluginType};

pub struct PluginRegistry {
    plugins: HashMap<String, PluginInfo>,
    by_type: HashMap<PluginType, Vec<String>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            by_type: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, info: PluginInfo) {
        let name = info.name.clone();
        
        self.by_type
            .entry(info.plugin_type.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());
        
        self.plugins.insert(name, info);
    }

    pub fn unregister_plugin(&mut self, name: &str) -> Option<PluginInfo> {
        if let Some(info) = self.plugins.remove(name) {
            if let Some(plugins) = self.by_type.get_mut(&info.plugin_type) {
                plugins.retain(|n| n != name);
            }
            Some(info)
        } else {
            None
        }
    }

    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }

    pub fn list_plugins_by_type(&self, plugin_type: &PluginType) -> Vec<&PluginInfo> {
        if let Some(names) = self.by_type.get(plugin_type) {
            names.iter()
                .filter_map(|name| self.plugins.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn find_plugins(&self, query: &str) -> Vec<&PluginInfo> {
        self.plugins
            .values()
            .filter(|info| {
                info.name.to_lowercase().contains(&query.to_lowercase()) ||
                info.description.to_lowercase().contains(&query.to_lowercase()) ||
                info.author.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }

    pub fn get_dependency_graph(&self) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        for info in self.plugins.values() {
            graph.add_node(info.name.clone());
            
            for dep in &info.dependencies {
                graph.add_dependency(info.name.clone(), dep.clone());
            }
        }
        
        graph
    }

    pub fn validate_dependencies(&self) -> Vec<DependencyError> {
        let mut errors = Vec::new();
        let available_plugins: std::collections::HashSet<&String> = self.plugins.keys().collect();
        
        for info in self.plugins.values() {
            for dep in &info.dependencies {
                if !available_plugins.contains(dep) {
                    errors.push(DependencyError {
                        plugin: info.name.clone(),
                        missing_dependency: dep.clone(),
                    });
                }
            }
        }
        
        errors
    }

    pub fn get_load_order(&self) -> Vec<String> {
        let graph = self.get_dependency_graph();
        graph.topological_sort()
    }

    pub fn export_registry(&self) -> RegistryExport {
        RegistryExport {
            plugins: self.plugins.clone(),
            metadata: RegistryMetadata {
                total_plugins: self.plugins.len(),
                plugin_types: self.get_plugin_type_counts(),
                last_updated: chrono::Utc::now(),
            },
        }
    }

    fn get_plugin_type_counts(&self) -> HashMap<PluginType, usize> {
        let mut counts = HashMap::new();
        
        for info in self.plugins.values() {
            *counts.entry(info.plugin_type.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    nodes: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: String) {
        self.nodes.entry(name).or_insert_with(Vec::new);
    }

    pub fn add_dependency(&mut self, plugin: String, dependency: String) {
        self.nodes.entry(plugin).or_insert_with(Vec::new).push(dependency);
    }

    pub fn topological_sort(&self) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();
        let mut result = Vec::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                if let Err(cycle) = self.dfs(node, &mut visited, &mut temp_visited, &mut result) {
                    eprintln!("Dependency cycle detected: {}", cycle);
                }
            }
        }

        result.reverse();
        result
    }

    fn dfs(
        &self,
        node: &str,
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<(), String> {
        if temp_visited.contains(node) {
            return Err(format!("Cycle detected at node: {}", node));
        }

        if visited.contains(node) {
            return Ok(());
        }

        temp_visited.insert(node.to_string());

        if let Some(dependencies) = self.nodes.get(node) {
            for dep in dependencies {
                self.dfs(dep, visited, temp_visited, result)?;
            }
        }

        temp_visited.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DependencyError {
    pub plugin: String,
    pub missing_dependency: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryExport {
    pub plugins: HashMap<String, PluginInfo>,
    pub metadata: RegistryMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryMetadata {
    pub total_plugins: usize,
    pub plugin_types: HashMap<PluginType, usize>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
