// TODO(doc) all of HIR

use std::sync::Arc;

use ambassador::{Delegate, delegatable_trait};
use derive_more::derive::{Display, From, TryInto};
use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::{
    hir::expr::{Break, Scope},
    mir,
    sim::{
        color::Color,
        observer::GlobalsSchema,
        patch::{OptionPatchId, PatchSchema},
        topology::Point,
        turtle::{Breed, BreedId, TurtleId, TurtleSchema},
        value::{NlBool, NlBox, NlFloat, NlList, PackedAny},
    },
    util::reflection::{ConcreteTy, Reflect},
};

mod build_mir;
pub mod expr;

// TODO fix these modules
// mod format;
// pub mod transforms;
// pub mod type_inference;

use build_mir::HirToLirFnBuilder;

new_key_type! {
    #[derive(Display)]
    #[display("{_0:?}")]
    pub struct FunctionId;
}

#[derive(derive_more::Debug)]
pub struct Program {
    pub globals: Box<[CustomVarDecl]>,
    pub globals_schema: Option<GlobalsSchema>,
    pub turtle_breeds: SlotMap<BreedId, Breed>,
    pub custom_turtle_vars: Vec<CustomVarDecl>,
    /// None if the turtle schema has not been calculated yet.
    pub turtle_schema: Option<TurtleSchema>,
    pub custom_patch_vars: Vec<CustomVarDecl>,
    /// None if the patch schema has not been calculated yet.
    pub patch_schema: Option<PatchSchema>,
    pub functions: SecondaryMap<FunctionId, Function>,
}

#[derive(Debug)]
pub struct CustomVarDecl {
    pub name: Arc<str>,
    pub ty: NlAbstractTy,
}

#[derive(derive_more::Debug)]
pub struct Function {
    pub debug_name: Option<Arc<str>>,
    pub is_entrypoint: bool,
    pub body: ExprKind,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub debug_name: Option<Arc<str>>,
    pub ty: NlAbstractTy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(u32);

/// Some kind of computation that takes inputs and produces outputs. The output
/// of an expression is immutable, though may change between instances if the
/// expression is evaluated in different instances (e.g. as part of a loop or in
/// different function calls).
#[delegatable_trait]
pub trait Expr {
    /// For certain low level nodes it doesn't make sense to have an abstract
    /// output type; those should return `None`.
    fn output_type(&self, program: &Program) -> NlAbstractTy;

    fn visit_childen(&self, visitor: impl FnMut(&ExprKind));

