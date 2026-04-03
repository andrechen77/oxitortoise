//! Primitives for moving turtles.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    mir,
};

fn pretty_print_patch_loc_relation<W: fmt::Write>(
    rel: &PatchLocRelation,
    p: &mut PrettyPrinter<W>,
    names: NameContext,
) -> fmt::Result {
    match rel {
        PatchLocRelation::Ahead => write!(p, "Ahead"),
        PatchLocRelation::LeftAhead(h) => {
            write!(p, "LeftAhead(")?;
            h.pretty_print(p, names)?;
            write!(p, ")")
        }
        PatchLocRelation::RightAhead(h) => {
            write!(p, "RightAhead(")?;
            h.pretty_print(p, names)?;
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
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.angle);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.angle.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for TurtleRotate")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let TurtleRotate { workspace, turtle, angle } = self;
        p.add_fn_call("turtle_rotate", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| angle.pretty_print(p, names))?;
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
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.distance.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for TurtleForward")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let TurtleForward { workspace, turtle, distance } = self;
        p.add_fn_call("turtle_forward", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| distance.pretty_print(p, names))?;
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
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Boolean
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.distance.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for CanMove")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let CanMove { workspace, turtle, distance } = self;
        p.add_fn_call("can_move", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| distance.pretty_print(p, names))?;
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
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
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

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        match &mut self.relative_loc {
            PatchLocRelation::Ahead => {}
            PatchLocRelation::LeftAhead(heading) | PatchLocRelation::RightAhead(heading) => {
                visitor(heading.as_mut());
            }
        }
        visitor(self.turtle.as_mut());
        visitor(self.distance.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for PatchRelative")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let PatchRelative { workspace, relative_loc, turtle, distance } = self;
        p.add_fn_call("patch_relative", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| pretty_print_patch_loc_relation(relative_loc, p, names))?;
            p.add_fn_arg_with(|p| turtle.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| distance.pretty_print(p, names))?;
            Ok(())
        })
    }
}
