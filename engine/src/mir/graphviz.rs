use slotmap::Key as _;

use crate::mir::{Function, NodeId};

impl Function {
    pub fn to_dot_string(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph {\n");

        // Add all nodes as vertices
        for (node_id, node) in &self.nodes {
            let node_label = format!("{}", node);
            // Escape quotes and other special characters for DOT format
            let escaped_label =
                node_label.replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");

            dot.push_str(&format!(
                "  {} [label=\"{}\"];\n",
                node_id.data().as_ffi(),
                escaped_label
            ));
        }

        // Add edges based on dependencies
        for (node_id, node) in &self.nodes {
            let dependencies = node.dependencies();
            for dep_id in dependencies {
                // Only add edge if the dependency node exists in the function
                if self.nodes.contains_key(dep_id) {
                    dot.push_str(&format!(
                        "  {} -> {};\n",
                        node_id.data().as_ffi(),
                        dep_id.data().as_ffi(),
                    ));
                }
            }
        }

        dot.push_str("}\n");
        dot
    }
}
