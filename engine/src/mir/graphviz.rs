use slotmap::Key as _;

use crate::mir::{Node as _, Function, StatementBlock};

impl Function {
    pub fn to_dot_string(&self) -> String {
        self.to_dot_string_with_options(false)
    }

    pub fn to_dot_string_with_options(&self, show_node_ids: bool) -> String {
        let mut dot = String::new();
        dot.push_str("digraph {\n");

        // Add all nodes as vertices
        for (node_id, node) in &*self.nodes.borrow() {
            let node_label = format!("{}", node).replacen(' ', "\n", 1);

            // Format the label with newline after node type and optionally include node ID
            let formatted_label = if show_node_ids {
                format!("{}\nID: {:?}", node_label, node_id)
            } else {
                node_label
            };

            // Escape quotes and other special characters for DOT format
            let escaped_label =
                formatted_label.replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");

            dot.push_str(&format!(
                "  {} [label=\"{}\"];\n",
                node_id.data().as_ffi(),
                escaped_label
            ));
        }

        // Add edges based on dependencies
        for (node_id, node) in &*self.nodes.borrow() {
            let dependencies = node.dependencies();
            for (i, dep_id) in dependencies.into_iter().enumerate() {
                // Only add edge if the dependency node exists in the function
                if self.nodes.borrow().contains_key(dep_id) {
                    dot.push_str(&format!(
                        "  {} -> {} [label=\"{}\"];\n",
                        node_id.data().as_ffi(),
                        dep_id.data().as_ffi(),
                        i,
                    ));
                }
            }
        }

        // Add statement blocks recursively
        let mut statement_counter = 0;
        // Add start node that points to first statement
        dot.push_str("  start [label=\"Start\", shape=box];\n");
        add_statement_block(&mut dot, &self.cfg, &mut statement_counter);
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
                    crate::mir::StatementKind::Node(node_id) => {
                        dot.push_str(&format!("  stmt{} [label=\"Eval\", shape=box, style=filled, fillcolor=lightblue];\n", stmt_id));
                        dot.push_str(&format!(
                            "  stmt{} -> {};\n",
                            stmt_id,
                            node_id.data().as_ffi()
                        ));
                    }
                    crate::mir::StatementKind::IfElse { condition, then_block, else_block } => {
                        dot.push_str(&format!(
                            "  stmt{} [label=\"IfElse\", shape=box, style=filled, fillcolor=lightblue];\n",
                            stmt_id
                        ));
                        dot.push_str(&format!(
                            "  stmt{} -> {};\n",
                            stmt_id,
                            condition.data().as_ffi()
                        ));

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
                    crate::mir::StatementKind::Repeat { num_repetitions, block } => {
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
                    crate::mir::StatementKind::Return { value } => {
                        dot.push_str(&format!(
                            "  stmt{} [label=\"Return\", shape=box, style=filled, fillcolor=lightblue];\n",
                            stmt_id
                        ));
                        dot.push_str(&format!("  stmt{} -> {};\n", stmt_id, value.data().as_ffi()));
                    }
                    crate::mir::StatementKind::Stop => {
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
}
