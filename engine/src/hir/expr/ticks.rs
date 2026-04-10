//! Nodes for primitives relating purely to the tick counter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, build_mir::translate_expr,
    },
    mir,
    sim::value::NlFloat,
    util::reflection::Reflect,
    workspace::Workspace,
};

#[derive(Debug, Clone)]
pub struct ResetTicks {
    pub workspace: Box<ExprKind>,
}

impl Expr for ResetTicks {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let ResetTicks { workspace } = self;
        p.add_fn_call("reset_ticks", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl ResetTicks {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = translate_expr(builder, &self.workspace)?;

        let operation = mir::Operation::CallHostFunction {
            function: &reset_ticks::FN_INFO,
            args: vec![mir::PlaceOperand::Copy(workspace_local.place())],
        };

        Some(builder.mir.add_operation(None, operation))
    }
}

mod reset_ticks {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "reset_ticks",
        parameter_types: &[<&mut Workspace>::TYPE],
        return_type: <()>::TYPE,
        link_name: "reset_ticks",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace) {
        workspace.world.tick_counter.reset();
    }
}

#[derive(Debug, Clone)]
pub struct AdvanceTick {
    pub workspace: Box<ExprKind>,
}

impl Expr for AdvanceTick {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let AdvanceTick { workspace } = self;
        p.add_fn_call("advance_tick", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl AdvanceTick {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = translate_expr(builder, &self.workspace)?;

        let operation = mir::Operation::CallHostFunction {
            function: &advance_tick::FN_INFO,
            args: vec![mir::PlaceOperand::Copy(workspace_local.place())],
        };

        Some(builder.mir.add_operation(None, operation))
    }
}

mod advance_tick {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "advance_tick",
        parameter_types: &[<&mut Workspace>::TYPE],
        return_type: <()>::TYPE,
        link_name: "advance_tick",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace) {
        // TODO(mvp) handle error, probably by returning it as a value
        workspace.world.tick_counter.advance().unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct GetTick {
    pub workspace: Box<ExprKind>,
}

impl Expr for GetTick {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let GetTick { workspace } = self;
        p.add_fn_call("get_tick", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl GetTick {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = translate_expr(builder, &self.workspace)?;

        let operation = mir::Operation::CallHostFunction {
            function: &get_tick::FN_INFO,
            args: vec![mir::PlaceOperand::Copy(workspace_local.place())],
        };

        Some(builder.mir.add_operation(None, operation))
    }
}

mod get_tick {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "get_tick",
        parameter_types: &[<&mut Workspace>::TYPE],
        return_type: <NlFloat>::TYPE,
        link_name: "get_tick",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace) -> NlFloat {
        // TODO(mvp) handle error, probably by returning result
        workspace.world.tick_counter.get().unwrap()
    }
}
