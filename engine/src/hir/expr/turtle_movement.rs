//! Primitives for moving turtles.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;
use reflection::{Reflect, mir};

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, NlAbstractTyAtom,
        build_mir::translate_expr,
    },
    sim::{patch::OptionPatchId, turtle::TurtleId, value::NlFloat},
    workspace::Workspace,
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
        NlAbstractTyAtom::Unit.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.angle);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.angle.as_mut());
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

impl TurtleRotate {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, turtle, angle } = self;
        let workspace = translate_expr(builder, workspace)?;
        let turtle = translate_expr(builder, turtle)?;
        let angle = translate_expr(builder, angle)?;

        let operation = mir::Operation::CallHostFunction {
            function: &turtle_rotate::FN_INFO,
            args: vec![
                mir::PlaceOperand::Copy(workspace.place()),
                mir::PlaceOperand::Move(turtle),
                mir::PlaceOperand::Move(angle),
            ],
        };
        Some(builder.mir.add_operation(None, operation))
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
        NlAbstractTyAtom::Unit.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.distance.as_mut());
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

impl TurtleForward {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, turtle, distance } = self;
        let workspace = translate_expr(builder, workspace)?;
        let turtle = translate_expr(builder, turtle)?;
        let distance = translate_expr(builder, distance)?;

        let operation = mir::Operation::CallHostFunction {
            function: &turtle_forward::FN_INFO,
            args: vec![
                mir::PlaceOperand::Copy(workspace.place()),
                mir::PlaceOperand::Move(turtle),
                mir::PlaceOperand::Move(distance),
            ],
        };
        Some(builder.mir.add_operation(None, operation))
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
        NlAbstractTyAtom::Boolean.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.turtle);
        visitor(&self.distance);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.turtle.as_mut());
        visitor(self.distance.as_mut());
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

impl CanMove {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, turtle, distance } = self;

        let workspace = translate_expr(builder, workspace)?;
        let turtle = translate_expr(builder, turtle)?;
        let distance = translate_expr(builder, distance)?;

        let patch_ahead = builder.mir.add_operation(
            None,
            mir::Operation::CallHostFunction {
                function: &patch_ahead::FN_INFO,
                args: vec![
                    mir::PlaceOperand::Copy(workspace.place()),
                    mir::PlaceOperand::Copy(turtle.place()),
                    mir::PlaceOperand::Copy(distance.place()),
                ],
            },
        );

        let can_move = OptionPatchId::write_check_nobody(builder, true, patch_ahead.place());
        Some(can_move)
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
        NlAbstractTy::from_iter([NlAbstractTyAtom::Patch, NlAbstractTyAtom::Nobody])
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
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

impl PatchRelative {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { workspace, relative_loc, turtle, distance } = self;
        let workspace = translate_expr(builder, workspace)?;
        let turtle = translate_expr(builder, turtle)?;
        let distance = translate_expr(builder, distance)?;

        let operation = match relative_loc {
            PatchLocRelation::Ahead => mir::Operation::CallHostFunction {
                function: &patch_ahead::FN_INFO,
                args: vec![
                    mir::PlaceOperand::Copy(workspace.place()),
                    mir::PlaceOperand::Copy(turtle.place()),
                    mir::PlaceOperand::Copy(distance.place()),
                ],
            },
            PatchLocRelation::LeftAhead(heading) => {
                let distance_negated = builder.mir.add_operation(
                    None,
                    mir::Operation::UnaryOp {
                        opcode: mir::UnaryOpcode::FNeg,
                        operand: mir::PlaceOperand::Copy(distance.place()),
                    },
                );
                let heading = translate_expr(builder, heading)?;
                mir::Operation::CallHostFunction {
                    function: &patch_right_and_ahead::FN_INFO,
                    args: vec![
                        mir::PlaceOperand::Copy(workspace.place()),
                        mir::PlaceOperand::Copy(turtle.place()),
                        mir::PlaceOperand::Copy(heading.place()),
                        mir::PlaceOperand::Copy(distance_negated.place()),
                    ],
                }
            }
            PatchLocRelation::RightAhead(heading) => {
                let heading = translate_expr(builder, heading)?;
                mir::Operation::CallHostFunction {
                    function: &patch_right_and_ahead::FN_INFO,
                    args: vec![
                        mir::PlaceOperand::Copy(workspace.place()),
                        mir::PlaceOperand::Copy(turtle.place()),
                        mir::PlaceOperand::Copy(heading.place()),
                        mir::PlaceOperand::Copy(distance.place()),
                    ],
                }
            }
        };

