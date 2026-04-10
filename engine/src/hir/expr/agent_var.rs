//! Nodes for getting and setting agent and global variables.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy,
        build_mir::{self, translate_expr},
    },
    mir,
    sim::{
        observer::Globals,
        patch::{PatchVarDesc, Patches},
        turtle::{TurtleVarDesc, Turtles},
        world::World,
    },
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

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, names: NameContext) -> fmt::Result {
        write!(
            p,
            "get_global_var({}#{})",
            self.index,
            names.global_vars()[self.index].name.as_ref()
        )
    }
}

impl GetGlobalVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();
        let workspace = ptr_to_workspace.proj_deref();
        let world = Workspace::mir_project_world(workspace);
        let globals = World::mir_project_globals(world);
        let var =
            Globals::mir_project_global_var(builder.mir, builder.type_mapping, self.index, globals);
        let var_ty = builder.mir.type_of_place(&var);
        Some(build_mir::clone_to_new(builder.mir, var, &var_ty.static_ty.as_ref().unwrap().clone))
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

impl GetTurtleVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();

        // calculate the turtle id
        let turtle_id = translate_expr(builder, &self.turtle)?.place();

        // project the turtle variable
        let var = turtle_var_place(builder, ptr_to_workspace, turtle_id, self.var);
        let var_ty = builder.mir.type_of_place(&var);

        // perform load
        Some(build_mir::clone_to_new(builder.mir, var, &var_ty.static_ty.as_ref().unwrap().clone))
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

impl SetTurtleVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();

        // calculate the value to store
        let value = translate_expr(builder, &self.value)?;

        // calculate the turtle id
        let turtle_id = translate_expr(builder, &self.turtle)?.place();

        // project the turtle variable
        let var = turtle_var_place(builder, ptr_to_workspace, turtle_id, self.var);

        // perform store
        build_mir::move_to_init(builder.mir, var, value);

        Some(builder.mir.unit_local())
    }
}

pub fn turtle_var_place(
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

impl GetPatchVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();

        // calculate the patch id
        let patch_id = translate_expr(builder, &self.patch)?.place();

        // project the patch variable
        let var = patch_var_place(builder, ptr_to_workspace, patch_id, self.var);
        let var_ty = builder.mir.type_of_place(&var);

        // perform load
        Some(build_mir::clone_to_new(builder.mir, var, &var_ty.static_ty.unwrap().clone))
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

impl SetPatchVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();

        // calculate the value to store
        let value = translate_expr(builder, &self.value)?;

        // calculate the patch id
        let patch_id = translate_expr(builder, &self.patch)?.place();

        // project the patch variable
        let var = patch_var_place(builder, ptr_to_workspace, patch_id, self.var);

        // perform store
        build_mir::move_to_init(builder.mir, var, value);

        Some(builder.mir.unit_local())
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
