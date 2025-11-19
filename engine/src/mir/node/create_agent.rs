//! The `create-turtles` command and friends.

use derive_more::derive::Display;
use lir::{ValRef, smallvec::smallvec};
use slotmap::Key as _;

use crate::{
    exec::jit::host_fn,
    mir::{
        Node, Function, MirTy, NlAbstractTy, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::turtle::BreedId,
};

#[derive(Debug, Display)]
#[display("CreateTurtles {breed:?}")]
pub struct CreateTurtles {
    /// The execution context to use.
    pub context: NodeId,
    /// The breed of turtles to create.
    pub breed: BreedId,
    /// The number of turtles to create.
    pub num_turtles: NodeId,
    /// A closure representing the commands to run for each created turtle.
    pub body: NodeId,
}

impl Node for CreateTurtles {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.num_turtles, self.body]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, function, nodes, self.context)
        else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let breed_id = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
            ty: lir::ValType::I64,
            value: self.breed.data().as_ffi(),
        }));
        let &[num_turtles] =
            lir_builder.get_node_results(program, function, nodes, self.num_turtles)
        else {
            panic!("expected node outputting number of turtles to be a single LIR value")
        };
        let &[env_ptr, fn_ptr] = lir_builder.get_node_results(program, function, nodes, self.body)
        else {
            panic!("expected node outputting closure body to be two LIR values");
        };
        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::CREATE_TURTLES,
            Box::new([ctx_ptr, ValRef(breed_id, 0), num_turtles, env_ptr, fn_ptr]),
        ));
        lir_builder.node_to_lir.insert(my_node_id, smallvec![ValRef(pc, 0)]);
        Ok(())
    }
}
