//! The `of` reporter.

use derive_more::derive::Display;

use crate::mir::{Node, Function, MirTy, NlAbstractTy, NodeId, Nodes, Program};

#[derive(Debug, Display)]
#[display("Of")]
pub struct Of {
    /// The execution context to use for the ask.
    pub context: NodeId,
    /// The recipients to ask.
    pub recipients: NodeId,
    /// A closure representing the reporter to run for each recipient.
    pub body: NodeId,
}

impl Node for Of {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.recipients, self.body]
    }

    fn output_type(&self, program: &Program, function: &Function, nodes: &Nodes) -> MirTy {
        let MirTy::Abstract(NlAbstractTy::Closure(closure)) =
            nodes[self.body].output_type(program, function, nodes)
        else {
            panic!("expected node outputting closure body to be a closure")
        };

        MirTy::Abstract(*closure.return_ty)
    }
}
