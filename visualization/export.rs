use anyhow::Result;
use std::io::Write;
use crate::visualization::{UrlGraph, SecurityGraph, GraphExport};
use crate::visualization::graph::InteractiveGraph;

pub struct GraphExporter;

impl GraphExporter {
    pub fn export_to_png<P: AsRef<std::path::Path>>(
        graph: &dyn GraphExport,
        output_path: P,
        width: u32,
        height: u32,
    ) -> Result<()> {
        // Validate dimensions
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!("Invalid dimensions: {}x{}", width, height));
        }
        
        // Log export info
        eprintln!("Exporting graph to PNG with dimensions {}x{}", width, height);
        
        let dot_content = graph.to_dot()?;
        
        // Use graphviz to render PNG with size
        let size_arg = format!("-Gsize={},{}", width, height);
        let mut child = std::process::Command::new("dot")
            .args(&["-Tpng", &size_arg, "-o", output_path.as_ref().to_str().unwrap()])
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(dot_content.as_bytes())?;
        }

        child.wait()?;
        Ok(())
    }

    pub fn export_to_svg<P: AsRef<std::path::Path>>(
        graph: &dyn GraphExport,
        output_path: P,
    ) -> Result<()> {
        let dot_content = graph.to_dot()?;
        
        let mut child = std::process::Command::new("dot")
            .args(&["-Tsvg", "-o", output_path.as_ref().to_str().unwrap()])
                    .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(dot_content.as_bytes())?;
        }

        child.wait()?;
        Ok(())
    }

    pub fn export_interactive_html<P: AsRef<std::path::Path>>(
        graph: &InteractiveGraph,
             output_path: P,
    ) -> Result<()> {
        let d3_data = graph.to_d3_data();
        let html_content = Self::generate_html_template(d3_data);
        
        std::fs::write(output_path, html_content)?;
        Ok(())
    }

    fn generate_html_template(d3_data: serde_json::Value) -> String {
        let data_json = serde_json::to_string(&d3_data).unwrap();
        
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("<head>\n");
        html.push_str("    <title>CyberSpider Interactive Graph</title>\n");
        html.push_str("    <script src=\"https://d3js.org/d3.v7.min.js\"></script>\n");
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("        .node { cursor: pointer; }\n");
        html.push_str("        .link { stroke: #999; stroke-opacity: 0.6; }\n");
        html.push_str("        .tooltip { position: absolute; text-align: center; padding: 8px; font-size: 12px; background: #fff; border: 1px solid #ccc; border-radius: 4px; pointer-events: none; }\n");
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <h2>CyberSpider URL Graph</h2>\n");
        html.push_str("    <svg width=\"1200\" height=\"800\"></svg>\n");
        html.push_str("    <div class=\"tooltip\"></div>\n");
        html.push_str("    \n");
        html.push_str("    <script>\n");
        html.push_str("        const data = ");
        html.push_str(&data_json);
        html.push_str(";\n");
        html.push_str("        \n");
        html.push_str("        const svg = d3.select(\"svg\");\n");
        html.push_str("        const width = +svg.attr(\"width\");\n");
        html.push_str("        const height = +svg.attr(\"height\");\n");
        html.push_str("        \n");
        html.push_str("        const simulation = d3.forceSimulation(data.nodes)\n");
        html.push_str("            .force(\"link\", d3.forceLink(data.links).id(d => d.id).distance(100))\n");
        html.push_str("            .force(\"charge\", d3.forceManyBody().strength(-300))\n");
        html.push_str("            .force(\"center\", d3.forceCenter(width / 2, height / 2));\n");
        html.push_str("        \n");
        html.push_str("        const link = svg.append(\"g\")\n");
        html.push_str("            .selectAll(\"line\")\n");
        html.push_str("            .data(data.links)\n");
        html.push_str("            .enter().append(\"line\")\n");
        html.push_str("            .attr(\"class\", \"link\")\n");
        html.push_str("            .attr(\"stroke-width\", 2);\n");
        html.push_str("        \n");
        html.push_str("        const node = svg.append(\"g\")\n");
        html.push_str("            .selectAll(\"circle\")\n");
        html.push_str("            .data(data.nodes)\n");
        html.push_str("            .enter().append(\"circle\")\n");
        html.push_str("            .attr(\"class\", \"node\")\n");
        html.push_str("            .attr(\"r\", 8)\n");
        html.push_str("            .attr(\"fill\", d => {\n");
        html.push_str("                const colors = {\n");
        html.push_str("                    \"admin\": \"#ff6b6b\",\n");
        html.push_str("                    \"api\": \"#4ecdc4\",\n");
        html.push_str("                    \"html\": \"#45b7d1\",\n");
        html.push_str("                    \"json\": \"#f7b731\",\n");
        html.push_str("                    \"other\": \"#95afc0\"\n");
        html.push_str("                };\n");
        html.push_str("                return colors[d.group] || \"#95afc0\";\n");
        html.push_str("            })\n");
        html.push_str("            .call(d3.drag()\n");
        html.push_str("                .on(\"start\", dragstarted)\n");
        html.push_str("                .on(\"drag\", dragged)\n");
        html.push_str("                .on(\"end\", dragended));\n");
        html.push_str("        \n");
        html.push_str("        const tooltip = d3.select(\".tooltip\");\n");
        html.push_str("        \n");
        html.push_str("        node.on(\"mouseover\", function(event, d) {\n");
        html.push_str("            tooltip.transition().duration(200).style(\"opacity\", .9);\n");
        html.push_str("            tooltip.html(`<strong>${d.label}</strong><br/>URL: ${d.id}<br/>Group: ${d.group}`)\n");
        html.push_str("                .style(\"left\", (event.pageX + 10) + \"px\")\n");
        html.push_str("                .style(\"top\", (event.pageY - 28) + \"px\");\n");
        html.push_str("        })\n");
        html.push_str("        .on(\"mouseout\", function(d) {\n");
        html.push_str("            tooltip.transition().duration(500).style(\"opacity\", 0);\n");
        html.push_str("        });\n");
        html.push_str("        \n");
        html.push_str("        simulation.on(\"tick\", () => {\n");
        html.push_str("            link\n");
        html.push_str("                .attr(\"x1\", d => d.source.x)\n");
        html.push_str("                .attr(\"y1\", d => d.source.y)\n");
        html.push_str("                .attr(\"x2\", d => d.target.x)\n");
        html.push_str("                .attr(\"y2\", d => d.target.y);\n");
        html.push_str("            \n");
        html.push_str("            node\n");
        html.push_str("                .attr(\"cx\", d => d.x)\n");
        html.push_str("                .attr(\"cy\", d => d.y);\n");
        html.push_str("        });\n");
        html.push_str("        \n");
        html.push_str("        function dragstarted(event, d) {\n");
        html.push_str("            if (!event.active) simulation.alphaTarget(0.3).restart();\n");
        html.push_str("            d.fx = d.x;\n");
        html.push_str("            d.fy = d.y;\n");
        html.push_str("        }\n");
        html.push_str("        \n");
        html.push_str("        function dragged(event, d) {\n");
        html.push_str("            d.fx = event.x;\n");
        html.push_str("            d.fy = event.y;\n");
        html.push_str("        }\n");
        html.push_str("        \n");
        html.push_str("        function dragended(event, d) {\n");
        html.push_str("            if (!event.active) simulation.alphaTarget(0);\n");
        html.push_str("            d.fx = null;\n");
        html.push_str("            d.fy = null;\n");
        html.push_str("        }\n");
        html.push_str("    </script>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        html
    }
}

