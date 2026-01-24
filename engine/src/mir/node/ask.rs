//! The `ask` command and `of` reporter.

use derive_more::derive::Display;
use lir::smallvec::{SmallVec, smallvec};

use crate::{
    exec::jit::host_fn,
    mir::{
        ClosureType, FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform,
        Program, WriteLirError, build_lir::LirInsnBuilder, node,
    },
};

/// A node representing an "ask" construct.
#[derive(Debug, Display)]
#[display("Ask {recipients:?}")]
pub struct Ask {
    /// The execution context to use for the ask.
    pub context: NodeId,
    /// The agents being asked.
    pub recipients: AskRecipient,
    /// A closure representing the commands to run for each recipient.
    pub body: NodeId,
}

impl Node for Ask {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        let mut deps = vec![self.context, self.body];
        if let Some(recipients) = self.recipients.node() {
            deps.push(recipients);
        }
        deps
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        // TODO(wishlist) a more robust solution would check the type of the
        // recipients to see if it refers to an entire agent class, rather than
        // just checking for a specific node. this would require augmenting the
        // type system to include special types for "entire agent class"

        fn type_erase_ask(program: &mut Program, _fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let &NodeKind::Ask(Ask { context: _, recipients, body: _ }) =
                &program.nodes[my_node_id]
            else {
                return false;
            };

            let AskRecipient::Any(recipients) = recipients else {
                return false;
            };

            if let NodeKind::Agentset(agentset) = &program.nodes[recipients] {
                let new_recipients = match agentset {
                    node::Agentset::AllTurtles => AskRecipient::AllTurtles,
                    node::Agentset::AllPatches => AskRecipient::AllPatches,
                };

                let NodeKind::Ask(ask) = &mut program.nodes[my_node_id] else {
                    panic!("expected node to be an Ask");
                };
                ask.recipients = new_recipients;
                return true;
            }
            false
        }

        Some(Box::new(type_erase_ask))
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[env_ptr, fn_ptr] = lir_builder.get_node_results(program, self.body) else {
            panic!("expected node outputting closure body to be two LIR values");
        };

        let lir_insn = match self.recipients {
            AskRecipient::AllTurtles => lir::generate_host_function_call(
                host_fn::ASK_ALL_TURTLES,
                Box::new([ctx_ptr, env_ptr, fn_ptr]),
            ),
            AskRecipient::AllPatches => lir::generate_host_function_call(
                host_fn::ASK_ALL_PATCHES,
                Box::new([ctx_ptr, env_ptr, fn_ptr]),
            ),
            _ => todo!("TODO(mvp) write LIR code to call a host function"),
        };

        let pc = lir_builder.push_lir_insn(lir_insn);
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("Of")]
pub struct Of {
    /// The execution context to use for the ask.
    pub context: NodeId,
    /// The recipients to ask.
    pub recipients: AskRecipient,
    /// A closure representing the reporter to run for each recipient.
    pub body: NodeId,
}

impl Node for Of {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        let mut deps = Vec::new();
        deps.push(self.context);
        if let Some(recipients) = self.recipients.node() {
            deps.push(recipients);
        }
        deps.push(self.body);
        deps
    }

    fn output_type(&self, program: &Program, fn_id: FunctionId) -> MirTy {
        let NlAbstractTy::Closure(closure) = program.nodes[self.body]
            .output_type(program, fn_id)
            .abstr
            .expect("closure must have an abstract type")
        else {
            panic!("expected node outputting closure body to be a closure")
        };

        (*closure.return_ty).into()
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn narrow_recipient_type(
            program: &mut Program,
            fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let &NodeKind::Of(Of { context: _, recipients, body: _ }) = &program.nodes[my_node_id]
            else {
                return false;
            };

            let Some(recipients_node) = recipients.node() else {
                return false;
            };
            let recipients_type = program.nodes[recipients_node].output_type(program, fn_id);
            match recipients_type.abstr.expect("recipients must have an abstract type") {
                NlAbstractTy::Turtle => {
                    let NodeKind::Of(Of { context: _, recipients, body: _ }) =
                        &mut program.nodes[my_node_id]
                    else {
                        panic!("expected node to be an Of");
                    };
                    *recipients = AskRecipient::SingleTurtle(recipients_node);
                }
                NlAbstractTy::Patch => {
                    let NodeKind::Of(Of { context: _, recipients, body: _ }) =
                        &mut program.nodes[my_node_id]
                    else {
                        panic!("expected node to be an Of");
                    };
                    *recipients = AskRecipient::SinglePatch(recipients_node);
                }
                _ => todo!("TODO(mvp) narrow for other recipient types as well"),
            }
            true
        }
        Some(Box::new(narrow_recipient_type))
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[env_ptr, fn_ptr] = lir_builder.get_node_results(program, self.body) else {
            panic!("expected node outputting closure body to be two LIR values");
        };
        let &[ctx_ptr] = lir_builder.get_node_results(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value");
        };

        // find the output type of the closure
        let NlAbstractTy::Closure(ClosureType { arg_ty: _, return_ty }) = program.nodes[self.body]
            .output_type(program, lir_builder.fn_id)
            .abstr
            .expect("closure must have an abstract type")
        else {
            panic!("expected node outputting a closure");
        };
        let closure_outputs: SmallVec<[lir::ValType; 1]> = return_ty
            .repr()
            .info()
            .mem_repr
            .expect("closure return type must have known ABI")
            .iter()
            .map(|&(_, r#type)| r#type.loaded_type())
            .collect();
        let closure_outputs_len = closure_outputs.len();

        match self.recipients {
            AskRecipient::SingleTurtle(recipient) | AskRecipient::SinglePatch(recipient) => {
                let &[recipient] = lir_builder.get_node_results(program, recipient) else {
                    panic!("expected node outputting recipients to be a single LIR value");
                };

                let pc = lir_builder.push_lir_insn(lir::InsnKind::CallIndirectFunction {
                    function: fn_ptr,
                    output_type: closure_outputs,
                    args: Box::from([env_ptr, ctx_ptr, recipient]),
                });
                let output_vals =
                    (0..closure_outputs_len).map(|i| lir::ValRef(pc, i as u8)).collect();
                lir_builder.node_to_lir.insert(my_node_id, output_vals);
            }
            _ => todo!("TODO(mvp) write LIR code to indirectly call the closure or use a host fn"),
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AskRecipient {
    AllTurtles,
    AllPatches,
    TurtleAgentset(NodeId),
    PatchAgentset(NodeId),
    SingleTurtle(NodeId),
    SinglePatch(NodeId),
    Any(NodeId),
    // TODO(mvp) add links
}

impl AskRecipient {
    pub fn node(&self) -> Option<NodeId> {
        match self {
            AskRecipient::AllTurtles => None,
            AskRecipient::AllPatches => None,
            AskRecipient::TurtleAgentset(id) => Some(*id),
            AskRecipient::PatchAgentset(id) => Some(*id),
            AskRecipient::SingleTurtle(id) => Some(*id),
            AskRecipient::SinglePatch(id) => Some(*id),
            AskRecipient::Any(id) => Some(*id),
        }
    }
}
