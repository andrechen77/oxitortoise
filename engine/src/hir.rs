// TODO(doc) all of HIR

use std::{collections::BTreeMap, fmt, sync::Arc};

use ambassador::{Delegate, delegatable_trait};
use derive_more::derive::{Display, From, TryInto};
use pretty_print::PrettyPrinter;

use crate::{
    mir::{self, prelude::*},
    sim::turtle::{TurtleBreed, TurtleBreedId},
};

mod build_mir;
pub mod expr;

// TODO fix these modules
mod format;
// pub mod transforms;
// pub mod type_inference;

pub use build_mir::{HirToMirFnBuilder, TypeMapping};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Display, PartialOrd, Ord)]
#[display("{_0}")]
pub struct FunctionId(pub u32);

#[derive(derive_more::Debug)]
pub struct Program {
    pub global_vars: Box<[CustomVarDecl]>,
    // TODO this version of Breed contains type information (active custom
    // fields) that would not be available/ at the HIR stage of compilation;
    // consider using a more abstract version of Breed instead
    pub turtle_breeds: BTreeMap<TurtleBreedId, TurtleBreed>,
    pub custom_turtle_vars: Vec<CustomVarDecl>,
    pub custom_patch_vars: Vec<CustomVarDecl>,
    pub functions: BTreeMap<FunctionId, Function>,
}

#[derive(Debug)]
pub struct CustomVarDecl {
    pub name: Arc<str>,
    pub ty: NlAbstractTy,
}

#[derive(derive_more::Debug)]
pub struct Function {
    pub debug_name: Option<Arc<str>>,
    /// The list of parameters for the function. Evaluation of the function
    /// requires that the body be wrapped in a Scope expression that provides
    /// values for these parameters.
    pub parameters: Vec<(LocalId, LocalDecl)>,
    pub body: ExprKind,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub debug_name: Option<Arc<str>>,
    pub ty: NlAbstractTy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display("L{_0}")]
