//! Nodes for primitives that operate on lists and agentsets.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::jit::host_fn,
    mir::{
        Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::value::{NlBox, NlList},
    util::reflection::Reflect as _,
};

#[derive(Debug, Display)]
#[display("OneOf")]
pub struct OneOf {
    pub context: NodeId,
    pub operand: NodeId,
}

impl Node for OneOf {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.operand]
    }

    fn output_type(&self, program: &Program, function: &Function, nodes: &Nodes) -> MirTy {
        let out_type = match nodes[self.operand].output_type(program, function, nodes) {
            MirTy::Abstract(NlAbstractTy::Agentset { agent_type }) => agent_type,
            MirTy::Abstract(NlAbstractTy::List { element_ty }) => element_ty,
            x => panic!("Impossible argument type for `one-of`: {:?}", x),
            // TODO this could also just be an unknown type;
        };

        MirTy::Abstract(*out_type)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx] = lir_builder.get_node_results(program, function, nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[operand] = lir_builder.get_node_results(program, function, nodes, self.operand)
        else {
            panic!("expected node outputting list to be a single LIR value")
        };

        let operand_type = nodes[self.operand].output_type(program, function, nodes).repr();

        if operand_type == <NlBox<NlList>>::CONCRETE_TY {
            let insn =
                lir::generate_host_function_call(host_fn::ONE_OF_LIST, Box::from([ctx, operand]));
            let pc = lir_builder.push_lir_insn(insn);
            lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);

            Ok(())
        } else {
            todo!("TODO(mvp) handle other operand types")
        }
    }
}
