//! Nodes for getting and setting agent and global variables.

use crate::{
    exec::CanonExecutionContext,
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir::prelude::*,
    sim::{observer::Globals, patch::PatchVarDesc, turtle::TurtleVarDesc, world::World},
    util::reflection::CloneKind,
    workspace::Workspace,
};

#[derive(Debug)]
pub struct GetGlobalVar {
    pub index: usize,
}

impl Expr for GetGlobalVar {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        let Some(var) = program.globals.get(self.index) else {
            panic!("Unknown global var index: {:?}", self.index);
        };
        var.ty.clone()
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        let _ = &mut visitor;
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: LocalId) {
        let ptr_to_context = builder.context_param();
        let context = ptr_to_context.proj(Projection::Deref);
        let workspace = CanonExecutionContext::mir_project_workspace(context);
        let world = Workspace::mir_project_world(workspace);
        let globals = World::mir_project_globals(world);
        let var =
            Globals::mir_project_global_var(builder.mir, builder.type_mapping, self.index, globals);

        let var_type = builder.type_mapping.globals_schema().field_type(self.index);

        match var_type.clone {
            CloneKind::Copy => {
                builder.mir.add_operation_with_dst(
                    local_out.into(),
                    Operation::Operand(PlaceOperand::Move(var.place)),
                );
            }
            CloneKind::Dynamic { clone_fn_info, .. } => builder.mir.add_operation_with_dst(
                local_out.into(),
                Operation::CallHostFunction {
                    function: clone_fn_info,
                    args: vec![PlaceOperand::Borrow(var.place)],
                },
            ),
            CloneKind::None => {
                panic!("Cannot load a variable of type {:?} as a copy", var_type);
            }
        }
    }
}

// TODO(mvp) if this is a variable that may or may not exist depending on the
// breed, then we should check the breed of the turtle as well

#[derive(Debug)]
pub struct GetTurtleVar {
    /// The turtle whose variable is being gotten.
    pub turtle: Box<ExprKind>,
    /// The variable to get.
    pub var: TurtleVarDesc,
}

// impl Expr for GetTurtleVar {
//     // Not pure!  Its value depends on `set` calls within the same block.  --Jason B. (11/12/25)
//     fn is_pure(&self) -> bool {
//         false
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("context", self.context), ("turtle", self.turtle)]
//     }
//
//     fn output_type(&self, program: &Program) -> HirTy {
//         // TODO(wishlist) this should probably be refactored into a function
//         match self.var {
//             TurtleVarDesc::Who => NlAbstractTy::Float.into(),
//             TurtleVarDesc::Color => NlAbstractTy::Color.into(),
//             TurtleVarDesc::Size => NlAbstractTy::Float.into(),
//             TurtleVarDesc::Pos => NlAbstractTy::Point.into(),
//             TurtleVarDesc::Xcor => NlAbstractTy::Float.into(),
//             TurtleVarDesc::Ycor => NlAbstractTy::Float.into(),
//             TurtleVarDesc::Custom(field) => program.custom_turtle_vars[field].ty.clone(),
//         }
//     }
//
//     fn lowering_expand(
//         &self,
//         _program: &Program,
//         _fn_id: FunctionId,
//         _my_node_id: NodeId,
//     ) -> Option<NodeTransform> {
//         todo!()
//     }
//
//     fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         PrettyPrinter::new(&mut out).add_struct("GetTurtleVar", |p| {
//             p.add_field_with("var", |p| write!(p, "{:?}", self.var))?;
//             if let TurtleVarDesc::Custom(field) = self.var {
//                 p.add_comment(&program.custom_turtle_vars[field].name)?;
//             }
//             Ok(())
//         })
//     }
// }

impl Expr for GetTurtleVar {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        match self.var {
            TurtleVarDesc::Who => NlAbstractTy::Float,
            TurtleVarDesc::Color => NlAbstractTy::Color,
            TurtleVarDesc::Size => NlAbstractTy::Float,
            TurtleVarDesc::Pos => NlAbstractTy::Point,
            TurtleVarDesc::Xcor => NlAbstractTy::Float,
            TurtleVarDesc::Ycor => NlAbstractTy::Float,
            TurtleVarDesc::Custom(field) => program.custom_turtle_vars[field].ty.clone(),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for GetTurtleVar");
    }
}

#[derive(Debug)]
pub struct SetTurtleVar {
    /// The turtle whose variable is being set.
    pub turtle: Box<ExprKind>,
    /// The variable to set.
    pub var: TurtleVarDesc,
    /// The value to set the variable to.
    pub value: Box<ExprKind>,
}