        Some(builder.mir.add_operation(None, operation))
    }
}

mod patch_right_and_ahead {
    use reflection::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "patch_right_and_ahead",
        parameter_types: &[
            <&mut Workspace>::STATIC_TYPE,
            TurtleId::STATIC_TYPE,
            NlFloat::STATIC_TYPE,
            NlFloat::STATIC_TYPE,
        ],
        return_type: <OptionPatchId>::STATIC_TYPE,
        link_name: "patch_right_and_ahead",
        link_addr: call as *const u8,
    };

    pub fn call(
        workspace: &mut Workspace,
        turtle_id: TurtleId,
        angle: NlFloat,
        distance: NlFloat,
    ) -> OptionPatchId {
        let world = &mut workspace.world;
        let heading = world.turtles.get_turtle_heading(turtle_id).unwrap();
        let heading_right = *heading + angle;
        let position = world.turtles.get_turtle_position(turtle_id).unwrap();
        let pos_ahead =
            world.topology.offset_distance_by_heading(*position, heading_right, distance);
        if let Some(pos_ahead) = pos_ahead {
            world.topology.patch_at(pos_ahead.round_to_int()).into()
        } else {
            OptionPatchId::NOBODY
        }
    }
}

mod patch_ahead {
    use crate::sim::patch::OptionPatchId;
    use reflection::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "patch_ahead",
        parameter_types: &[
            <&mut Workspace>::STATIC_TYPE,
            TurtleId::STATIC_TYPE,
            NlFloat::STATIC_TYPE,
        ],
        return_type: <OptionPatchId>::STATIC_TYPE,
        link_name: "patch_ahead",
        link_addr: call as *const u8,
    };

    pub fn call(
        workspace: &mut Workspace,
        turtle_id: TurtleId,
        distance: NlFloat,
    ) -> OptionPatchId {
        let world = &mut workspace.world;
        let heading = world.turtles.get_turtle_heading(turtle_id).unwrap();
        let position = world.turtles.get_turtle_position(turtle_id).unwrap();
        let pos_ahead = world.topology.offset_distance_by_heading(*position, *heading, distance);
        if let Some(pos_ahead) = pos_ahead {
            world.topology.patch_at(pos_ahead.round_to_int()).into()
        } else {
            OptionPatchId::NOBODY
        }
    }
}

mod turtle_rotate {
    use reflection::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "turtle_rotate",
        parameter_types: &[
            <&mut Workspace>::STATIC_TYPE,
            TurtleId::STATIC_TYPE,
            NlFloat::STATIC_TYPE,
        ],
        return_type: <()>::STATIC_TYPE,
        link_name: "turtle_rotate",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace, turtle_id: TurtleId, angle: NlFloat) {
        if let Some(heading) = workspace.world.turtles.get_turtle_heading_mut(turtle_id) {
            *heading += angle;
        }
    }
}

mod turtle_forward {
    use reflection::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "turtle_forward",
        parameter_types: &[
            <&mut Workspace>::STATIC_TYPE,
            TurtleId::STATIC_TYPE,
            NlFloat::STATIC_TYPE,
        ],
        return_type: <()>::STATIC_TYPE,
        link_name: "turtle_forward",
        link_addr: call as *const u8,
    };

    pub fn call(workspace: &mut Workspace, turtle_id: TurtleId, distance: NlFloat) {
        let world = &mut workspace.world;
        let heading = world.turtles.get_turtle_heading(turtle_id).unwrap();
        let position = world.turtles.get_turtle_position(turtle_id).unwrap();
        let new_pos = world.topology.offset_distance_by_heading(*position, *heading, distance);
        if let Some(new_pos) = new_pos {
            *world.turtles.get_turtle_position_mut(turtle_id).unwrap() = new_pos;
        }
    }
}
