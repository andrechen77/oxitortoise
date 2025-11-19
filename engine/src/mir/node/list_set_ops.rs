//! Nodes for primitives that operate on lists and agentsets.

use derive_more::derive::Display;

use crate::mir::{Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program};

#[derive(Debug, Display)]
#[display("OneOf")]
pub struct OneOf {
    pub context: NodeId,
    pub xs: NodeId,
}

impl Node for OneOf {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.xs]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        let out_type = match _nodes[self.xs].output_type(_program, _function, _nodes) {
            MirTy::Abstract(NlAbstractTy::Agentset { agent_type }) => agent_type,
            MirTy::Abstract(NlAbstractTy::List { element_ty }) => element_ty,
            x => panic!("Impossible argument type for `one-of`: {:?}", x),
        };

        MirTy::Abstract(*out_type)
    }
}
