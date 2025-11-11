use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::mir::{
    EffectfulNode, EffectfulNodeKind, FunctionId, MirType, NetlogoAbstractType, NodeId,
    NodeTransform, Nodes, Program, WriteLirError, build_lir::LirInsnBuilder, node,
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

impl EffectfulNode for Ask {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        let mut deps = vec![self.context, self.body];
        if let Some(recipients) = self.recipients.node() {
            deps.push(recipients);
        }
        deps
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirType {
        MirType::Abstract(NetlogoAbstractType::Unit)
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

        fn type_erase_ask(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let function = program.functions[fn_id].borrow();
            let mut nodes = function.nodes.borrow_mut();

            let EffectfulNodeKind::Ask(ask) = &nodes[my_node_id] else {
                panic!("expected node to be an Ask");
            };
            let AskRecipient::Any(recipients) = ask.recipients else {
                return false;
            };

            if let EffectfulNodeKind::Agentset(agentset) = &nodes[recipients] {
                let new_recipients = match agentset {
                    node::Agentset::AllTurtles => AskRecipient::AllTurtles,
                    node::Agentset::AllPatches => AskRecipient::AllPatches,
                };

                let EffectfulNodeKind::Ask(ask) = &mut nodes[my_node_id] else {
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
        my_node_id: NodeId,
        nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[env_ptr, fn_ptr] = lir_builder.get_node_results(nodes, self.body) else {
            panic!("expected node outputting closure body to be two LIR values");
        };

        // TODO(mvp) consult registry of host functions to find signatures.
        // there should be a registry of host functions that we can consult
        // which abstracts the details of the signatures of each function.
        let lir_insn = match self.recipients {
            AskRecipient::AllTurtles => lir::InsnKind::CallHostFunction {
                function: lir_builder.program_builder.host_function_ids.ask_all_turtles,
                output_type: smallvec![],
                args: Box::new([ctx_ptr, env_ptr, fn_ptr]),
            },
            AskRecipient::AllPatches => lir::InsnKind::CallHostFunction {
                function: lir_builder.program_builder.host_function_ids.ask_all_patches,
                output_type: smallvec![],
                args: Box::new([ctx_ptr, env_ptr, fn_ptr]),
            },
            _ => todo!("TODO(mvp) write LIR code to call a host function"),
        };

        let pc = lir_builder.push_lir_insn(lir_insn);
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
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
    fn node(&self) -> Option<NodeId> {
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