// impl Expr for SetTurtleVar {
//     fn is_pure(&self) -> bool {
//         false
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("context", self.context), ("turtle", self.turtle), ("value", self.value)]
//     }
//     fn output_type(&self, _program: &Program) -> HirTy {
//         NlAbstractTy::Unit.into()
//     }
//
//     fn lowering_expand(
//         &self,
//         _program: &Program,
//         _fn_id: FunctionId,
//         _my_node_id: NodeId,
//     ) -> Option<NodeTransform> {
//         todo!()
//     }
//
//     fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         let var_name = match self.var {
//             TurtleVarDesc::Who => "who",
//             TurtleVarDesc::Color => "color",
//             TurtleVarDesc::Size => "size",
//             TurtleVarDesc::Pos => "pos",
//             TurtleVarDesc::Xcor => "xcor",
//             TurtleVarDesc::Ycor => "ycor",
//             TurtleVarDesc::Custom(field) => program.custom_turtle_vars[field].name.as_ref(),
//         };
//         PrettyPrinter::new(&mut out).add_struct("SetTurtleVar", |p| {
//             p.add_field_with("var", |p| write!(p, "{:?}", self.var))?;
//             p.add_comment(var_name)
//         })
//     }
// }

impl Expr for SetTurtleVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle);
        visitor(&self.value);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for SetTurtleVar");
    }
}

// /// Helper function to derive a pointer to the data row of a turtle's variable.
// /// Returns the NodeId of the node that outputs the pointer to data row, as well
// /// as the byte offset of the field within the data row. This is used by both
// /// loads and stores.
// fn context_to_turtle_data(
//     program: &mut Program,
//     context: NodeId,
//     turtle_id: NodeId,
//     var: TurtleVarDesc,
// ) -> (NodeId, usize) {
//     todo!()
// }

#[derive(Debug)]
pub struct TurtleIdToIndex {
    /// The turtle id to convert.
    pub turtle_id: Box<ExprKind>,
}

// impl Expr for TurtleIdToIndex {
//     fn is_pure(&self) -> bool {
//         true
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("turtle_id", self.turtle_id)]
//     }
//
//     fn output_type(&self, _program: &Program) -> HirTy {
//         U32_CONCRETE_TY.into()
//     }
//
//     fn write_lir_execution<I: InstallLir>(
//         &self,
//         program: &Program,
//         my_node_id: NodeId,
//         lir_builder: &mut LirInsnBuilder,
//     ) -> Result<(), WriteLirError> {
//         todo!()
//     }
//
//     fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         PrettyPrinter::new(&mut out).add_struct("TurtleIdToIndex", |_| Ok(()))
//     }
// }

impl Expr for TurtleIdToIndex {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Numeric
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle_id);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for TurtleIdToIndex");
    }
}

#[derive(Debug)]
pub struct GetPatchVar {
    /// The patch whose variable is being gotten.
    pub patch: Box<ExprKind>,
    /// The variable to get.
    pub var: PatchVarDesc,
}

// impl Expr for GetPatchVar {
//     fn is_pure(&self) -> bool {
//         false
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("context", self.context), ("patch", self.patch)]
//     }
//
//     fn output_type(&self, program: &Program) -> HirTy {
//         match self.var {
//             PatchVarDesc::Pcolor => NlAbstractTy::Color.into(),
//             PatchVarDesc::Pos => NlAbstractTy::Point.into(),
//             PatchVarDesc::Custom(field) => program.custom_patch_vars[field].ty.clone(),
//         }
//     }
//
//     fn lowering_expand(
//         &self,
//         _program: &Program,
//         _fn_id: FunctionId,
//         _my_node_id: NodeId,
//     ) -> Option<NodeTransform> {
//         todo!()
//     }
//
//     fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         PrettyPrinter::new(&mut out).add_struct("GetPatchVar", |p| {
//             p.add_field_with("var", |p| write!(p, "{:?}", self.var))?;
//             if let PatchVarDesc::Custom(field) = self.var {
//                 p.add_comment(&program.custom_patch_vars[field].name)?;
//             }
//             Ok(())
//         })
//     }
// }

impl Expr for GetPatchVar {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        match self.var {
            PatchVarDesc::Pcolor => NlAbstractTy::Color,
            PatchVarDesc::Pos => NlAbstractTy::Point,
            PatchVarDesc::Custom(field) => program.custom_patch_vars[field].ty.clone(),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.patch);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for GetPatchVar");
    }
}

#[derive(Debug)]
pub struct SetPatchVar {
    /// The patch whose variable is being set.
    pub patch: Box<ExprKind>,
    /// The variable to set.
    pub var: PatchVarDesc,
    /// The value to set the variable to.
    pub value: Box<ExprKind>,
}

