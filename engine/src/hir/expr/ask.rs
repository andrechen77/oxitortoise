//! The `ask` command and `of` reporter.

use std::fmt;

use pretty_print::PrettyPrinter;
use reflection::{Reflect, mir};

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, build_mir::translate_expr,
        expr::Agentset, ty::NlAbstractTyAtom,
    },
    sim::{
        patch::{OptionPatchId, PatchId},
        turtle::TurtleId,
    },
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
        NlAbstractTyAtom::Unit.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
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

impl Ask {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = translate_expr(builder, &self.workspace)?;
        let rng_local = translate_expr(builder, &self.rng)?;

        let operation = match self.recipients.as_ref() {
            ExprKind::Agentset(Agentset::AllTurtles) => {
                // statically known to be all turtles
                let ExprKind::Closure(closure) = self.body.as_ref() else {
                    panic!("expected ask body to be a closure literal, got: {:?}", self.body);
                };
                let body_local =
                    closure.write_mir_execution_with_static_types::<TurtleId, ()>(builder);
                mir::Operation::CallHostFunction {
                    function: &ask_all_turtles::FN_INFO,
                    args: vec![
                        mir::PlaceOperand::Direct(workspace_local.place()),
                        mir::PlaceOperand::Direct(rng_local.place()),
                        mir::PlaceOperand::Direct(body_local.place()),
                    ],
                }
            }
            ExprKind::Agentset(Agentset::AllPatches) => {
                // statically known to be all patches
                let ExprKind::Closure(closure) = self.body.as_ref() else {
                    panic!("expected ask body to be a closure literal, got: {:?}", self.body);
                };
                let body_local =
                    closure.write_mir_execution_with_static_types::<PatchId, ()>(builder);
                mir::Operation::CallHostFunction {
                    function: &ask_all_patches::FN_INFO,
                    args: vec![
                        mir::PlaceOperand::Direct(workspace_local.place()),
                        mir::PlaceOperand::Direct(rng_local.place()),
                        mir::PlaceOperand::Direct(body_local.place()),
                    ],
                }
            }
            other => {
                let _recipients_local = translate_expr(builder, &self.recipients)?;
                let _body_local = translate_expr(builder, &self.body)?;
                todo!("TODO(mvp) write MIR execution for Ask with dynamic agentset: {:?}", other);
            }
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod ask_all_turtles {
    use reflection::mir::HostFunctionInfo;

    use crate::{
        exec::jit::JitCallback,
        sim::{turtle::TurtleId, value::agentset::shuffled_turtles},
        util::rng::CanonRng,
        workspace::Workspace,
    };

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "ask_all_turtles",
        parameter_types: &[
            <&mut Workspace>::STATIC_TYPE,
            <&mut CanonRng>::STATIC_TYPE,
            // The lifetime is not actually 'static, but rather the
            // existentially quantified lifetime that would have been inferred
            // if it was part of a real Rust signature
            <JitCallback<'static, TurtleId, ()>>::STATIC_TYPE,
        ],
        return_type: <()>::STATIC_TYPE,
        link_name: "ask_all_turtles",
        link_addr: call as *const u8,
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
        sim::{patch::PatchId, value::agentset::shuffled_patches},
        util::rng::CanonRng,
        workspace::Workspace,
    };
    use reflection::{Reflect as _, mir::HostFunctionInfo};

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "ask_all_patches",
        parameter_types: &[
            <&mut Workspace>::STATIC_TYPE,
            <&mut CanonRng>::STATIC_TYPE,
            // The lifetime is not actually 'static, but rather the
            // existentially quantified lifetime that would have been inferred
            // if it was part of a real Rust signature
            <JitCallback<'static, PatchId, ()>>::STATIC_TYPE,
        ],
        return_type: <()>::STATIC_TYPE,
        link_name: "ask_all_patches",
        link_addr: call as *const u8,
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
        let NlAbstractTy::Union(un) = body_ty else {
            panic!("Of body must have closure type, got: {:?}", body_ty);
        };
        let closure = un.get_closure().unwrap();
        closure.return_ty.as_ref().clone()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
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

impl Of {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, rng, recipients, body } = self;

        let workspace_local = translate_expr(builder, workspace)?;
        let rng_local = translate_expr(builder, rng)?;

        match recipients.as_ref() {
            ExprKind::Agentset(Agentset::AllTurtles) => todo!(),
            ExprKind::Agentset(Agentset::AllPatches) => todo!(),
            other => {
                let recipients_local = translate_expr(builder, other)?;
                let recipients_ty = builder.mir.type_of_place(&recipients_local.place());
                if recipients_ty.is::<PatchId>() || recipients_ty.is::<OptionPatchId>() {
                    // FIXME option shouldn't be here, this is just to make it compile for now
                    let ExprKind::Closure(closure) = body.as_ref() else {
                        panic!("expected of body to be a closure literal, got: {:?}", body);
                    };

                    closure.write_mir_inline_call(
                        builder,
                        &[workspace_local.place(), rng_local.place(), recipients_local.place()],
                    )
                } else {
                    todo!("TODO(mvp) handle other recipient types, namely {:?}", recipients_ty);
                }
            }
        }
    }
}
