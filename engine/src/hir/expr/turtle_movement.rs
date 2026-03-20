//! Primitives for moving turtles.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub enum PatchLocRelation {
    Ahead,
    LeftAhead(Box<ExprKind>),
    RightAhead(Box<ExprKind>),
}

#[derive(Debug)]
pub struct TurtleRotate {
    pub turtle: Box<ExprKind>,
    pub angle: Box<ExprKind>,
}

impl Expr for TurtleRotate {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle);
        visitor(&self.angle);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for TurtleRotate")
    }
}

#[derive(Debug)]
pub struct TurtleForward {
    pub turtle: Box<ExprKind>,
    pub distance: Box<ExprKind>,
}

impl Expr for TurtleForward {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for TurtleForward")
    }
}

#[derive(Debug)]
pub struct CanMove {
    pub turtle: Box<ExprKind>,
    pub distance: Box<ExprKind>,
}

impl Expr for CanMove {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Boolean
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for CanMove")
    }
}

#[derive(Debug)]
pub struct PatchRelative {
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

    fn write_mir_execution(
        &self,
        _builder: &mut HirToMirFnBuilder,
        _local_out: mir::LocalId,
    ) {
        todo!("TODO(mvp) write MIR execution for PatchRelative")
    }
}