impl GraphExport for UrlGraph {
    fn to_dot(&self) -> Result<String> {
        let mut dot = String::new();
        dot.push_str("digraph CyberSpider {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=filled];\n");

        // Add nodes
        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            let color = self.get_node_color(node);
                let label = node.title.as_ref().unwrap_or(&node.url).replace("\"", "\\\"");
            
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n",
                node.url.replace("\"", "\\\""),
                label,
                color
            ));
        }

        // Add edges
        for edge_idx in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge_idx).unwrap();
            let edge = &self.graph[edge_idx];
            let color = self.get_edge_color(&edge.link_type);
            
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\", color=\"{}\"];\n",
                self.graph[source].url.replace("\"", "\\\""),
                self.graph[target].url.replace("\"", "\\\""),
                edge.anchor_text.as_ref().unwrap_or(&edge.link_type.to_string()).replace("\"", "\\\""),
                color
            ));
        }

        dot.push_str("}\n");
        Ok(dot)
    }

    fn to_mermaid(&self) -> Result<String> {
        let mut mermaid = String::new();
        mermaid.push_str("graph TD\n");

        // Add nodes
        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            let node_id = self.sanitize_mermaid_id(&node.url);
            let label = node.title.as_ref().unwrap_or(&node.url);
            
            mermaid.push_str(&format!(
                "    {}[\"{}\"]\n",
                node_id,
                label.replace("\"", "&quot;")
            ));
        }

        // Add edges
        for edge_idx in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge_idx).unwrap();
            let source_id = self.sanitize_mermaid_id(&self.graph[source].url);
            let target_id = self.sanitize_mermaid_id(&self.graph[target].url);
            
            mermaid.push_str(&format!("    {} --> {}\n", source_id, target_id));
        }

        Ok(mermaid)
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        let nodes: Vec<serde_json::Value> = self.graph
            .node_indices()
            .map(|idx| {
                let node = &self.graph[idx];
                serde_json::json!({
                    "id": node.url,
                    "label": node.title.as_ref().unwrap_or(&node.url),
                    "url": node.url,
                    "status_code": node.status_code,
                    "content_type": node.content_type,
                    "depth": node.depth,
                    "discovered_at": node.discovered_at
                })
            })
            .collect();

        let edges: Vec<serde_json::Value> = self.graph
            .edge_indices()
            .map(|edge_idx| {
                let (source, target) = self.graph.edge_endpoints(edge_idx).unwrap();
                let edge = &self.graph[edge_idx];
                serde_json::json!({
                    "source": self.graph[source].url,
                    "target": self.graph[target].url,
                    "link_type": edge.link_type,
                    "anchor_text": edge.anchor_text
                })
            })
            .collect();

        Ok(serde_json::json!({
            "nodes": nodes,
            "edges": edges
        }))
    }

    fn to_csv(&self) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("source_url,target_url,link_type,anchor_text\n");

        for edge_idx in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge_idx).unwrap();
            let edge = &self.graph[edge_idx];
            
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\"\n",
                self.graph[source].url,
                self.graph[target].url,
                edge.link_type,
                edge.anchor_text.as_ref().unwrap_or(&String::new())
            ));
        }

        Ok(csv)
    }
}