    /// Writes the MIR statements that correspond to the calculation represented
    /// by this expression. This means executing any necessary side effelts and
    /// making the output of this node available in the given `local_out`. It is
    /// not a precondition that all dependent expressions have been executed.
    ///
    /// Implementations may use [`MirFunctionBuilder::translate_hir_node`] to get
    /// the MIR values for the dependencies, which will recursively call
    /// `write_mir_execution` if necessary.
    fn write_mir_execution(&self, builder: &mut HirToLirFnBuilder, local_out: mir::LocalId);
}

#[derive(Debug, From, TryInto, Delegate)]
#[try_into(owned, ref, ref_mut)]
#[delegate(Expr)]
pub enum ExprKind {
    Scope(Scope),
    Break(Break),
    // Agentset(Agentset),
    // AdvanceTick(AdvanceTick),
    // Ask(Ask),
    // BinaryOperation(BinaryOperation),
    // Block(Block),
    // Break(Break),
    // CallUserFn(CallUserFn),
    // CanMove(CanMove),
    // CheckNobody(CheckNobody),
    // ClearAll(ClearAll),
    // Closure(Closure),
    // Constant(Constant),
    // CreateTurtles(CreateTurtles),
    // DeriveElement(DeriveElement),
    // DeriveField(DeriveField),
    // Diffuse(Diffuse),
    // Distancexy(Distancexy),
    // EuclideanDistanceNoWrap(EuclideanDistanceNoWrap),
    // GetGlobalVar(GetGlobalVar),
    // GetLocalVar(GetLocalVar),
    // GetPatchVar(GetPatchVar),
    // GetPatchVarAsTurtleOrPatch(GetPatchVarAsTurtleOrPatch),
    // GetTick(GetTick),
    // GetTurtleVar(GetTurtleVar),
    // IfElse(IfElse),
    // ListLiteral(ListLiteral),
    // MaxPxcor(MaxPxcor),
    // MaxPycor(MaxPycor),
    // MemLoad(MemLoad),
    // MemStore(MemStore),
    // Of(Of),
    // OffsetDistanceByHeading(OffsetDistanceByHeading),
    // OneOf(OneOf),
    // PatchAt(PatchAt),
    // PatchRelative(PatchRelative),
    // PointConstructor(PointConstructor),
    // RandomInt(RandomInt),
    // ResetTicks(ResetTicks),
    // Repeat(Repeat),
    // ScaleColor(ScaleColor),
    // SetDefaultShape(SetDefaultShape),
    // SetLocalVar(SetLocalVar),
    // SetPatchVar(SetPatchVar),
    // SetPatchVarAsTurtleOrPatch(SetPatchVarAsTurtleOrPatch),
    // SetTurtleVar(SetTurtleVar),
    // TurtleForward(TurtleForward),
    // TurtleIdToIndex(TurtleIdToIndex),
    // TurtleRotate(TurtleRotate),
    // UnaryOp(UnaryOp),
}

/// A representation of an element of the lattice making up all NetLogo types.
#[derive(PartialEq, Debug, Clone, Eq, Hash, Default, Display)]
pub enum NlAbstractTy {
    Unit,
    /// Top only includes types that make sense in the NetLogo environment.
    Top,
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
        if self == Self::Top {
            Self::Top
        } else if self == other {
            self
        } else if self == Self::Bottom {
            other
        } else {
            Self::Top
        }
        // TODO implement more granular least upper bound for other types
    }

    pub fn repr(&self) -> ConcreteTy {
        // TODO(mvp) add machine types for all abstract types
        match self {
            Self::Unit => <()>::ty(),
            Self::Top => PackedAny::ty(),
            Self::Bottom => unimplemented!("bottom type has no concrete representation"),
            Self::Numeric => NlFloat::ty(),
            Self::Color => Color::ty(),
            Self::Float => NlFloat::ty(),
            Self::Boolean => NlBool::ty(),
            Self::String => todo!(),
            Self::Point => Point::ty(),
            Self::Agent => PackedAny::ty(),
            Self::Patch => OptionPatchId::ty(),
            Self::Turtle => TurtleId::ty(),
            Self::Link => todo!(""),
            Self::Agentset { agent_type: _ } => todo!(""),
            // If a type is just "nobody", then it is inhabited by only one
            // value and therefore holds no data. Operations that take the
            // nobody value as an operand typically see it as an inhabitant of
            // some other type, e.g. nobody as a patch id, or nobody as a turtle
            // id. This is why "nobody" just by itself has no concrete
            // representation.
            Self::Nobody => unimplemented!("nobody type has no concrete representation"),
            Self::Closure(_) => todo!(),
            Self::List { element_ty } if **element_ty == Self::Top => <NlBox<NlList>>::ty(),
            Self::List { element_ty: _ } => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Display)]
#[display("{} -> {}", arg_ty, return_ty)]
pub struct ClosureType {
    pub arg_ty: Box<NlAbstractTy>,
    pub return_ty: Box<NlAbstractTy>,
}

impl ClosureType {
    // TODO(wishlist) this should be linked somehow to the machine-level calling
    // convention defined in jit.rs
    #[allow(dead_code)] // remove when used
    const PARAM_ENV_IDX: usize = 0;
    #[allow(dead_code)] // remove when used
    const PARAM_CONTEXT_IDX: usize = 1;
    const PARAM_ARG_IDX: usize = 2;
}
