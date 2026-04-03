//! Nodes for getting and setting agent and global variables.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    mir,
    sim::{
        observer::Globals,
        patch::{PatchVarDesc, Patches},
        turtle::{TurtleVarDesc, Turtles},
        world::World,
    },
    util::reflection::CloneKind,
    workspace::Workspace,
};

#[derive(Debug, Clone)]
pub struct GetGlobalVar {
    pub workspace: Box<ExprKind>,
    pub index: usize,
}

impl Expr for GetGlobalVar {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let Some(var) = names.global_vars().get(self.index) else {
            panic!("Unknown global var index: {:?}", self.index);
        };
        var.ty.clone()
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        let ptr_to_workspace = builder.workspace_param().place();
        let workspace = ptr_to_workspace.proj_deref();
        let world = Workspace::mir_project_world(workspace);
        let globals = World::mir_project_globals(world);
        let var =
            Globals::mir_project_global_var(builder.mir, builder.type_mapping, self.index, globals);
        let var_ty = builder.mir.type_of_place(&var);
        Some(clone_to_new(builder.mir, var, &var_ty.static_ty.as_ref().unwrap().clone))
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, names: NameContext) -> fmt::Result {
        write!(
            p,
            "get_global_var({}#{})",
            self.index,
            names.global_vars()[self.index].name.as_ref()
        )
    }
}

// TODO(mvp) if this is a variable that may or may not exist depending on the
// breed, then we should check the breed of the turtle as well

#[derive(Debug, Clone)]
pub struct GetTurtleVar {
    pub workspace: Box<ExprKind>,
    /// The turtle whose variable is being gotten.
    pub turtle: Box<ExprKind>,
    /// The variable to get.
    pub var: TurtleVarDesc,
}

impl Expr for GetTurtleVar {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        match self.var {
            TurtleVarDesc::Who => NlAbstractTy::Float,
            TurtleVarDesc::Color => NlAbstractTy::Color,
            TurtleVarDesc::Size => NlAbstractTy::Float,
            TurtleVarDesc::Pos => NlAbstractTy::Point,
            TurtleVarDesc::Xcor => NlAbstractTy::Float,
            TurtleVarDesc::Ycor => NlAbstractTy::Float,
            TurtleVarDesc::Custom(field) => names.custom_turtle_vars()[field].ty.clone(),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        let ptr_to_workspace = builder.workspace_param().place();

        // calculate the turtle id
        let turtle_id = self.turtle.write_mir_execution(builder)?;

        // project the turtle variable
        let var = turtle_var_place(builder, ptr_to_workspace, turtle_id, self.var);
        let var_ty = builder.mir.type_of_place(&var);

        // perform load
        Some(clone_to_new(builder.mir, var, &var_ty.static_ty.as_ref().unwrap().clone))
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let GetTurtleVar { var, workspace, turtle } = self;
        p.add_fn_call("get_turtle_var", |p| {
            p.add_fn_arg_with(|p| var.pretty_print(p, names.custom_turtle_vars()))?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetTurtleVar {
    pub workspace: Box<ExprKind>,
    /// The turtle whose variable is being set.
    pub turtle: Box<ExprKind>,
    /// The variable to set.
    pub var: TurtleVarDesc,
    /// The value to set the variable to.
    pub value: Box<ExprKind>,
}

impl Expr for SetTurtleVar {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.value);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.value.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        let ptr_to_workspace = builder.workspace_param().place();

        // calculate the value to store
        let value = self.value.write_mir_execution(builder)?;

        // calculate the turtle id
        let turtle_id = self.turtle.write_mir_execution(builder)?;

        // project the turtle variable
        let var = turtle_var_place(builder, ptr_to_workspace, turtle_id, self.var);

        // perform store
        move_to_init(builder.mir, var, value);

        Some(builder.mir.unit_local())
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let SetTurtleVar { var, workspace, turtle, value } = self;
        p.add_fn_call("set_turtle_var", |p| {
            p.add_fn_arg_with(|p| var.pretty_print(p, names.custom_turtle_vars()))?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| value.pretty_print(p, names))?;
            Ok(())
        })
    }
}

fn turtle_var_place(
    builder: &mut HirToMirFnBuilder,
    ptr_to_workspace: mir::Place,
    turtle_id: mir::Place,
    var: TurtleVarDesc,
) -> mir::Place {
    let workspace = ptr_to_workspace.proj_deref();
    let world = Workspace::mir_project_world(workspace);
    let turtles = World::mir_project_turtles(world);
    Turtles::mir_project_turtle_variable(builder.mir, builder.type_mapping, turtles, turtle_id, var)
}

#[derive(Debug, Clone)]
pub struct GetPatchVar {
    pub workspace: Box<ExprKind>,
    /// The patch whose variable is being gotten.
    pub patch: Box<ExprKind>,
    /// The variable to get.
    pub var: PatchVarDesc,
}

impl Expr for GetPatchVar {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        match self.var {
            PatchVarDesc::Pcolor => NlAbstractTy::Color,
            PatchVarDesc::Pos => NlAbstractTy::Point,
            PatchVarDesc::Custom(field) => names.custom_patch_vars()[field].ty.clone(),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.patch);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.patch.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        let ptr_to_workspace = builder.workspace_param().place();

