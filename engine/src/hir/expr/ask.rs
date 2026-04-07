//! The `ask` command and `of` reporter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        ClosureType, Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, expr::Agentset,
    },
    mir,
    util::reflection::Reflect,
};

#[derive(Debug, Clone)]
pub struct Ask {
    pub workspace: Box<ExprKind>,
    pub rng: Box<ExprKind>,
    /// The agents being asked.
    pub recipients: Box<ExprKind>,
    /// The closure representing the commands to run for each recipient.
    pub body: Box<ExprKind>,
}

impl Expr for Ask {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.rng);
        visitor(&self.recipients);
        visitor(&self.body);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.rng.as_mut());
        visitor(self.recipients.as_mut());
        visitor(self.body.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = self.workspace.write_mir_execution(builder)?;
        let rng_local = self.rng.write_mir_execution(builder)?;

        let operation = match self.recipients.as_ref() {
            ExprKind::Agentset(Agentset::AllTurtles) => {
                // statically known to be all turtles
                let body_local = self.body.write_mir_execution(builder)?;
                mir::Operation::CallHostFunction {
                    function: &ask_all_turtles::FN_INFO,
                    args: vec![
                        mir::PlaceOperand::Copy(workspace_local.place()),
                        mir::PlaceOperand::Copy(rng_local.place()),
                        mir::PlaceOperand::Move(body_local),
                    ],
                }
            }
            ExprKind::Agentset(Agentset::AllPatches) => {
                // statically known to be all patches
                let body_local = self.body.write_mir_execution(builder)?;
                mir::Operation::CallHostFunction {
                    function: &ask_all_patches::FN_INFO,
                    args: vec![
                        mir::PlaceOperand::Copy(workspace_local.place()),
                        mir::PlaceOperand::Copy(rng_local.place()),
                        mir::PlaceOperand::Move(body_local),
                    ],
                }
            }
            other => {
                let recipients_local = self.recipients.write_mir_execution(builder)?;
                let body_local = self.body.write_mir_execution(builder)?;
                todo!("TODO(mvp) write MIR execution for Ask with dynamic agentset: {:?}", other);
            }
        };
        Some(builder.mir.add_operation(None, operation))
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Ask { workspace, rng, recipients, body } = self;
        p.add_fn_call("ask", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| recipients.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names))?;
            Ok(())
        })
    }
}

mod ask_all_turtles {
    use crate::{
        exec::jit::JitCallback,
        mir::HostFunctionInfo,
        sim::{turtle::TurtleId, value::agentset::shuffled_turtles},
        util::rng::CanonRng,
        workspace::Workspace,
    };

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "ask_all_turtles",
        parameter_types: &[
            <&mut Workspace>::TYPE,
            <&mut CanonRng>::TYPE,
            // The lifetime is not actually 'static, but rather the
            // existentially quantified lifetime that would have been inferred
            // if it was part of a real Rust signature
            <JitCallback<'static, TurtleId, ()>>::TYPE,
        ],
        return_type: <()>::TYPE,
    };

    pub fn call(
        workspace: &mut Workspace,
        rng: &mut CanonRng,
        mut callback: JitCallback<TurtleId, ()>,
    ) {
        let mut iter = shuffled_turtles(&workspace.world);
        while let Some(turtle) = iter.next(rng) {
            callback.call_mut(workspace, rng, turtle);
        }
    }
}

mod ask_all_patches {
    use crate::{
        exec::jit::JitCallback,
        mir::HostFunctionInfo,
        sim::{patch::PatchId, value::agentset::shuffled_patches},
        util::rng::CanonRng,
        workspace::Workspace,
    };

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "ask_all_patches",
        parameter_types: &[
            <&mut Workspace>::TYPE,
            <&mut CanonRng>::TYPE,
            // The lifetime is not actually 'static, but rather the
            // existentially quantified lifetime that would have been inferred
            // if it was part of a real Rust signature
            <JitCallback<'static, PatchId, ()>>::TYPE,
        ],
        return_type: <()>::TYPE,
    };

    pub fn call(
        workspace: &mut Workspace,
        rng: &mut CanonRng,
        mut callback: JitCallback<PatchId, ()>,
    ) {
        let mut iter = shuffled_patches(&workspace.world);
        while let Some(patch) = iter.next(rng) {
            callback.call_mut(workspace, rng, patch);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Of {
    pub workspace: Box<ExprKind>,
    pub rng: Box<ExprKind>,
    /// The recipients to ask.
    pub recipients: Box<ExprKind>,
    /// The closure representing the reporter to run for each recipient.
    pub body: Box<ExprKind>,
}

impl Expr for Of {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let body_ty = self.body.output_type(names);
        let NlAbstractTy::Closure(ClosureType { return_ty, .. }) = body_ty else {
            panic!("Of body must have closure type, got: {:?}", body_ty);
        };
        *return_ty
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.rng);
        visitor(&self.recipients);
        visitor(&self.body);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.rng.as_mut());
        visitor(self.recipients.as_mut());
        visitor(self.body.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for Of")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Of { workspace, rng, recipients, body } = self;
        p.add_fn_call("of", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| recipients.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names))?;
            Ok(())
        })
    }
}
