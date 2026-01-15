use std::collections::HashSet;

use slotmap::Key as _;

use crate::mir::{FunctionId, MirVisitor, Node as _, NodeId, Program, visit_mir_function};

pub fn to_dot_string(program: &Program, fn_id: FunctionId) -> String {
    to_dot_string_with_options(program, fn_id, false)
}

pub fn to_dot_string_with_options(
    program: &Program,
    fn_id: FunctionId,
    show_node_ids: bool,
) -> String {
    // collect all nodes reachable from the start of the function
    struct NodeCollectorVisitor {
        nodes: HashSet<NodeId>,
    }
    impl MirVisitor for NodeCollectorVisitor {
        fn visit_node(&mut self, _program: &Program, _fn_id: FunctionId, node_id: NodeId) {
            self.nodes.insert(node_id);
        }
    }
    let mut visitor = NodeCollectorVisitor { nodes: HashSet::new() };
    visit_mir_function(&mut visitor, program, fn_id);
    let NodeCollectorVisitor { nodes } = visitor;

    // build the dot string

    let mut dot = String::new();
    dot.push_str("digraph {\n");

    // add all nodes as vertices
    for node_id in &nodes {
        let node = &program.nodes[*node_id];
        let node_label = format!("{}", node).replacen(' ', "\n", 1);

        // Format the label with newline after node type and optionally include node ID
        let formatted_label =
            if show_node_ids { format!("{:?}\n{}", node_id, node_label) } else { node_label };

        // Escape quotes and other special characters for DOT format
        let escaped_label =
            formatted_label.replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");

        dot.push_str(&format!("  {} [label=\"{}\"];\n", node_id.data().as_ffi(), escaped_label));
    }

    // add edges based on dependencies
    for node_id in &nodes {
        let node = &program.nodes[*node_id];
        let dependencies = node.dependencies();
        for (i, dep_id) in dependencies.into_iter().enumerate() {
            dot.push_str(&format!(
                "  {} -> {} [label=\"{}\"];\n",
                node_id.data().as_ffi(),
                dep_id.data().as_ffi(),
                i,
            ));
        }
    }

    dot.push_str("}\n");
    dot
}
