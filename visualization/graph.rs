use anyhow::Result;
use std::collections::HashMap;
use crate::visualization::{UrlGraph, UrlNode, UrlEdge};

pub struct GraphBuilder {
    url_graph: UrlGraph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            url_graph: UrlGraph::new(),
        }
    }

    pub fn build_from_crawl_data(&mut self, urls: Vec<UrlNode>, edges: Vec<UrlEdge>) -> Result<()> {
        // Add all nodes first
        for url_node in urls {
            self.url_graph.add_node(url_node);
        }

        // Add all edges
        for edge in edges {
            self.url_graph.add_edge(edge);
        }

        Ok(())
    }

    pub fn build_interactive_graph(&self) -> Result<InteractiveGraph> {
        let nodes = self.url_graph.graph
            .node_indices()
            .map(|idx| {
                let node = &self.url_graph.graph[idx];
                GraphNode {
                    id: node.url.clone(),
                    label: node.title.clone().unwrap_or_else(|| node.url.clone()),
                    group: self.determine_node_group(node),
                    x: 0.0,
                    y: 0.0,
                    data: node.clone(),
                }
            })
            .collect();

        let edges = self.url_graph.graph
            .edge_indices()
            .map(|edge_idx| {
                let (source, target) = self.url_graph.graph.edge_endpoints(edge_idx).unwrap();
                let edge_data = &self.url_graph.graph[edge_idx];
                
                GraphEdge {
                    source: self.url_graph.graph[source].url.clone(),
                    target: self.url_graph.graph[target].url.clone(),
                    label: edge_data.anchor_text.clone().unwrap_or_else(|| edge_data.link_type.to_string()),
                    data: edge_data.clone(),
                }
            })
            .collect();

        Ok(InteractiveGraph::new(nodes, edges))
    }

    fn determine_node_group(&self, node: &UrlNode) -> String {
        if node.url.contains("/admin") || node.url.contains("/login") {
            "admin".to_string()
        } else if node.url.contains("/api") {
            "api".to_string()
        } else if node.content_type.as_ref().map_or(false, |ct| ct.contains("application/json")) {
            "json".to_string()
        } else if node.content_type.as_ref().map_or(false, |ct| ct.contains("text/html")) {
            "html".to_string()
        } else {
            "other".to_string()
        }
    }

    pub fn analyze_graph_structure(&self) -> GraphAnalysis {
        let node_count = self.url_graph.get_node_count();
        let edge_count = self.url_graph.get_edge_count();
        let connected_components = self.url_graph.get_connected_components();
        let centrality = self.url_graph.get_node_centrality();

        let density = if node_count > 1 {
            edge_count as f64 / (node_count * (node_count - 1)) as f64
        } else {
            0.0
        };

        let avg_degree = if node_count > 0 {
            (2 * edge_count) as f64 / node_count as f64
        } else {
            0.0
        };

        let central_nodes: Vec<String> = centrality
            .iter()
            .filter(|(_, score)| **score > 0.1)
            .map(|(url, _)| url.clone())
            .collect();

        GraphAnalysis {
            node_count,
            edge_count,
            density,
            avg_degree,
            connected_components_count: connected_components.len(),
            central_nodes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InteractiveGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    layout: GraphLayout,
}

impl InteractiveGraph {
    pub fn new(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) -> Self {
        let layout = GraphLayout::new();
        Self {
            nodes,
            edges,
            layout,
        }
    }

    pub fn calculate_layout(&mut self, algorithm: LayoutAlgorithm) {
        self.layout.calculate(&mut self.nodes, &self.edges, algorithm);
    }

    pub fn to_d3_data(&self) -> serde_json::Value {
        serde_json::json!({
            "nodes": self.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "label": n.label,
                    "group": n.group,
                    "x": n.x,
                    "y": n.y,
                    "data": n.data
                })
            }).collect::<Vec<_>>(),
            "edges": self.edges.iter().map(|e| {
                serde_json::json!({
                    "source": e.source,
                    "target": e.target,
                    "label": e.label,
                    "data": e.data
                })
            }).collect::<Vec<_>>()
        })
    }

    pub fn filter_by_group(&self, group: &str) -> InteractiveGraph {
        let filtered_nodes: Vec<GraphNode> = self.nodes
            .iter()
            .filter(|n| n.group == group)
            .cloned()
            .collect();

        let node_ids: std::collections::HashSet<&String> = filtered_nodes
            .iter()
            .map(|n| &n.id)
            .collect();

        let filtered_edges: Vec<GraphEdge> = self.edges
            .iter()
            .filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target))
            .cloned()
            .collect();

        InteractiveGraph::new(filtered_nodes, filtered_edges)
    }
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub group: String,
    pub x: f64,
    pub y: f64,
    pub data: UrlNode,
}

