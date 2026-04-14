//! The `distancexy` reporter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy,
        build_mir::translate_expr,
        expr::{
            EuclideanDistanceNoWrap, GetTurtleVar, PointConstructor, patch_var_place,
            turtle_var_place,
        },
    },
    mir,
    sim::{
        patch::{PatchId, PatchVarDesc},
        topology::{self, Point},
        turtle::{TurtleId, TurtleVarDesc},
        value::NlFloat,
    },
    util::reflection::Reflect,
};

#[derive(Debug, Clone)]
pub struct Distancexy {
    pub workspace: Box<ExprKind>,
    pub agent: Box<ExprKind>,
    pub x: Box<ExprKind>,
    pub y: Box<ExprKind>,
}

impl Expr for Distancexy {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.agent);
        visitor(&self.x);
        visitor(&self.y);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.agent.as_mut());
        visitor(self.x.as_mut());
        visitor(self.y.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Distancexy { workspace, agent, x, y } = self;
        p.add_fn_call("distancexy", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| agent.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| x.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| y.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl Distancexy {
    pub fn desugar(self, names: NameContext) -> ExprKind {
        let Self { workspace, agent, x, y } = self;

        let agent_type = agent.output_type(names);
        if agent_type == NlAbstractTy::Turtle {
            ExprKind::from(EuclideanDistanceNoWrap {
                a: Box::new(ExprKind::from(GetTurtleVar {
                    workspace,
                    turtle: agent,
                    var: TurtleVarDesc::Pos,
                })),
                b: Box::new(ExprKind::from(PointConstructor { x, y })),
            })
        } else {
            panic!("unable to desugar");
        }
    }

    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, agent, x, y } = self;

        let x_local = translate_expr(builder, x)?;
        let y_local = translate_expr(builder, y)?;
        let reference_position = Point::mir_initialize_from_local(builder.mir, x_local, y_local);

        let agent_local = translate_expr(builder, agent)?;
        let agent_ty = builder.mir.type_of_place(&agent_local.place());

        let agent_position = if agent_ty.is::<TurtleId>() {
            let workspace = translate_expr(builder, workspace)?;
            // get the turtle location
            turtle_var_place(builder, workspace.place(), agent_local.place(), TurtleVarDesc::Pos)
        } else if agent_ty.is::<PatchId>() {
            let workspace = translate_expr(builder, workspace)?;
            // get the patch location
            patch_var_place(builder, workspace.place(), agent_local.place(), PatchVarDesc::Pos)
        } else {
            todo!("TODO(mvp) handle other agent types, namely {:?}", agent_ty);
        };

        let operation = mir::Operation::CallHostFunction {
            function: &euclidean_distance_no_wrap::FN_INFO,
            args: vec![
                mir::PlaceOperand::Copy(agent_position),
                mir::PlaceOperand::Copy(reference_position.place()),
            ],
        };

        Some(builder.mir.add_operation(None, operation))
    }
}

mod euclidean_distance_no_wrap {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "distancexy",
        parameter_types: &[<Point>::TYPE, <Point>::TYPE],
        return_type: <NlFloat>::TYPE,
        link_name: "distancexy",
        link_addr: call as *const u8,
    };

    pub fn call(a: Point, b: Point) -> NlFloat {
        topology::euclidean_distance_no_wrap(a, b)
    }
}
