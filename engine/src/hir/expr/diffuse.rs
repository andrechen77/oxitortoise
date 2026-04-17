//! The `diffuse` command.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::NlAbstractTyAtom,
    sim::{agent_schema::AgentFieldDescriptor, value::NlFloat},
    util::reflection::Reflect,
    workspace::Workspace,
};

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, build_mir::translate_expr,
    },
    mir,
    sim::patch::PatchVarDesc,
};

#[derive(Debug, Clone)]
pub struct Diffuse {
    pub workspace: Box<ExprKind>,
    /// The patch variable to diffuse.
    pub variable: PatchVarDesc,
    /// The amount to diffuse.
    pub amt: Box<ExprKind>,
}

impl Expr for Diffuse {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTyAtom::Unit.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.amt);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.amt.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Diffuse { workspace, variable, amt } = self;
        p.add_fn_call("diffuse", |p| {
            p.add_fn_arg_with(|p| variable.pretty_print(p, names.custom_patch_vars()))?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| amt.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl Diffuse {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace = translate_expr(builder, &self.workspace)?;
        let amt = translate_expr(builder, &self.amt)?;

        let operation = mir::Operation::CallHostFunction {
            function: &diffuse_8_single_variable_buffer::FN_INFO,
            args: vec![mir::PlaceOperand::Copy(workspace.place()), mir::PlaceOperand::Move(amt)],
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod diffuse_8_single_variable_buffer {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "diffuse_8_single_variable_buffer",
        parameter_types: &[<&mut Workspace>::TYPE, AgentFieldDescriptor::TYPE, NlFloat::TYPE],
        return_type: <()>::TYPE,
        link_name: "diffuse_8_single_variable_buffer",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace, field: AgentFieldDescriptor, amt: NlFloat) {
        crate::sim::topology::diffuse::diffuse_8_single_variable_buffer(
            &mut workspace.world,
            field,
            amt,
        )
    }
}