        // calculate the patch id
        let patch_id = self.patch.write_mir_execution(builder)?;

        // project the patch variable
        let var = patch_var_place(builder, ptr_to_workspace, patch_id, self.var);
        let var_ty = builder.mir.type_of_place(&var);

        // perform load
        Some(clone_to_new(builder.mir, var, &var_ty.static_ty.unwrap().clone))
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let GetPatchVar { var, workspace, patch } = self;
        p.add_fn_call("get_patch_var", |p| {
            p.add_fn_arg_with(|p| var.pretty_print(p, names.custom_patch_vars()))?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| patch.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetPatchVar {
    pub workspace: Box<ExprKind>,
    /// The patch whose variable is being set.
    pub patch: Box<ExprKind>,
    /// The variable to set.
    pub var: PatchVarDesc,
    /// The value to set the variable to.
    pub value: Box<ExprKind>,
}

impl Expr for SetPatchVar {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.patch);
        visitor(&self.value);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.patch.as_mut());
        visitor(self.value.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        let ptr_to_workspace = builder.workspace_param().place();

        // calculate the value to store
        let value = self.value.write_mir_execution(builder)?;

        // calculate the patch id
        let patch_id = self.patch.write_mir_execution(builder)?;

        // project the patch variable
        let var = patch_var_place(builder, ptr_to_workspace, patch_id, self.var);

        // perform store
        move_to_init(builder.mir, var, value);

        Some(builder.mir.unit_local())
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let SetPatchVar { var, workspace, patch, value } = self;
        p.add_fn_call("set_patch_var", |p| {
            p.add_fn_arg_with(|p| var.pretty_print(p, names.custom_patch_vars()))?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| patch.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| value.pretty_print(p, names))?;
            Ok(())
        })
    }
}

fn patch_var_place(
    builder: &mut HirToMirFnBuilder,
    ptr_to_workspace: mir::Place,
    patch_id: mir::Place,
    var: PatchVarDesc,
) -> mir::Place {
    let workspace = ptr_to_workspace.proj_deref();
    let world = Workspace::mir_project_world(workspace);
    let patches = World::mir_project_patches(world);
    Patches::mir_project_patch_variable(builder.mir, builder.type_mapping, patches, patch_id, var)
}

/// Moves a value from one place to another. The source place is not
/// deinitialized. The destination place will not be deinitialized (i.e. it is
/// assumed to be uninitialized). Useful for loading variables from memory.
fn clone_to_new(
    builder: &mut mir::FunctionBuilder,
    src: mir::Place,
    clone_kind: &CloneKind,
) -> mir::Place {
    let dst =
        builder.create_local(mir::LocalDecl { debug_name: None, ty: builder.type_of_place(&src) });
    match clone_kind {
        CloneKind::Copy => builder.add_operation_with_dst(
            dst.place(),
            mir::Operation::Operand(mir::PlaceOperand::Move(src)),
        ),
        CloneKind::Dynamic { clone_fn_info, .. } => builder.add_operation_with_dst(
            dst.place(),
            mir::Operation::CallHostFunction {
                function: clone_fn_info,
                args: vec![mir::PlaceOperand::Borrow(src)],
            },
        ),
        CloneKind::None => {
            panic!("Cannot load a variable from memory that is neither Copy nor Clone");
        }
    }
    dst.place()
}

/// Moves a value from one place to another. This may potentially destroy the
/// source place if it is not Copy. The destination place is considered
/// initialized and will be deinitialized before the value is moved in.
fn move_to_init(builder: &mut mir::FunctionBuilder, dst_init: mir::Place, src: mir::Place) {
    // deinitialize the destination place
    builder.add_statement(mir::Statement::Elementary(mir::ElementaryStatement::Drop {
        src: dst_init.clone(),
    }));

    // move the value into the place
    builder.add_operation_with_dst(dst_init, mir::Operation::Operand(mir::PlaceOperand::Move(src)));
}
