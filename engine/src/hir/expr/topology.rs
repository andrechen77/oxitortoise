//! Primitives relating purely to the topology of the world.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program, format::NameContext},
    mir,
};

#[derive(Debug, Clone)]
pub struct OffsetDistanceByHeading {
    pub workspace: Box<ExprKind>,
    pub position: Box<ExprKind>,
    pub amt: Box<ExprKind>,
    pub heading: Box<ExprKind>,
}

impl Expr for OffsetDistanceByHeading {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Point
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.position);
        visitor(&self.amt);
        visitor(&self.heading);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for OffsetDistanceByHeading")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let OffsetDistanceByHeading { workspace, position, amt, heading } = self;
        p.add_fn_call("offset_distance_by_heading", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| position.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| amt.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| heading.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct PatchAt {
    pub workspace: Box<ExprKind>,
    pub x: Box<ExprKind>,
    pub y: Box<ExprKind>,
}

impl Expr for PatchAt {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Patch
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.x);
        visitor(&self.y);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for PatchAt")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let PatchAt { workspace, x, y } = self;
        p.add_fn_call("patch_at", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| x.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| y.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct MaxPxcor {
    pub workspace: Box<ExprKind>,
}

impl Expr for MaxPxcor {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for MaxPxcor")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let MaxPxcor { workspace } = self;
        p.add_fn_call("max_pxcor", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct MaxPycor {
    pub workspace: Box<ExprKind>,
}

impl Expr for MaxPycor {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for MaxPycor")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let MaxPycor { workspace } = self;
        p.add_fn_call("max_pycor", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
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

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for EuclideanDistanceNoWrap")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let EuclideanDistanceNoWrap { a, b } = self;
        p.add_fn_call("euclidean_distance_no_wrap", |p| {
            p.add_fn_arg_with(|p| a.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| b.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
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

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for PointConstructor")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let PointConstructor { x, y } = self;
        p.add_fn_call("point_constructor", |p| {
            p.add_fn_arg_with(|p| x.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| y.pretty_print(p, names))?;
            Ok(())
        })
    }
}
