use derive_more::derive::Display;
use lir::smallvec::smallvec;
use slotmap::SlotMap;

use crate::mir::{
    EffectfulNode, FunctionId, LocalId, MirType, NetlogoAbstractType, NodeId, WriteLirError,
    build_lir::LirInsnBuilder,
};

#[derive(Debug, Display)]
#[display("Closure {captures:?} {body:?}")]
pub struct Closure {
    /// All the local variables that are captured by the closure.
    pub captures: Vec<LocalId>,
    /// The body of the closure.
    pub body: FunctionId,
    // TODO other stuff like whether the closure uses a context pointer, env
    // pointer, self/myself, etc.

    // Backward edge for the projection node for the environment pointer and
    // function pointer from this closure.
    // pub projections: Cell<Option<(NodeId, NodeId)>>,
}

// impl Closure {
//     fn generate_projections(
//         &self,
//         my_node_id: NodeId,
//         nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
//     ) -> (NodeId, NodeId) {
//         if let Some(projs) = self.projections.get() {
//             projs
//         } else {
//             let proj_env_ptr = nodes.insert(Box::new(ProjClosureEnv { closure: my_node_id }));
//             let proj_fn_ptr = nodes.insert(Box::new(ProjClosureFnPtr { closure: my_node_id }));
//             let projs = (proj_env_ptr, proj_fn_ptr);
//             self.projections.set(Some(projs));
//             projs
//         }
//     }
// }

impl EffectfulNode for Closure {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(
        &self,
        program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> crate::mir::MirType {
        let return_ty = program.functions[self.body].borrow().return_ty.clone();
        MirType::Abstract(NetlogoAbstractType::Closure { return_ty: Box::new(return_ty) })
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // insert an instruction to create the env pointer
        let mk_env_ptr = if !self.captures.is_empty() {
            todo!(
                "verify that the captured variables are on the stack and create a pointer to the stack"
            )
        } else {
            lir_builder
                .push_lir_insn(lir::InsnKind::Const(lir::Const { ty: lir::ValType::Ptr, value: 0 }))
        };

        // insert an instruction to create the function pointer
        let mk_fn_ptr = lir_builder.push_lir_insn(lir::InsnKind::UserFunctionPtr {
            function: lir_builder.program_builder.available_user_functions[&self.body],
        });

        // let (proj_env_ptr, proj_fn_ptr) = self.projections.get().unwrap();
        // lir_builder.node_to_lir.insert(proj_env_ptr, ValRef(mk_env_ptr, 0));
        // lir_builder.node_to_lir.insert(proj_fn_ptr, ValRef(mk_fn_ptr, 0));
        lir_builder
            .node_to_lir
            .insert(my_node_id, smallvec![lir::ValRef(mk_env_ptr, 0), lir::ValRef(mk_fn_ptr, 0),]);

        Ok(())
    }
}

// #[derive(Debug)]
// pub struct ProjClosureEnv {
//     pub closure: NodeId,
// }

// impl EffectfulNode for ProjClosureEnv {
//     fn has_side_effects(&self) -> bool {
//         false
//     }

//     fn dependencies(&self) -> Vec<NodeId> {
//         vec![self.closure]
//     }

//     fn output_type(
//         &self,
//         workspace: &crate::workspace::Workspace,
//         nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
//         locals: &SlotMap<LocalId, LocalDeclaration>,
//     ) -> Option<NetlogoInternalType> {
//         Some(NetlogoInternalType::UNTYPED_PTR)
//     }

//     fn write_lir_insns(
//         &self,
//         my_node_id: NodeId,
//         lir_builder: &mut LirInsnBuilder,
//     ) -> Result<(), ()> {
//         // do nothing. if a projection's dependencies have been executed, then
//         // the projection is automatically available.
//         assert!(lir_builder.node_to_lir.contains_key(&my_node_id));
//         Ok(())
//     }
// }

// #[derive(Debug)]
// pub struct ProjClosureFnPtr {
//     pub closure: NodeId,
// }

// impl EffectfulNode for ProjClosureFnPtr {
//     fn has_side_effects(&self) -> bool {
//         false
//     }

//     fn dependencies(&self) -> Vec<NodeId> {
//         vec![self.closure]
//     }

//     fn output_type(
//         &self,
//         workspace: &crate::workspace::Workspace,
//         nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
//         locals: &SlotMap<LocalId, LocalDeclaration>,
//     ) -> Option<NetlogoInternalType> {
//         Some(NetlogoInternalType::UNTYPED_PTR)
//     }

//     fn write_lir_insns(
//         &self,
//         my_node_id: NodeId,
//         lir_builder: &mut LirInsnBuilder,
//     ) -> Result<(), ()> {
//         // do nothing. if a projection's dependencies have been executed, then
//         // the projection is automatically available.
//         assert!(lir_builder.node_to_lir.contains_key(&my_node_id));
//         Ok(())
//     }
// }