impl GraphNode {
    fn new(id: String, label: String, group: String) -> Self {
        let id_clone = id.clone();
        let label_clone = label.clone();
        Self {
            id: id.clone(),
            label: label.clone(),
            group,
            x: 0.0,
            y: 0.0,
            data: UrlNode {
                url: id_clone,
                title: Some(label_clone),
                status_code: None,
                content_type: None,
                depth: 0,
                parent_url: None,
                discovered_at: chrono::Utc::now(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: String,
    pub data: UrlEdge,
}

#[derive(Debug, Clone)]
pub struct GraphLayout {
    positions: HashMap<String, (f64, f64)>,
}

impl GraphLayout {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    /// Create nodes from URLs using the GraphNode::new constructor
    pub fn create_nodes_from_urls(urls: &[String]) -> Vec<GraphNode> {
        urls.iter().enumerate().map(|(i, url)| {
            let group = if url.starts_with("https://") {
                "secure".to_string()
            } else if url.starts_with("http://") {
                "insecure".to_string()
            } else {
                "other".to_string()
            };
            
            GraphNode::new(
                format!("node_{}", i),
                url.clone(),
                group
            )
        }).collect()
    }

    /// Get position for a specific node (uses the positions field)
    pub fn get_position(&self, node_id: &str) -> Option<(f64, f64)> {
        self.positions.get(node_id).copied()
    }

    /// Set position for a specific node
    pub fn set_position(&mut self, node_id: String, x: f64, y: f64) {
        self.positions.insert(node_id, (x, y));
    }

    pub fn calculate(&mut self, nodes: &mut [GraphNode], edges: &[GraphEdge], algorithm: LayoutAlgorithm) {
        match algorithm {
            LayoutAlgorithm::Force => self.force_directed_layout(nodes, edges),
            LayoutAlgorithm::Circular => self.circular_layout(nodes),
            LayoutAlgorithm::Hierarchical => self.hierarchical_layout(nodes, edges),
            LayoutAlgorithm::Random => self.random_layout(nodes),
        }
    }

    fn force_directed_layout(&mut self, nodes: &mut [GraphNode], edges: &[GraphEdge]) {
        let mut positions: HashMap<String, (f64, f64)> = HashMap::new();
        let nodes_count = nodes.len();
        
        // Initialize random positions
        for (i, node) in nodes.iter_mut().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / nodes_count as f64;
            let radius = 200.0;
            node.x = radius * angle.cos();
            node.y = radius * angle.sin();
            positions.insert(node.id.clone(), (node.x, node.y));
        }

        // Apply force-directed algorithm (simplified)
        for _ in 0..100 {
            let mut forces: HashMap<String, (f64, f64)> = HashMap::new();

            // Repulsive forces between all nodes
            for (i, node_i) in nodes.iter().enumerate() {
                for (j, node_j) in nodes.iter().enumerate() {
                    if i != j {
                        let dx = node_i.x - node_j.x;
                        let dy = node_i.y - node_j.y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        
                        if distance > 0.1 {
                            let force = 1000.0 / (distance * distance);
                            let fx = force * dx / distance;
                            let fy = force * dy / distance;
                            
                            let (existing_fx, existing_fy) = forces.get(&node_i.id).unwrap_or(&(0.0, 0.0));
                            forces.insert(node_i.id.clone(), (existing_fx + fx, existing_fy + fy));
                        }
                    }
                }
            }

            // Attractive forces for connected nodes
            for edge in edges {
                if let (Some(source_pos), Some(target_pos)) = 
                    (positions.get(&edge.source), positions.get(&edge.target)) {
                    let dx = target_pos.0 - source_pos.0;
                    let dy = target_pos.1 - source_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance > 0.1 {
                        let force = distance * 0.01;
                        let fx = force * dx / distance;
                        let fy = force * dy / distance;
                        
                        let (existing_fx, existing_fy) = forces.get(&edge.source).unwrap_or(&(0.0, 0.0));
                        forces.insert(edge.source.clone(), (existing_fx + fx, existing_fy + fy));
                        
                        let (existing_fx, existing_fy) = forces.get(&edge.target).unwrap_or(&(0.0, 0.0));
                        forces.insert(edge.target.clone(), (existing_fx - fx, existing_fy - fy));
                    }
                }
            }

            // Apply forces
            for node in nodes.iter_mut() {
                if let Some((fx, fy)) = forces.get(&node.id) {
                    node.x += fx * 0.01;
                    node.y += fy * 0.01;
                    positions.insert(node.id.clone(), (node.x, node.y));
                }
            }
        }
    }

    fn circular_layout(&mut self, nodes: &mut [GraphNode]) {
        let center_x = 0.0;
        let center_y = 0.0;
        let radius = 200.0;
        let nodes_count = nodes.len();

        for (i, node) in nodes.iter_mut().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / nodes_count as f64;
            node.x = center_x + radius * angle.cos();
            node.y = center_y + radius * angle.sin();
        }
    }

    fn hierarchical_layout(&mut self, nodes: &mut [GraphNode], _edges: &[GraphEdge]) {
        // Simplified hierarchical layout
        let mut levels: HashMap<usize, Vec<&mut GraphNode>> = HashMap::new();
        
        // Group nodes by depth
        for node in nodes.iter_mut() {
            let depth = node.data.depth;
            levels.entry(depth).or_insert_with(Vec::new).push(node);
        }

        // Position nodes by level
        let level_height = 150.0;
        for (depth, level_nodes) in levels {
            let node_width = 100.0;
            let total_width = level_nodes.len() as f64 * node_width;
            let start_x = -total_width / 2.0;

            for (i, node) in level_nodes.into_iter().enumerate() {
                node.x = start_x + i as f64 * node_width;
                node.y = depth as f64 * level_height;
            }
        }
    }

    fn random_layout(&mut self, nodes: &mut [GraphNode]) {
        for node in nodes.iter_mut() {
            node.x = (rand::random::<f64>() - 0.5) * 400.0;
            node.y = (rand::random::<f64>() - 0.5) * 400.0;
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutAlgorithm {
    Force,
    Circular,
    Hierarchical,
    Random,
}

#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub avg_degree: f64,
    pub connected_components_count: usize,
    pub central_nodes: Vec<String>,
}
