//! Primitives relating purely to the topology of the world.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub struct OffsetDistanceByHeading {
    pub position: Box<ExprKind>,
    pub amt: Box<ExprKind>,
    pub heading: Box<ExprKind>,
}

impl Expr for OffsetDistanceByHeading {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Point
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.position);
        visitor(&self.amt);
        visitor(&self.heading);
    }

    fn write_mir_execution(
        &self,
        _builder: &mut HirToMirFnBuilder,
        _local_out: mir::LocalId,
    ) {
        todo!("TODO(mvp) write MIR execution for OffsetDistanceByHeading")
    }
}

#[derive(Debug)]
pub struct PatchAt {
    pub x: Box<ExprKind>,
    pub y: Box<ExprKind>,
}

impl Expr for PatchAt {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Patch
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.x);
        visitor(&self.y);
    }

    fn write_mir_execution(
        &self,
        _builder: &mut HirToMirFnBuilder,
        _local_out: mir::LocalId,
    ) {
        todo!("TODO(mvp) write MIR execution for PatchAt")
    }
}

#[derive(Debug)]
pub struct MaxPxcor;

impl Expr for MaxPxcor {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {}

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for MaxPxcor")
    }
}

#[derive(Debug)]
pub struct MaxPycor;

impl Expr for MaxPycor {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {}

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for MaxPycor")
    }
}

#[derive(Debug)]
pub struct EuclideanDistanceNoWrap {
    pub a: Box<ExprKind>,
    pub b: Box<ExprKind>,
}

impl Expr for EuclideanDistanceNoWrap {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.a);
        visitor(&self.b);
    }

    fn write_mir_execution(
        &self,
        _builder: &mut HirToMirFnBuilder,
        _local_out: mir::LocalId,
    ) {
        todo!("TODO(mvp) write MIR execution for EuclideanDistanceNoWrap")
    }
}

#[derive(Debug)]
pub struct PointConstructor {
    pub x: Box<ExprKind>,
    pub y: Box<ExprKind>,
}

impl Expr for PointConstructor {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Point
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.x);
        visitor(&self.y);
    }

    fn write_mir_execution(
        &self,
        _builder: &mut HirToMirFnBuilder,
        _local_out: mir::LocalId,
    ) {
        todo!("TODO(mvp) write MIR execution for PointConstructor")
    }
}