impl UrlGraph {
    fn get_node_color(&self, node: &crate::visualization::UrlNode) -> String {
        match node.status_code {
            Some(200..=299) => "#90EE90".to_string(), // Light green
            Some(300..=399) => "#87CEEB".to_string(), // Sky blue
            Some(400..=499) => "#FFD700".to_string(), // Gold
            Some(500..=599) => "#FF6B6B".to_string(), // Light red
            _ => "#D3D3D3".to_string(), // Light gray
        }
    }

    fn get_edge_color(&self, link_type: &crate::visualization::LinkType) -> String {
        match link_type {
            crate::visualization::LinkType::Direct => "#333333".to_string(),
            crate::visualization::LinkType::JavaScript => "#FF6B6B".to_string(),
            crate::visualization::LinkType::Form => "#4ECDC4".to_string(),
            crate::visualization::LinkType::Sitemap => "#45B7D1".to_string(),
            crate::visualization::LinkType::Robots => "#96CEB4".to_string(),
            crate::visualization::LinkType::External => "#FFEAA7".to_string(),
        }
    }

    fn sanitize_mermaid_id(&self, url: &str) -> String {
        url.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .take(50)
            .collect()
    }
}

impl GraphExport for SecurityGraph {
    fn to_dot(&self) -> Result<String> {
        let mut dot = String::new();
        dot.push_str("digraph SecurityFindings {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [shape=ellipse, style=filled];\n");

        for (i, finding) in self.findings.iter().enumerate() {
            let color = match finding.severity.as_str() {
                "critical" => "#FF6B6B",
                "high" => "#FFA500",
                "medium" => "#FFD700",
                "low" => "#90EE90",
                _ => "#D3D3D3",
            };

            dot.push_str(&format!(
                "    finding{} [label=\"{}\\n{}\", fillcolor=\"{}\"];\n",
                i,
                finding.finding_type.replace("\"", "\\\""),
                finding.severity,
                color
            ));
        }

        dot.push_str("}\n");
        Ok(dot)
    }

    fn to_mermaid(&self) -> Result<String> {
        let mut mermaid = String::new();
        mermaid.push_str("graph TD\n");

        for (i, finding) in self.findings.iter().enumerate() {
            mermaid.push_str(&format!(
                "    finding{}[\"{}: {}\"]\n",
                i,
                finding.finding_type,
                finding.severity
            ));
        }

        Ok(mermaid)
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        serde_json::to_value(&self.findings).map_err(Into::into)
    }

    fn to_csv(&self) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("finding_type,severity,description,url,evidence,recommendation\n");

        for finding in &self.findings {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                finding.finding_type,
                finding.severity,
                finding.description.replace("\"", "\"\""),
                finding.url,
                finding.evidence.replace("\"", "\"\""),
                String::new().replace("\"", "\"\"")
            ));
        }

        Ok(csv)
    }
}
