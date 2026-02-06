use slotmap::Key;

use crate::mir::{
    Function, FunctionId, MirVisitor, Node as _, NodeId, Program, visit_mir_function,
};
use std::{
    collections::HashSet,
    fmt::{self, Write},
};

/// Stores the overall pretty printed state.
struct PrettyPrinter {
    out: String,
    indent: usize,
}

impl PrettyPrinter {
    fn new() -> Self {
        Self { out: String::new(), indent: 0 }
    }

    /// Starts a new line with indentation.
    fn line(&mut self) -> fmt::Result {
        const INDENT: &str = "    ";
        write!(self.out, "\n{}", INDENT.repeat(self.indent))
    }

    fn add_struct(
        &mut self,
        name: &str,
        then: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "{} {{", name)?;
        self.indent += 1;
        then(self)?;
        self.indent -= 1;
        self.line()?;
        write!(self, "}}")
    }

    fn add_field(
        &mut self,
        name: &str,
        then: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        self.line()?;
        write!(self, "{}: ", name)?;
        then(self)?;
        write!(self, ",")
    }

    fn add_map<K: Copy, V>(
        &mut self,
        kv_pairs: impl Iterator<Item = (K, V)>,
        mut fmt_key: impl FnMut(&mut Self, K) -> fmt::Result,
        mut fmt_value: impl FnMut(&mut Self, (K, V)) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "{{")?;
        self.indent += 1;
        for (key, value) in kv_pairs {
            self.line()?;
            fmt_key(self, key)?;
            write!(self, ": ")?;
            fmt_value(self, (key, value))?;
            write!(self, ",")?;
        }
        self.indent -= 1;
        self.line()?;
        write!(self, "}}")
    }

    fn add_list<T>(
        &mut self,
        items: impl Iterator<Item = T>,
        mut fmt_item: impl FnMut(&mut Self, T) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "[")?;
        self.indent += 1;
        for item in items {
            self.line()?;
            fmt_item(self, item)?;
            write!(self, ",")?;
        }
        self.indent -= 1;
        self.line()?;
        write!(self, "]")
    }
}

impl<'a> Write for PrettyPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lines = s.split("\n");
        if let Some(first_line) = lines.next() {
            write!(self.out, "{}", first_line)?;
        }
        for line in lines {
            self.line()?;
            write!(self.out, "{}", line)?;
        }
        Ok(())
    }
}

impl Program {
    pub fn pretty_print(&self) -> String {
        let mut printer = PrettyPrinter::new();

        let Program {
            globals,
            globals_schema,
            turtle_breeds,
            custom_turtle_vars,
            turtle_schema,
            patch_schema,
            custom_patch_vars,
            functions,
            nodes,
            locals: program_locals,
        } = self;

        let _ = printer.add_struct("Program", |p| {
            p.add_field("globals", |p| {
                p.add_map(
                    globals.iter().enumerate(),
                    |p, index| write!(p, "{}", index),
                    |p, (_, global)| write!(p, "{:?}", global),
                )
            })?;
            p.add_field("globals_schema", |p| write!(p, "{:#?}", globals_schema))?;
            p.add_field("turtle_breeds", |p| {
                p.add_list(turtle_breeds.iter().map(|(breed_id, _)| breed_id), |p, breed_id| {
                    write!(p, "{:?}", breed_id)
                })
            })?;
            p.add_field("custom_turtle_vars", |p| {
                p.add_map(
                    custom_turtle_vars.iter().enumerate(),
                    |p, index| write!(p, "{}", index),
                    |p, (_, var)| write!(p, "{:?}", var),
                )
            })?;
            p.add_field("turtle_schema", |p| write!(p, "{:#?}", turtle_schema))?;
            p.add_field("custom_patch_vars", |p| {
                p.add_map(
                    custom_patch_vars.iter().enumerate(),
                    |p, index| write!(p, "{}", index),
                    |p, (_, var)| write!(p, "{:?}", var),
                )
            })?;
            p.add_field("patch_schema", |p| write!(p, "{:#?}", patch_schema))?;
            p.add_field("functions", |p| {
                p.add_map(
                    functions.iter(),
                    |p, fn_id| {
                        write!(p, "{:?}", fn_id)?;
                        if let Some(debug_name) = &functions[fn_id].debug_name {
                            write!(p, " ({:?})", debug_name)?;
                        }
                        Ok(())
                    },
                    |p, (fn_id, function)| {
                        let Function { debug_name, parameters, locals, return_ty, root_node } =
                            function;
                        p.add_struct("Function", |p| {
                            p.add_field("debug_name", |p| write!(p, "{:?}", debug_name))?;
                            p.add_field("parameters", |p| {
                                p.add_list(parameters.iter(), |p, param| write!(p, "{:?}", param))
                            })?;
                            p.add_field("return_ty", |p| write!(p, "{:?}", return_ty))?;
                            p.add_field("locals", |p| {
                                p.add_map(
                                    locals
                                        .iter()
                                        .map(|&local_id| (local_id, &program_locals[local_id])),
                                    |p, local_id| write!(p, "{:?}", local_id),
                                    |p, local_decl| write!(p, "{:?}", local_decl),
                                )
                            })?;
                            p.add_field("nodes", |p| {
                                p.add_struct("", |p| {
                                    p.line()?;
                                    write!(p, "digraph {{")?;
                                    p.indent += 1;

                                    // collect all nodes reachable from the start of the function
                                    struct NodeCollectorVisitor {
                                        nodes: HashSet<NodeId>,
                                    }
                                    impl MirVisitor for NodeCollectorVisitor {
                                        fn visit_node(
                                            &mut self,
                                            _program: &Program,
                                            _fn_id: FunctionId,
                                            node_id: NodeId,
                                        ) {
                                            self.nodes.insert(node_id);
                                        }
                                    }
                                    let mut visitor =
                                        NodeCollectorVisitor { nodes: HashSet::new() };
                                    visit_mir_function(&mut visitor, self, fn_id);

                                    // add all nodes as vertices
                                    for node_id in &visitor.nodes {
                                        let node = &nodes[*node_id];
                                        let node_label = format!("{}", node).replacen(' ', "\n", 1);
                                        let formatted_label =
                                            format!("{:?}\n{}", node_id.data(), node_label);
                                        let escaped_label = formatted_label
                                            .replace('"', "\\\"")
                                            .replace('\n', "\\n")
                                            .replace('\r', "\\r");

                                        p.line()?;
                                        write!(
                                            p,
                                            "{:?} [label=\"{}\"];",
                                            node_id.data().as_ffi(),
                                            escaped_label
                                        )?;
                                    }

                                    // add edges based on dependencies
                                    for node_id in &visitor.nodes {
                                        let node = &nodes[*node_id];
                                        let dependencies = node.dependencies();
                                        for (i, dep_id) in dependencies.into_iter().enumerate() {
                                            p.line()?;
                                            write!(
                                                p,
                                                "{:?} -> {:?} [label=\"{}\"];",
                                                node_id.data().as_ffi(),
                                                dep_id.data().as_ffi(),
                                                i,
                                            )?;
                                        }
                                    }

                                    p.indent -= 1;
                                    p.line()?;
                                    write!(p, "}}")
                                })
                            })
                        })
                    },
                )
            })
        });

        printer.out
    }
}
