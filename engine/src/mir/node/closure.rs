//! Nodes to represent closures.

use std::fmt::{self, Write};

use lir::smallvec::smallvec;
use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        ClosureType, FunctionId, LocalId, MirTy, NlAbstractTy, Node, NodeId, Program,
        WriteLirError, build_lir::LirInsnBuilder,
    },
};

#[derive(Debug)]
pub struct Closure {
    /// All the local variables that are captured by the closure.
    pub captures: Vec<LocalId>,
    /// The body of the closure. This determines the calling convention of
    /// the function.
    pub body: FunctionId,
}

impl Node for Closure {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![]
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        let body_arg = program.functions[self.body].parameters[ClosureType::PARAM_ARG_IDX];
        let arg_ty = program.locals[body_arg]
            .ty
            .clone()
            .abstr
            .expect("closure argument must have an abstract type");
        let return_ty = program.functions[self.body]
            .return_ty
            .clone()
            .abstr
            .expect("closure return must have an abstract type");
        NlAbstractTy::Closure(ClosureType {
            arg_ty: Box::new(arg_ty),
            return_ty: Box::new(return_ty),
        })
        .into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        _program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // insert an instruction to create the env pointer
        let mk_env_ptr = if self.captures.is_empty() {
            lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Value::NULL))
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

    fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("Closure", |p| {
            p.add_field_with("body", |p| write!(p, "{:?}", self.body))?;
            if let Some(fn_name) = program.functions[self.body].debug_name.as_deref() {
                p.add_comment(fn_name)?;
            }
            Ok(())
        })
    }
}
