// TODO(doc) all of HIR

use std::{collections::BTreeMap, fmt, sync::Arc};

use ambassador::{Delegate, delegatable_trait};
use derive_more::derive::{Display, From, TryInto};
use pretty_print::PrettyPrinter;

use crate::{
    mir,
    sim::turtle::{TurtleBreed, TurtleBreedId},
};

mod build_mir;
pub mod expr;
mod format;
mod type_inference;

// TODO fix these modules
// pub mod transforms;

pub use build_mir::{HirToMirFnBuilder, TypeMapping};
pub use type_inference::narrow_types;

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
    pub function_bodies: BTreeMap<FunctionId, ExprKind>,
}

#[derive(Debug)]
pub struct CustomVarDecl {
    pub name: Arc<str>,
    pub ty: NlAbstractTy,
}

#[derive(derive_more::Debug)]
pub struct Function {
    pub debug_name: Arc<str>,
    /// The list of parameters for the function. Evaluation of the function
    /// requires that the body be wrapped in a Scope expression that provides
    /// values for these parameters.
    pub parameters: BTreeMap<LocalId, LocalDecl>,
    /// This is stored separately from the function body, so both must be updated
    /// when the function body is updated.
    pub return_ty: NlAbstractTy,
    /// Whether this function is an entrypoint.
    ///
    /// The arguments to entrypoint functions are set and not subject to
    /// narrowing type inference.
    pub is_entrypoint: bool,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub debug_name: Option<Arc<str>>,
    pub ty: NlAbstractTy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display("L{_0}")]
pub struct Label(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[display("V{_0}")]
pub struct LocalId(pub u32);

/// Some kind of computation that takes inputs and produces outputs. The output
/// of an expression is immutable, though may change between instances if the
/// expression is evaluated in different instances (e.g. as part of a loop or in
/// different function calls).
#[delegatable_trait]
pub trait Expr {
    fn output_type(&self, names: NameContext) -> NlAbstractTy;

    fn visit_children(&self, visitor: impl FnMut(&ExprKind));

    /// Like [`visit_children`](Expr::visit_children), but allows mutating each
    /// child expression in place.
    fn visit_children_mut(&mut self, visitor: impl FnMut(&mut ExprKind));

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
        names: NameContext,
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
    // TODO the meet and join methods should take &mut self and &other and just
    // modify self in place instead of requiring ownership

    /// Calculates the least upper bound of two types.
    pub fn join(self, other: NlAbstractTy) -> NlAbstractTy {
        use NlAbstractTy as Ty;
        match (self, other) {
            (Ty::Bottom, other) | (other, Ty::Bottom) => other,
            (a, b) if a == b => a,
            (Ty::Workspace, other) | (other, Ty::Workspace) => {
                panic!("Cannot join Workspace with type {:?}", other)
            }
            (Ty::Rng, other) | (other, Ty::Rng) => panic!("Cannot join Rng with type {:?}", other),
            (Ty::NlTop, _) | (_, Ty::NlTop) => Ty::NlTop,
            (_, _) => Ty::NlTop,
        }
    }

    /// Calculates the greatest lower bound of two types.
    pub fn meet(self, other: NlAbstractTy) -> NlAbstractTy {
        use NlAbstractTy as Ty;
        match (self, other) {
            (Ty::Bottom, _) | (_, Ty::Bottom) => Ty::Bottom,
            (a, b) if a == b => a,
            (Ty::Workspace, other) | (other, Ty::Workspace) => panic!(
                "Meeting Workspace with a non-bottom type ({:?}) would be Bottom, which is almost certainly a bug",
                other
            ),
            (Ty::Rng, other) | (other, Ty::Rng) => {
                panic!(
                    "Meeting Rng with a non-bottom type ({:?}) would be Bottom, which is almost certainly a bug",
                    other
                )
            }
            (Ty::NlTop, other) | (other, Ty::NlTop) => other,
            (a, b) => panic!(
                "Meeting incompatible types {:?} and {:?} would be Bottom, which is almost certainly a bug",
                a, b
            ),
        }
    }

    pub fn repr(&self) -> mir::MirType {
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

#[derive(Debug, Clone, Copy)]
pub enum NameContext<'a> {
    Global {
        global_vars: &'a [CustomVarDecl],
        turtle_breeds: &'a BTreeMap<TurtleBreedId, TurtleBreed>,
        custom_turtle_vars: &'a Vec<CustomVarDecl>,
        custom_patch_vars: &'a Vec<CustomVarDecl>,
        functions: &'a BTreeMap<FunctionId, Function>,
    },
    Local {
        local_vars: &'a BTreeMap<LocalId, LocalDecl>,
        parent: &'a NameContext<'a>,
    },
}

impl<'a> NameContext<'a> {
    pub fn from_program(program: &'a Program) -> Self {
        NameContext::Global {
            global_vars: &program.global_vars,
            turtle_breeds: &program.turtle_breeds,
            custom_turtle_vars: &program.custom_turtle_vars,
            custom_patch_vars: &program.custom_patch_vars,
            functions: &program.functions,
        }
    }

    pub fn from_program_mut(
        program: &'a mut Program,
    ) -> (Self, &'a mut BTreeMap<FunctionId, ExprKind>) {
        (
            NameContext::Global {
                global_vars: &program.global_vars,
                turtle_breeds: &program.turtle_breeds,
                custom_turtle_vars: &program.custom_turtle_vars,
                custom_patch_vars: &program.custom_patch_vars,
                functions: &program.functions,
            },
            &mut program.function_bodies,
        )
    }

    pub fn with_locals(&'a self, local_vars: &'a BTreeMap<LocalId, LocalDecl>) -> Self {
        NameContext::Local { local_vars, parent: self }
    }

    pub fn global_vars(&self) -> &'a [CustomVarDecl] {
        match self {
            NameContext::Global { global_vars, .. } => global_vars,
            NameContext::Local { parent, .. } => parent.global_vars(),
        }
    }

    pub fn turtle_breeds(&self) -> &'a BTreeMap<TurtleBreedId, TurtleBreed> {
        match self {
            NameContext::Global { turtle_breeds, .. } => turtle_breeds,
            NameContext::Local { parent, .. } => parent.turtle_breeds(),
        }
    }

    pub fn custom_turtle_vars(&self) -> &'a Vec<CustomVarDecl> {
        match self {
            NameContext::Global { custom_turtle_vars, .. } => custom_turtle_vars,
            NameContext::Local { parent, .. } => parent.custom_turtle_vars(),
        }
    }

    pub fn custom_patch_vars(&self) -> &'a Vec<CustomVarDecl> {
        match self {
            NameContext::Global { custom_patch_vars, .. } => custom_patch_vars,
            NameContext::Local { parent, .. } => parent.custom_patch_vars(),
        }
    }

    pub fn functions(&self) -> &'a BTreeMap<FunctionId, Function> {
        match self {
            NameContext::Global { functions, .. } => functions,
            NameContext::Local { parent, .. } => parent.functions(),
        }
    }

    pub fn lookup_local_var(&self, local_id: LocalId) -> Option<&'a LocalDecl> {
        match self {
            NameContext::Global { .. } => None,
            NameContext::Local { local_vars, parent } => {
                local_vars.get(&local_id).or_else(|| parent.lookup_local_var(local_id))
            }
        }
    }
}
