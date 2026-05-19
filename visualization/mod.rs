pub mod graph;
pub mod export;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::Display;
use petgraph::{Graph, stable_graph::NodeIndex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlNode {
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub depth: usize,
    pub parent_url: Option<String>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlEdge {
    pub source: String,
    pub target: String,
    pub link_type: LinkType,
    pub anchor_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    Direct,
    JavaScript,
    Form,
    Sitemap,
    Robots,
    External,
}

impl Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkType::Direct => write!(f, "Direct"),
            LinkType::JavaScript => write!(f, "JavaScript"),
            LinkType::Form => write!(f, "Form"),
            LinkType::Sitemap => write!(f, "Sitemap"),
            LinkType::Robots => write!(f, "Robots"),
            LinkType::External => write!(f, "External"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityNode {
    pub id: String,
    pub finding_type: String,
    pub severity: String,
    pub description: String,
    pub url: String,
    pub evidence: String,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdomainNode {
    pub subdomain: String,
    pub base_domain: String,
    pub source: String,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3BucketNode {
    pub bucket_url: String,
    pub base_domain: String,
    pub verified: bool,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

pub trait VisualizationEngine {
    fn create_url_graph(&self, urls: &[UrlNode], edges: &[UrlEdge]) -> Result<UrlGraph>;
    fn create_security_graph(&self, findings: &[SecurityNode]) -> Result<SecurityGraph>;
    fn create_domain_map(&self, subdomains: &[SubdomainNode]) -> Result<DomainMap>;
    fn create_topology_view(&self, data: &CrawlData) -> Result<TopologyView>;
    fn export_graph(&self, graph: &dyn GraphExport, format: ExportFormat) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct UrlGraph {
    graph: Graph<UrlNode, UrlEdge>,
    node_indices: HashMap<String, NodeIndex>,
}

impl UrlGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: UrlNode) -> NodeIndex {
        let index = self.graph.add_node(node.clone());
        self.node_indices.insert(node.url.clone(), index);
        index
    }

    pub fn add_edge(&mut self, edge: UrlEdge) -> Option<NodeIndex> {
        let source_idx = self.node_indices.get(&edge.source)?;
        let target_idx = self.node_indices.get(&edge.target)?;
        self.graph.add_edge(*source_idx, *target_idx, edge);
        Some(*source_idx)
    }

    pub fn get_node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn get_edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn find_shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let from_idx = self.node_indices.get(from)?;
        let to_idx = self.node_indices.get(to)?;
        
        let path_result = petgraph::algo::astar(
            &self.graph,
            *from_idx,
            |finish| finish == *to_idx,
            |_e| 1.0,
            |_| 0.0,
        );
        
        if let Some((_, path)) = path_result {
            let result: Vec<String> = path.iter()
                .map(|&node_idx| self.graph[node_idx].url.clone())
                .collect();
            Some(result)
        } else {
            None
        }
    }

    pub fn get_connected_components(&self) -> Vec<Vec<String>> {
        let mut components = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        for node_idx in self.graph.node_indices() {
            if !visited.contains(&node_idx) {
                // Simple DFS implementation since petgraph::algo::dfs may have changed
                let mut component_nodes = Vec::new();
                let mut stack = vec![node_idx];
                let mut local_visited = std::collections::HashSet::new();
                
                while let Some(current) = stack.pop() {
                    if local_visited.insert(current) {
                        component_nodes.push(current);
                        // Add neighbors to stack
                        for neighbor in self.graph.neighbors(current) {
                            if !local_visited.contains(&neighbor) {
                                stack.push(neighbor);
                            }
                        }
                    }
                }
                
                for node in &component_nodes {
                    visited.insert(*node);
                }
                
                let component_urls: Vec<String> = component_nodes
                    .into_iter()
                    .map(|idx| self.graph[idx].url.clone())
                    .collect();
                
                components.push(component_urls);
            }
        }
        
        components
    }

    pub fn get_node_centrality(&self) -> HashMap<String, f64> {
        let mut centrality = HashMap::new();
        
        for node_idx in self.graph.node_indices() {
            let degree = self.graph.edges(node_idx).count();
            let centrality_score = degree as f64 / self.graph.node_count() as f64;
            centrality.insert(self.graph[node_idx].url.clone(), centrality_score);
        }
        
        centrality
    }
}

#[derive(Debug, Clone)]
pub struct SecurityGraph {
    findings: Vec<SecurityNode>,
    severity_groups: HashMap<String, Vec<usize>>,
}

impl SecurityGraph {
    pub fn new(findings: Vec<SecurityNode>) -> Self {
        let mut severity_groups = HashMap::new();
        
        for (i, finding) in findings.iter().enumerate() {
            severity_groups.entry(finding.severity.clone()).or_insert_with(Vec::new).push(i);
        }
        
        Self {
            findings,
            severity_groups,
        }
    }

    pub fn get_findings_by_severity(&self, severity: &str) -> Vec<&SecurityNode> {
        self.severity_groups
            .get(severity)
            .map(|indices| indices.iter().map(|&i| &self.findings[i]).collect())
            .unwrap_or_default()
    }

    pub fn get_severity_distribution(&self) -> HashMap<String, usize> {
        self.severity_groups
            .iter()
            .map(|(severity, indices)| (severity.clone(), indices.len()))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DomainMap {
    base_domain: String,
    subdomains: Vec<SubdomainNode>,
    relationships: HashMap<String, Vec<String>>,
}

impl DomainMap {
    pub fn new(base_domain: String, subdomains: Vec<SubdomainNode>) -> Self {
        let mut relationships = HashMap::new();
        
        for subdomain in &subdomains {
            relationships.insert(subdomain.subdomain.clone(), Vec::new());
        }
        
        Self {
            base_domain,
            subdomains,
            relationships,
        }
    }

    pub fn add_relationship(&mut self, from: String, to: String) {
        self.relationships.entry(from).or_insert_with(Vec::new).push(to);
    }

    pub fn get_subdomain_count(&self) -> usize {
        self.subdomains.len()
    }

    pub fn get_subdomains_by_source(&self, source: &str) -> Vec<&SubdomainNode> {
        self.subdomains
            .iter()
            .filter(|s| s.source == source)
            .collect()
    }

    /// Get the base domain (uses the base_domain field)
    pub fn get_base_domain(&self) -> &str {
        &self.base_domain
    }

    /// Check if a subdomain belongs to this domain map
    pub fn contains_subdomain(&self, subdomain: &str) -> bool {
        self.subdomains.iter().any(|s| s.subdomain == subdomain)
    }

    /// Get subdomains that are actually subdomains of the base domain
    pub fn get_valid_subdomains(&self) -> Vec<&SubdomainNode> {
        self.subdomains
            .iter()
            .filter(|s| s.subdomain.ends_with(&self.base_domain))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TopologyView {
    url_graph: UrlGraph,
    security_graph: SecurityGraph,
    domain_map: DomainMap,
    s3_buckets: Vec<S3BucketNode>,
}

impl TopologyView {
    pub fn new(
        url_graph: UrlGraph,
        security_graph: SecurityGraph,
        domain_map: DomainMap,
        s3_buckets: Vec<S3BucketNode>,
    ) -> Self {
        Self {
            url_graph,
            security_graph,
            domain_map,
            s3_buckets,
        }
    }

    pub fn get_overview_stats(&self) -> TopologyStats {
        TopologyStats {
            total_urls: self.url_graph.get_node_count(),
            total_links: self.url_graph.get_edge_count(),
            total_security_findings: self.security_graph.findings.len(),
            total_subdomains: self.domain_map.get_subdomain_count(),
            total_s3_buckets: self.s3_buckets.len(),
            connected_components: self.url_graph.get_connected_components().len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyStats {
    pub total_urls: usize,
    pub total_links: usize,
    pub total_security_findings: usize,
    pub total_subdomains: usize,
    pub total_s3_buckets: usize,
    pub connected_components: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlData {
    pub urls: Vec<UrlNode>,
    pub edges: Vec<UrlEdge>,
    pub security_findings: Vec<SecurityNode>,
    pub subdomains: Vec<SubdomainNode>,
    pub s3_buckets: Vec<S3BucketNode>,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Dot,
    Graphviz,
    Mermaid,
    Json,
    Csv,
}

pub trait GraphExport {
    fn to_dot(&self) -> Result<String>;
    fn to_mermaid(&self) -> Result<String>;
    fn to_json(&self) -> Result<serde_json::Value>;
    fn to_csv(&self) -> Result<String>;
}