pub struct Label(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(pub u32);

/// Some kind of computation that takes inputs and produces outputs. The output
/// of an expression is immutable, though may change between instances if the
/// expression is evaluated in different instances (e.g. as part of a loop or in
/// different function calls).
#[delegatable_trait]
pub trait Expr {
    fn output_type(&self, program: &Program) -> NlAbstractTy;

    fn visit_children(&self, visitor: impl FnMut(&ExprKind));

    /// Writes the MIR statements that correspond to the calculation represented
    /// by this expression. This means executing any necessary side effelts and
    /// making the output of this node available in the given `local_out`. It is
    /// not a precondition that all dependent expressions have been executed.
    ///
    /// Implementations may use [`MirFunctionBuilder::translate_hir_node`] to get
    /// the MIR values for the dependencies, which will recursively call
    /// `write_mir_execution` if necessary.
    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId);

    fn pretty_print<W: fmt::Write>(
        &self,
        printer: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result;
}

#[derive(Debug, From, TryInto, Delegate, Clone)]
#[try_into(owned, ref, ref_mut)]
#[delegate(Expr)]
pub enum ExprKind {
    Agentset(expr::Agentset),
    AdvanceTick(expr::AdvanceTick),
    Ask(expr::Ask),
    BinaryArith(expr::BinaryArith),
    BinaryBool(expr::BinaryBool),
    BinaryCmp(expr::BinaryCmp),
    Block(expr::Block),
    Break(expr::Break),
    CallUserFn(expr::CallUserFn),
    CanMove(expr::CanMove),
    ClearAll(expr::ClearAll),
    Closure(expr::Closure),
    Constant(expr::Constant),
    CreateTurtles(expr::CreateTurtles),
    Diffuse(expr::Diffuse),
    Distancexy(expr::Distancexy),
    EuclideanDistanceNoWrap(expr::EuclideanDistanceNoWrap),
    GetGlobalVar(expr::GetGlobalVar),
    GetLocalVar(expr::GetLocalVar),
    GetPatchVar(expr::GetPatchVar),
    GetTick(expr::GetTick),
    GetTurtleVar(expr::GetTurtleVar),
    IfElse(expr::IfElse),
    LogicalNot(expr::LogicalNot),
    ListLiteral(expr::ListLiteral),
    MaxPxcor(expr::MaxPxcor),
    MaxPycor(expr::MaxPycor),
    Negate(expr::Negate),
    Of(expr::Of),
    OffsetDistanceByHeading(expr::OffsetDistanceByHeading),
    OneOf(expr::OneOf),
    PatchAt(expr::PatchAt),
    PatchRelative(expr::PatchRelative),
    PointConstructor(expr::PointConstructor),
    RandomInt(expr::RandomInt),
    ResetTicks(expr::ResetTicks),
    // Repeat(expr::Repeat),
    ScaleColor(expr::ScaleColor),
    Scope(expr::Scope),
    SetDefaultShape(expr::SetDefaultShape),
    SetLocalVar(expr::SetLocalVar),
    SetPatchVar(expr::SetPatchVar),
    SetTurtleVar(expr::SetTurtleVar),
    TurtleForward(expr::TurtleForward),
    TurtleRotate(expr::TurtleRotate),
}

/// A representation of an element of the lattice making up all NetLogo types.
#[derive(PartialEq, Debug, Clone, Eq, Hash, Default, Display)]
pub enum NlAbstractTy {
    // An independent type used to model a reference to the workspace itself.
    Workspace,
    // An independent type used to model a reference to the random number
    // generator itself.
    Rng,
    Unit,
    /// Supertype of all other types except for Workspace and Rng.
    NlTop,
    /// A type that has no inhabitants.
    #[default]
    Bottom,
    Numeric,
    Color,
    Float,
    Boolean,
    String,
    Point,
    Agent,
    Patch,
    Turtle,
    Link,
    Agentset {
        agent_type: Box<NlAbstractTy>,
    },
    Nobody,
    Closure(ClosureType),
    List {
        element_ty: Box<NlAbstractTy>,
    },
}

impl NlAbstractTy {
    /// Calculates the least upper bound of two types.
    pub fn join(self, other: NlAbstractTy) -> NlAbstractTy {
        if self == other {
            return self;
        }
        match self {
            Self::Workspace => panic!("Cannot join Workspace with other types"),
            Self::Rng => panic!("Cannot join Rng with other types"),
            Self::NlTop => Self::NlTop,
            Self::Bottom => other,
            _ => Self::NlTop,
        }
    }

    pub fn repr(&self) -> MirType {
        todo!(
            "We could just get rid of this entirely and have the type mappings be defined hir::TypeMapping"
        )
        // match self {
        //     Self::Unit => <()>::mir_type(),
        //     Self::NlTop => PackedAny::mir_type(),
        //     Self::Bottom => unimplemented!("bottom type has no concrete representation"),
        //     Self::Numeric => NlFloat::mir_type(),
        //     Self::Color => Color::mir_type(),
        //     Self::Float => NlFloat::mir_type(),
        //     Self::Boolean => bool::mir_type(),
        //     Self::String => todo!(),
        //     Self::Point => Point::mir_type(),
        //     Self::Agent => PackedAny::mir_type(),
        //     Self::Patch => OptionPatchId::mir_type(),
        //     Self::Turtle => TurtleId::mir_type(),
        //     Self::Link => todo!(""),
        //     Self::Agentset { agent_type: _ } => todo!(""),
        //     // If a type is just "nobody", then it is inhabited by only one
        //     // value and therefore holds no data. Operations that take the
        //     // nobody value as an operand typically see it as an inhabitant of
        //     // some other type, e.g. nobody as a patch id, or nobody as a turtle
        //     // id. This is why "nobody" just by itself has no concrete
        //     // representation.
        //     Self::Nobody => unimplemented!("nobody type has no concrete representation"),
        //     Self::Closure(_) => todo!(),
        //     Self::List { element_ty } if **element_ty == Self::NlTop => <NlBox<NlList>>::mir_type(),
        //     Self::List { element_ty: _ } => todo!(),
        // }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Display)]
#[display("({}) -> {}", arg_tys.iter().map(|ty| ty.to_string()).collect::<Vec<String>>().join(", "), return_ty)]
pub struct ClosureType {
    pub arg_tys: Vec<NlAbstractTy>,
    pub return_ty: Box<NlAbstractTy>,
}
