use std::fmt;

use crate::hir::{NameContext, NlAbstractTy};

use ambassador::{Delegate, delegatable_trait};
use derive_more::{From, TryInto};
use pretty_print::PrettyPrinter;

pub use self::{
    agent_var::*, agentset::*, arith_op::*, ask::*, clear::*, closure::*, color::*, constant::*,
    control_flow::*, create_agent::*, diffuse::*, distancexy::*, list_set_ops::*, local_var::*,
    rand::*, set_default_shape::*, ticks::*, topology::*, turtle_movement::*, user_fn::*,
};

mod agent_var;
mod agentset;
mod arith_op;
mod ask;
mod clear;
mod closure;
mod color;
mod constant;
mod control_flow;
mod create_agent;
mod diffuse;
mod distancexy;
mod list_set_ops;
mod local_var;
mod rand;
mod set_default_shape;
mod ticks;
mod topology;
mod turtle_movement;
mod user_fn;

/// Some kind of computation that takes inputs and produces outputs. The output
/// of an expression is immutable, though may change between instances if the
/// expression is evaluated in different instances (e.g. as part of a loop or in
/// different function calls).
#[delegatable_trait]
pub trait Expr: Sized + Into<ExprKind> {
    fn output_type(&self, names: NameContext) -> NlAbstractTy;

    fn visit_children(&self, visitor: impl FnMut(&ExprKind));

    /// Like [`visit_children`](Expr::visit_children), but allows mutating each
    /// child expression in place.
    fn visit_children_mut(&mut self, visitor: impl FnMut(&mut ExprKind));

    fn pretty_print<W: fmt::Write>(
        &self,
        printer: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result;
}

#[derive(Debug, From, TryInto, Delegate, Clone)]
#[try_into(owned, ref, ref_mut)]
#[delegate(Expr)]
pub enum ExprKind {
    Agentset(Agentset),
    AdvanceTick(AdvanceTick),
    Ask(Ask),
    BinaryArith(BinaryArith),
    BinaryBool(BinaryBool),
    BinaryCmp(BinaryCmp),
    Block(Block),
    Break(Break),
    CallUserFn(CallUserFn),
    CanMove(CanMove),
    ClearAll(ClearAll),
    Closure(Closure),
    UnitLiteral(UnitLiteral),
    NumberLiteral(NumberLiteral),
    StringLiteral(StringLiteral),
    NobodyLiteral(NobodyLiteral),
    CreateTurtles(CreateTurtles),
    Diffuse(Diffuse),
    Distancexy(Distancexy),
    EuclideanDistanceNoWrap(EuclideanDistanceNoWrap),
    GetGlobalVar(GetGlobalVar),
    GetLocalVar(GetLocalVar),
    GetPatchVar(GetPatchVar),
    GetTick(GetTick),
    GetTurtleVar(GetTurtleVar),
    IfElse(IfElse),
    LogicalNot(LogicalNot),
    ListLiteral(ListLiteral),
    MaxPxcor(MaxPxcor),
    MaxPycor(MaxPycor),
    Negate(Negate),
    Of(Of),
    OffsetDistanceByHeading(OffsetDistanceByHeading),
    OneOf(OneOf),
    PatchAt(PatchAt),
    PatchRelative(PatchRelative),
    PointConstructor(PointConstructor),
    RandomInt(RandomInt),
    ResetTicks(ResetTicks),
    // Repeat(Repeat),
    ScaleColor(ScaleColor),
    Scope(Scope),
    SetDefaultShape(SetDefaultShape),
    SetLocalVar(SetLocalVar),
    SetPatchVar(SetPatchVar),
    SetTurtleVar(SetTurtleVar),
    TurtleForward(TurtleForward),
    TurtleRotate(TurtleRotate),
}