// impl Expr for SetPatchVar {
//     fn is_pure(&self) -> bool {
//         false
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("context", self.context), ("patch", self.patch), ("value", self.value)]
//     }
//
//     fn output_type(&self, _program: &Program) -> HirTy {
//         NlAbstractTy::Unit.into()
//     }
//
//     fn lowering_expand(
//         &self,
//         _program: &Program,
//         _fn_id: FunctionId,
//         _my_node_id: NodeId,
//     ) -> Option<NodeTransform> {
//         todo!()
//     }
//
//     fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         PrettyPrinter::new(&mut out).add_struct("SetPatchVar", |p| {
//             p.add_field_with("var", |p| write!(p, "{:?}", self.var))?;
//             if let PatchVarDesc::Custom(field) = self.var {
//                 p.add_comment(&program.custom_patch_vars[field].name)?;
//             }
//             Ok(())
//         })
//     }
// }

impl Expr for SetPatchVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.patch);
        visitor(&self.value);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for SetPatchVar");
    }
}

// /// Helper function to derive a pointer to the data row of a patch's variable. Returns the
// /// the NodeId of the node that outputs the pointer to data row, as well as the byte
// /// offset of the field within the data row. This is used by both loads and stores.
// fn context_to_patch_data(
//     program: &mut Program,
//     context: NodeId,
//     patch_id: NodeId,
//     var: PatchVarDesc,
// ) -> (NodeId, usize) {
//     todo!()
// }

/// A node for getting an patch variable when the type of the agent is unknown.
#[derive(Debug)]
pub struct GetPatchVarAsTurtleOrPatch {
    /// The patch whose variable is being gotten, or the turtle who is getting
    /// the variable of the current patch.
    pub agent: Box<ExprKind>,
    /// The variable to get.
    pub var: PatchVarDesc,
}

// impl Expr for GetPatchVarAsTurtleOrPatch {
//     fn is_pure(&self) -> bool {
//         false
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("context", self.context), ("agent", self.agent)]
//     }
//
//     fn output_type(&self, program: &Program) -> HirTy {
//         // TODO(wishlist) refactor to deduplicate with GetPatchVar
//         match self.var {
//             PatchVarDesc::Pcolor => NlAbstractTy::Color.into(),
//             PatchVarDesc::Pos => NlAbstractTy::Point.into(),
//             PatchVarDesc::Custom(field) => program.custom_patch_vars[field].ty.clone(),
//         }
//     }
//
//     fn peephole_transform(
//         &self,
//         _program: &Program,
//         _fn_id: FunctionId,
//         _my_node_id: NodeId,
//     ) -> Option<NodeTransform> {
//         todo!()
//     }
//
//     fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         PrettyPrinter::new(&mut out).add_struct("GetPatchVarAsTurtleOrPatch", |p| {
//             p.add_field_with("var", |p| write!(p, "{:?}", self.var))?;
//             if let PatchVarDesc::Custom(field) = self.var {
//                 p.add_comment(&program.custom_patch_vars[field].name)?;
//             }
//             Ok(())
//         })
//     }
// }

impl Expr for GetPatchVarAsTurtleOrPatch {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        match self.var {
            PatchVarDesc::Pcolor => NlAbstractTy::Color,
            PatchVarDesc::Pos => NlAbstractTy::Point,
            PatchVarDesc::Custom(field) => program.custom_patch_vars[field].ty.clone(),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.agent);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for GetPatchVarAsTurtleOrPatch");
    }
}

#[derive(Debug)]
pub struct SetPatchVarAsTurtleOrPatch {
    /// The patch whose variable is being set, or the turtle who is setting
    /// the variable of the current patch.
    pub agent: Box<ExprKind>,
    /// The variable to set.
    pub var: PatchVarDesc,
    /// The value to set the variable to.
    pub value: Box<ExprKind>,
}

// impl Expr for SetPatchVarAsTurtleOrPatch {
//     fn is_pure(&self) -> bool {
//         false
//     }
//
//     fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
//         vec![("context", self.context), ("agent", self.agent), ("value", self.value)]
//     }
//
//     fn output_type(&self, _program: &Program) -> HirTy {
//         NlAbstractTy::Unit.into()
//     }
//
//     fn peephole_transform(
//         &self,
//         _program: &Program,
//         _fn_id: FunctionId,
//         _my_node_id: NodeId,
//     ) -> Option<NodeTransform> {
//         todo!()
//     }
//
//     fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
//         PrettyPrinter::new(&mut out).add_struct("SetPatchVarAsTurtleOrPatch", |p| {
//             p.add_field_with("var", |p| write!(p, "{:?}", self.var))?;
//             if let PatchVarDesc::Custom(field) = self.var {
//                 p.add_comment(&program.custom_patch_vars[field].name)?;
//             }
//             Ok(())
//         })
//     }
// }

impl Expr for SetPatchVarAsTurtleOrPatch {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.agent);
        visitor(&self.value);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        todo!("write_mir_execution for SetPatchVarAsTurtleOrPatch");
    }
}
