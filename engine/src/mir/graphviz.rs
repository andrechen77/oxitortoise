use std::collections::HashSet;

use slotmap::Key as _;

use crate::mir::{
    FunctionId, MirVisitor, Node as _, NodeId, Program, StatementBlock, StatementKind,
    visit_mir_function,
};

pub fn to_dot_string(program: &Program, fn_id: FunctionId) -> String {
    to_dot_string_with_options(program, fn_id, false)
}

pub fn to_dot_string_with_options(
    program: &Program,
    fn_id: FunctionId,
    show_node_ids: bool,
) -> String {
    let function = &program.functions[fn_id];

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

    let mut statement_counter = 0;
    // add start node that points to first statement
    dot.push_str("  start [label=\"Start\", shape=box];\n");
    add_statement_block(&mut dot, &function.cfg, &mut statement_counter);
    if statement_counter > 0 {
        dot.push_str("  start -> stmt1 [color=blue];\n");
    }
    fn add_statement_block(
        dot: &mut String,
        block: &StatementBlock,
        statement_counter: &mut usize,
    ) {
        let mut prev_stmt_id = None;

        for stmt in &block.statements {
            *statement_counter += 1;
            let stmt_id = *statement_counter;

            // Add the statement node
            match stmt {
                StatementKind::Node(node_id) => {
                    dot.push_str(&format!("  stmt{} [label=\"Eval\", shape=box, style=filled, fillcolor=lightblue];\n", stmt_id));
                    dot.push_str(&format!("  stmt{} -> {};\n", stmt_id, node_id.data().as_ffi()));
                }
                StatementKind::IfElse { condition, then_block, else_block } => {
                    dot.push_str(&format!(
                        "  stmt{} [label=\"IfElse\", shape=box, style=filled, fillcolor=lightblue];\n",
                        stmt_id
                    ));
                    dot.push_str(&format!("  stmt{} -> {};\n", stmt_id, condition.data().as_ffi()));

                    // Add then block
                    let then_start = *statement_counter + 1;
                    add_statement_block(dot, then_block, statement_counter);
                    dot.push_str(&format!(
                        "  stmt{} -> stmt{} [label=\"then\"];\n",
                        stmt_id, then_start
                    ));

                    // Add else block
                    let else_start = *statement_counter + 1;
                    add_statement_block(dot, else_block, statement_counter);
                    dot.push_str(&format!(
                        "  stmt{} -> stmt{} [label=\"else\"];\n",
                        stmt_id, else_start
                    ));
                }
                StatementKind::Repeat { num_repetitions, block } => {
                    dot.push_str(&format!(
                        "  stmt{} [label=\"Repeat\", shape=box, style=filled, fillcolor=lightblue];\n",
                        stmt_id
                    ));
                    dot.push_str(&format!(
                        "  stmt{} -> {};\n",
                        stmt_id,
                        num_repetitions.data().as_ffi()
                    ));

                    let block_start = *statement_counter + 1;
                    add_statement_block(dot, block, statement_counter);
                    dot.push_str(&format!(
                        "  stmt{} -> stmt{} [label=\"body\"];\n",
                        stmt_id, block_start
                    ));
                }
                StatementKind::Return { value } => {
                    dot.push_str(&format!(
                        "  stmt{} [label=\"Return\", shape=box, style=filled, fillcolor=lightblue];\n",
                        stmt_id
                    ));
                    dot.push_str(&format!("  stmt{} -> {};\n", stmt_id, value.data().as_ffi()));
                }
                StatementKind::Stop => {
                    dot.push_str(&format!("  stmt{} [label=\"Stop\", shape=box, style=filled, fillcolor=lightblue];\n", stmt_id));
                }
            }

            // Connect to previous statement if it exists
            if let Some(prev_id) = prev_stmt_id {
                dot.push_str(&format!("  stmt{} -> stmt{} [color=blue];\n", prev_id, stmt_id));
            }
            prev_stmt_id = Some(stmt_id);
        }
    }

    dot.push_str("}\n");
    dot
}
