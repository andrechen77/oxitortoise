//! Primitives for moving turtles.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

fn pretty_print_patch_loc_relation<W: fmt::Write>(
    rel: &PatchLocRelation,
    p: &mut PrettyPrinter<W>,
    program: &Program,
) -> fmt::Result {
    match rel {
        PatchLocRelation::Ahead => write!(p, "Ahead"),
        PatchLocRelation::LeftAhead(h) => {
            write!(p, "LeftAhead(")?;
            h.pretty_print(p, program)?;
            write!(p, ")")
        }
        PatchLocRelation::RightAhead(h) => {
            write!(p, "RightAhead(")?;
            h.pretty_print(p, program)?;
            write!(p, ")")
        }
    }
}

#[derive(Debug, Clone)]
pub enum PatchLocRelation {
    Ahead,
    LeftAhead(Box<ExprKind>),
    RightAhead(Box<ExprKind>),
}

#[derive(Debug, Clone)]
pub struct TurtleRotate {
    pub workspace: Box<ExprKind>,
    pub turtle: Box<ExprKind>,
    pub angle: Box<ExprKind>,
}

impl Expr for TurtleRotate {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.angle);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for TurtleRotate")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let TurtleRotate { workspace, turtle, angle } = self;
        p.add_fn_call("turtle_rotate", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| angle.pretty_print(p, program))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct TurtleForward {
    pub workspace: Box<ExprKind>,
    pub turtle: Box<ExprKind>,
    pub distance: Box<ExprKind>,
}

impl Expr for TurtleForward {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for TurtleForward")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let TurtleForward { workspace, turtle, distance } = self;
        p.add_fn_call("turtle_forward", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| distance.pretty_print(p, program))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct CanMove {
    pub workspace: Box<ExprKind>,
    pub turtle: Box<ExprKind>,
    pub distance: Box<ExprKind>,
}

impl Expr for CanMove {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Boolean
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for CanMove")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let CanMove { workspace, turtle, distance } = self;
        p.add_fn_call("can_move", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| distance.pretty_print(p, program))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct PatchRelative {
    pub workspace: Box<ExprKind>,
    pub relative_loc: PatchLocRelation,
    pub turtle: Box<ExprKind>,
    pub distance: Box<ExprKind>,
}

impl Expr for PatchRelative {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Patch
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        match &self.relative_loc {
            PatchLocRelation::Ahead => {}
            PatchLocRelation::LeftAhead(heading) | PatchLocRelation::RightAhead(heading) => {
                visitor(heading)
            }
        }
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for PatchRelative")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let PatchRelative { workspace, relative_loc, turtle, distance } = self;
        p.add_fn_call("patch_relative", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| pretty_print_patch_loc_relation(relative_loc, p, program))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| distance.pretty_print(p, program))?;
            Ok(())
        })
    }
}
