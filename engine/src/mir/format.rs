use pretty_print::PrettyPrinter;
use slotmap::Key;

use crate::mir::{
    Function, FunctionId, MirVisitor, Node as _, NodeId, Program, TurtleBreeds, visit_mir_function,
};
use std::{collections::HashSet, fmt::Write};

impl Program {
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        let mut printer = PrettyPrinter::new(&mut out);

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
            p.add_field("turtle_breeds", |p| match turtle_breeds {
                TurtleBreeds::Full(breeds) => p
                    .add_list(breeds.iter().map(|(breed_id, _)| breed_id), |p, breed_id| {
                        write!(p, "{:?}", breed_id)
                    }),
                TurtleBreeds::Partial(breeds) => p
                    .add_list(breeds.iter().map(|(breed_id, _)| breed_id), |p, breed_id| {
                        write!(p, "{:?}", breed_id)
                    }),
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
                        let Function {
                            debug_name,
                            parameters,
                            locals,
                            return_ty,
                            root_node: _,
                            is_entrypoint,
                        } = function;
                        p.add_struct("Function", |p| {
                            p.add_field("debug_name", |p| write!(p, "{:?}", debug_name))?;
                            p.add_field("is_entrypoint", |p| write!(p, "{}", is_entrypoint))?;
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

                                    p.line()?;
                                    write!(p, "digraph {{")?;
                                    p.indented(|p| {
                                        // add all nodes as vertices
                                        for node_id in &visitor.nodes {
                                            let node = &nodes[*node_id];
                                            let mut label = format!("{:?}\n", node_id.data());
                                            node.pretty_print(self, &mut label)?;
                                            label = label
                                                .replace('"', "\\\"")
                                                .replace('\n', "\\n")
                                                .replace('\r', "\\r");

                                            p.line()?;
                                            write!(
                                                p,
                                                "{:?} [label=\"{}\"];",
                                                node_id.data().as_ffi(),
                                                label
                                            )?;
                                        }

                                        // add edges based on dependencies
                                        for node_id in &visitor.nodes {
                                            let node = &nodes[*node_id];
                                            let dependencies = node.dependencies();
                                            for (i, (dep_name, dep_id)) in
                                                dependencies.into_iter().enumerate()
                                            {
                                                p.line()?;
                                                write!(
                                                    p,
                                                    "{:?} -> {:?} [label=\"{}:{}\"];",
                                                    node_id.data().as_ffi(),
                                                    dep_id.data().as_ffi(),
                                                    i,
                                                    dep_name,
                                                )?;
                                            }
                                        }
                                        Ok(())
                                    })?;

                                    p.line()?;
                                    write!(p, "}}")
                                })
                            })
                        })
                    },
                )
            })
        });

        out
    }
}
