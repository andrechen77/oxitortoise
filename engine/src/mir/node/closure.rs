use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::mir::{
    ClosureType, EffectfulNode, FunctionId, LocalId, MirTy, NlAbstractTy, NodeId, Nodes,
    WriteLirError, build_lir::LirInsnBuilder,
};

#[derive(Debug, Display)]
#[display("Closure {captures:?} {body:?}")]
pub struct Closure {
    /// All the local variables that are captured by the closure.
    pub captures: Vec<LocalId>,
    /// The body of the closure. This determines the calling convention of
    /// the function.
    pub body: FunctionId,
}

impl EffectfulNode for Closure {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(
        &self,
        program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> crate::mir::MirTy {
        let function = program.functions[self.body].borrow();
        let arg_ty = function.locals[function.parameters[ClosureType::PARAM_ARG_IDX]]
            .ty
            .clone()
            .as_abstract();
        let return_ty = function.return_ty.clone();
        MirTy::Abstract(NlAbstractTy::Closure(ClosureType {
            arg_ty: Box::new(arg_ty),
            return_ty: Box::new(return_ty.as_abstract()),
        }))
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        _nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // insert an instruction to create the env pointer
        let mk_env_ptr = if self.captures.is_empty() {
            lir_builder
                .push_lir_insn(lir::InsnKind::Const(lir::Const { ty: lir::ValType::Ptr, value: 0 }))
        } else {
            // TODO(mvp) verify that the captured variables are on the stack and
            // create a pointer to the stack frame
            todo!()
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
