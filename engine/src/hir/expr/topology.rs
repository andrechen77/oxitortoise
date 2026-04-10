//! Primitives relating purely to the topology of the world.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy,
        build_mir::{self, translate_expr},
    },
    mir,
    sim::{
        patch::PatchId,
        topology::{Point, Topology},
        world::World,
    },
    util::reflection::Reflect,
    workspace::Workspace,
};

#[derive(Debug, Clone)]
pub struct OffsetDistanceByHeading {
    pub workspace: Box<ExprKind>,
    pub position: Box<ExprKind>,
    pub amt: Box<ExprKind>,
    pub heading: Box<ExprKind>,
}

impl Expr for OffsetDistanceByHeading {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Point
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.position);
        visitor(&self.amt);
        visitor(&self.heading);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.position.as_mut());
        visitor(self.amt.as_mut());
        visitor(self.heading.as_mut());
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
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Patch
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.x);
        visitor(&self.y);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.x.as_mut());
        visitor(self.y.as_mut());
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

impl PatchAt {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, x, y } = self;
        let workspace = translate_expr(builder, workspace)?;
        let x = translate_expr(builder, x)?;
        let y = translate_expr(builder, y)?;

        let operation = mir::Operation::CallHostFunction {
            function: &patch_at::FN_INFO,
            args: vec![
                mir::PlaceOperand::Copy(workspace.place()),
                mir::PlaceOperand::Move(x),
                mir::PlaceOperand::Move(y),
            ],
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

#[derive(Debug, Clone)]
pub struct MaxPxcor {
    pub workspace: Box<ExprKind>,
}

impl Expr for MaxPxcor {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
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

impl MaxPxcor {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();
        let workspace = ptr_to_workspace.proj_deref();
        let world = Workspace::mir_project_world(workspace);
        let topology = World::mir_project_topology(world);
        let max_pxcor = Topology::mir_project_max_pxcor(topology);
        let max_pxcor_ty = builder.mir.type_of_place(&max_pxcor);
        Some(build_mir::clone_to_new(
            builder.mir,
            max_pxcor,
            &max_pxcor_ty.static_ty.as_ref().unwrap().clone,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct MaxPycor {
    pub workspace: Box<ExprKind>,
}

impl Expr for MaxPycor {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
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

impl MaxPycor {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let ptr_to_workspace = translate_expr(builder, &self.workspace)?.place();
        let workspace = ptr_to_workspace.proj_deref();
        let world = Workspace::mir_project_world(workspace);
        let topology = World::mir_project_topology(world);
        let max_pycor = Topology::mir_project_max_pycor(topology);
        let max_pycor_ty = builder.mir.type_of_place(&max_pycor);
        Some(build_mir::clone_to_new(
            builder.mir,
            max_pycor,
            &max_pycor_ty.static_ty.as_ref().unwrap().clone,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct EuclideanDistanceNoWrap {
    pub a: Box<ExprKind>,
    pub b: Box<ExprKind>,
}

impl Expr for EuclideanDistanceNoWrap {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.a);
        visitor(&self.b);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.a.as_mut());
        visitor(self.b.as_mut());
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
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Point
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.x);
        visitor(&self.y);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.x.as_mut());
        visitor(self.y.as_mut());
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

mod patch_at {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "patch_at",
        parameter_types: &[<&mut Workspace>::TYPE, Point::TYPE],
        return_type: PatchId::TYPE,
        link_name: "patch_at",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace, point: Point) -> PatchId {
        let point_int = point.round_to_int();
        workspace.world.topology.patch_at(point_int)
    }
}
