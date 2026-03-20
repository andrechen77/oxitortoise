//! Nodes for getting and setting agent and global variables.

use crate::{
    exec::CanonExecutionContext,
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir::{self, prelude::*},
    sim::{
        observer::Globals,
        patch::{PatchVarDesc, Patches},
        turtle::{TurtleVarDesc, Turtles},
        world::World,
    },
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
        clone_to_uninit(
            builder.mir,
            local_out.place(),
            var.place,
            &var.ty.static_ty.as_ref().unwrap().clone,
        );
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

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: LocalId) {
        // calculate the turtle id
        let turtle_id = builder.translate_expr(&self.turtle);

        // project the turtle variable
        let var = turtle_var_place(builder, turtle_id, self.var);

        // perform load
        clone_to_uninit(
            builder.mir,
            local_out.place(),
            var.place,
            &var.ty.static_ty.unwrap().clone,
        );
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

impl Expr for SetTurtleVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle);
        visitor(&self.value);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        // calculate the value to store
        let value = builder.translate_expr(&self.value);

        // calculate the turtle id
        let turtle_id = builder.translate_expr(&self.turtle);

        // project the turtle variable
        let var = turtle_var_place(builder, turtle_id, self.var);

        // perform store
        move_to_init(builder.mir, var, value.place);
    }
}

fn turtle_var_place(
    builder: &mut HirToMirFnBuilder,
    turtle_id: TypedPlace,
    var: TurtleVarDesc,
) -> TypedPlace {
    let ptr_to_context = builder.context_param();
    let context = ptr_to_context.proj(Projection::Deref);
    let workspace = CanonExecutionContext::mir_project_workspace(context);
    let world = Workspace::mir_project_world(workspace);
    let turtles = World::mir_project_turtles(world);
    Turtles::mir_project_turtle_variable(builder.mir, builder.type_mapping, turtles, turtle_id, var)
}

#[derive(Debug)]
pub struct GetPatchVar {
    /// The patch whose variable is being gotten.
    pub patch: Box<ExprKind>,
    /// The variable to get.
    pub var: PatchVarDesc,
}

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

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: LocalId) {
        // calculate the patch id
        let patch_id = builder.translate_expr(&self.patch);

        // project the patch variable
        let var = patch_var_place(builder, patch_id, self.var);

        // perform load
        clone_to_uninit(
            builder.mir,
            local_out.place(),
            var.place,
            &var.ty.static_ty.unwrap().clone,
        );
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

impl Expr for SetPatchVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.patch);
        visitor(&self.value);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, _local_out: LocalId) {
        // calculate the value to store
        let value = builder.translate_expr(&self.value);

        // calculate the patch id
        let patch_id = builder.translate_expr(&self.patch);

        // project the patch variable
        let var = patch_var_place(builder, patch_id, self.var);

        // perform store
        move_to_init(builder.mir, var, value.place);
    }
}

fn patch_var_place(
    builder: &mut HirToMirFnBuilder,
    patch_id: TypedPlace,
    var: PatchVarDesc,
) -> TypedPlace {
    let ptr_to_context = builder.context_param();
    let context = ptr_to_context.proj(Projection::Deref);
    let workspace = CanonExecutionContext::mir_project_workspace(context);
    let world = Workspace::mir_project_world(workspace);
    let patches = World::mir_project_patches(world);
    Patches::mir_project_patch_variable(builder.mir, builder.type_mapping, patches, patch_id, var)
}

/// A node for getting an patch variable when the type of the agent is unknown.
#[derive(Debug)]
pub struct GetPatchVarAsTurtleOrPatch {
    /// The patch whose variable is being gotten, or the turtle who is getting
    /// the variable of the current patch.
    pub agent: Box<ExprKind>,
    /// The variable to get.
    pub var: PatchVarDesc,
}

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

/// Moves a value from one place to another. The source place is not
/// deinitialized. The destination place will not be deinitialized (i.e. it is
/// assumed to be uninitialized). Useful for loading variables from memory.
fn clone_to_uninit(
    builder: &mut FunctionBuilder,
    dst_uninit: Place,
    src: Place,
    clone_kind: &CloneKind,
) {
    match clone_kind {
        CloneKind::Copy => {
            builder.add_operation_with_dst(dst_uninit, Operation::Operand(PlaceOperand::Move(src)));
        }
        CloneKind::Dynamic { clone_fn_info, .. } => builder.add_operation_with_dst(
            dst_uninit,
            Operation::CallHostFunction {
                function: clone_fn_info,
                args: vec![PlaceOperand::Borrow(src)],
            },
        ),
        CloneKind::None => {
            panic!("Cannot load a variable from memory that is neither Copy nor Clone");
        }
    }
}

/// Moves a value from one place to another. This may potentially destroy the
/// source place if it is not Copy. The destination place is considered
/// initialized and will be deinitialized before the value is moved in.
fn move_to_init(builder: &mut FunctionBuilder, dst_init: TypedPlace, src: Place) {
    // deinitialize the destination place
    builder.add_statement(mir::Statement::Elementary(mir::ElementaryStatement::Drop {
        src: dst_init.place.clone(),
    }));

    // move the value into the place
    builder.add_operation_with_dst(dst_init.place, Operation::Operand(PlaceOperand::Move(src)));
}
