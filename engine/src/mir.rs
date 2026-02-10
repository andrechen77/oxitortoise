// TODO(doc) all of MIR

use std::{fmt, rc::Rc};

use ambassador::{Delegate, delegatable_trait};
use derive_more::derive::{Display, From, TryInto};
use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::{
    exec::jit::InstallLir,
    mir::build_lir::LirInsnBuilder,
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

mod build_lir;
mod format;
pub mod node;
pub mod transforms;
pub mod type_inference;

pub use build_lir::{LirProgramBuilder, mir_to_lir};

new_key_type! {
    #[derive(Display)]
    #[display("{_0:?}")]
    pub struct FunctionId;
}

#[derive(derive_more::Debug)]
pub struct Program {
    pub globals: Box<[CustomVarDecl]>,
    pub globals_schema: Option<GlobalsSchema>,
    pub turtle_breeds: TurtleBreeds,
    pub custom_turtle_vars: Vec<CustomVarDecl>,
    /// None if the turtle schema has not been calculated yet.
    pub turtle_schema: Option<TurtleSchema>,
    pub custom_patch_vars: Vec<CustomVarDecl>,
    /// None if the patch schema has not been calculated yet.
    pub patch_schema: Option<PatchSchema>,
    pub functions: SecondaryMap<FunctionId, Function>,
    /// The set of all nodes in the program, where a "node" is some kind of
    /// computation, as in sea-of-nodes. A node can only belong to a single
    /// function at a time.
    #[debug(skip)]
    pub nodes: Nodes,
    /// The set of all local variables in the program. A local variable can
    /// only belong to a single function at a time.
    pub locals: SlotMap<LocalId, LocalDeclaration>,
}

// TODO This sucks pls separate the two levels that MIR has melded together
#[derive(derive_more::Debug)]
pub enum TurtleBreeds {
    Full(SecondaryMap<BreedId, Breed>),
    Partial(SlotMap<BreedId, BreedPartial>),
}

impl TurtleBreeds {
    pub fn name(&self, breed_id: BreedId) -> &str {
        match self {
            TurtleBreeds::Full(breeds) => &breeds[breed_id].name,
            TurtleBreeds::Partial(breeds) => &breeds[breed_id].name,
        }
    }
}

#[derive(Debug)]
pub struct BreedPartial {
    pub name: Rc<str>,
    pub singular_name: Rc<str>,
}

#[derive(Debug)]
pub struct CustomVarDecl {
    pub name: Rc<str>,
    pub ty: MirTy,
}

pub type Nodes = SlotMap<NodeId, NodeKind>;

#[derive(derive_more::Debug)]
pub struct Function {
    pub debug_name: Option<Rc<str>>,
    pub is_entrypoint: bool,
    /// A list of local variables which are parameters to the function. This
    /// includes implicit parameters such as the closure environment, the
    /// context pointer, and the executing agent.
    pub parameters: Vec<LocalId>,
    /// A list of all local variables in the function.
    pub locals: Vec<LocalId>,
    pub return_ty: MirTy,
    /// The root node that gets executed when the function is called.
    pub root_node: NodeId,
}

new_key_type! {
    pub struct LocalId;
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Debug)]
pub struct LocalDeclaration {
    pub debug_name: Option<Rc<str>>,
    pub ty: MirTy,
    pub storage: LocalStorage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalStorage {
    /// The local variable is stored on the stack, so its address can be taken.
    Stack,
    /// The local variable is stored in a virtual register.
    Register,
}

#[derive(Debug)]
pub struct WriteLirError;

/// A local transformation that can be applied to a node. The function
/// returns `true` if the transformation was successfully applied, `false`
/// otherwise.
pub type NodeTransform = Box<dyn FnOnce(&mut Program, FunctionId, NodeId) -> bool>;

/// Some kind of computation that takes inputs and produces outputs. The output
/// of a node is immutable, though may change between instances if the node is
/// part of a loop. The output of a node can be referenced by its node id.
/// The execution of a node may have side effects; if it does, then it is
/// incorrect to deduplicate nodes; if it doesn't, deduplication is correct.
#[delegatable_trait]
pub trait Node {
    fn is_pure(&self) -> bool;

    /// All nodes that this node depends on. Note that this doesn't mean the
    /// dependent nodes are always executed when the current node is executed;
    /// this might not be the case for the control flow nodes. The dependencies
    /// are returned with identifiers for debugging purposes.
    fn dependencies(&self) -> Vec<(&'static str, NodeId)>;

    /// For certain low level nodes it doesn't make sense to have an abstract
    /// output type; those should return `None`.
    fn output_type(&self, program: &Program, fn_id: FunctionId) -> MirTy;

    /// Returns a possible local transformation that could apply to this node.
    /// This can return at most one transformation, even if multiple are
    /// applicable. The implementation should attempt to choose the best
    /// transformation using local peephole analysis. The returned
    /// transformation, if any, is not guaranteed to be applicable; it will
    /// return `true` if anything was applied.
    fn peephole_transform(
        &self,
        program: &Program,
        fn_id: FunctionId,
        my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        let _ = program;
        let _ = fn_id;
        let _ = my_node_id;
        None
    }

    /// Attempt to expand the node into a lower level representation.
    /// Functionally identical to [`Node::peephole_transform`], but can assume
    /// a later stage of the compilation process (e.g. schemas are defined
    /// and everything).
    fn lowering_expand(
        &self,
        program: &Program,
        fn_id: FunctionId,
        my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        let _ = program;
        let _ = fn_id;
        let _ = my_node_id;
        None
    }

    // TODO(mvp_ants) the LIR execution of a node should depend on the access
    // type, e.g. by-value, by-reference, etc.
    /// Writes the LIR instructions that correspond to the calculation
    /// represented by this node. This means executing any necessary side
    /// effects and making the LIR values for this node's outputs available. It
    /// is *not* a precondition that all dependent nodes named in
    /// `self.dependencies()` have also had their LIR instructions written. If
    /// this node depends on other nodes that haven't had their LIR instructions
    /// written, this function should execute those instructions first.
    /// Implementations may use [`LirInsnBuilder::get_node_results`] to get
    /// the LIR values for the dependencies, which will recursively call
    /// `write_lir_execution` if necessary.
    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let _ = program;
        let _ = my_node_id;
        let _ = lir_builder;
        Err(WriteLirError)
    }

    fn pretty_print(&self, program: &Program, out: impl fmt::Write) -> fmt::Result;
}

use node::*;

#[derive(Debug, From, TryInto, Delegate)]
#[try_into(owned, ref, ref_mut)]
#[delegate(Node)]
pub enum NodeKind {
    Agentset(Agentset),
    AdvanceTick(AdvanceTick),
    Ask(Ask),
    BinaryOperation(BinaryOperation),
    Block(Block),
    Break(Break),
    CallUserFn(CallUserFn),
    CanMove(CanMove),
    CheckNobody(CheckNobody),
    ClearAll(ClearAll),
    Closure(Closure),
    Constant(Constant),
    CreateTurtles(CreateTurtles),
    DeriveElement(DeriveElement),
    DeriveField(DeriveField),
    Diffuse(Diffuse),
    Distancexy(Distancexy),
    EuclideanDistanceNoWrap(EuclideanDistanceNoWrap),
    GetGlobalVar(GetGlobalVar),
    GetLocalVar(GetLocalVar),
    GetPatchVar(GetPatchVar),
    GetPatchVarAsTurtleOrPatch(GetPatchVarAsTurtleOrPatch),
    GetTick(GetTick),
    GetTurtleVar(GetTurtleVar),
    IfElse(IfElse),
    ListLiteral(ListLiteral),
    MaxPxcor(MaxPxcor),
    MaxPycor(MaxPycor),
    MemLoad(MemLoad),
    MemStore(MemStore),
    Of(Of),
    OffsetDistanceByHeading(OffsetDistanceByHeading),
    OneOf(OneOf),
    PatchAt(PatchAt),
    PatchRelative(PatchRelative),
    PointConstructor(PointConstructor),
    RandomInt(RandomInt),
    ResetTicks(ResetTicks),
    Repeat(Repeat),
    ScaleColor(ScaleColor),
    SetDefaultShape(SetDefaultShape),
    SetLocalVar(SetLocalVar),
    SetPatchVar(SetPatchVar),
    SetPatchVarAsTurtleOrPatch(SetPatchVarAsTurtleOrPatch),
    SetTurtleVar(SetTurtleVar),
    TurtleForward(TurtleForward),
    TurtleIdToIndex(TurtleIdToIndex),
    TurtleRotate(TurtleRotate),
    UnaryOp(UnaryOp),
}

#[derive(Clone, PartialEq, Default)]
pub struct MirTy {
    /// An abstract type that conceptually makes sense in the NetLogo virtual
    /// machine. This may be `None` if there is no abstract type that makes
    /// sense (e.g. an agent is internally represented as an index into a data
    /// structure).
    pub abstr: Option<NlAbstractTy>,
    /// A concrete type. This may be `None` if the concrete type is not known
    /// yet.
    pub concrete: Option<ConcreteTy>,
}

impl MirTy {
    pub fn repr(&self) -> ConcreteTy {
        if let Some(concrete) = self.concrete {
            concrete
        } else if let Some(abstr) = &self.abstr {
            abstr.repr()
        } else {
            panic!("MirTy is empty");
        }
    }
}

impl From<NlAbstractTy> for MirTy {
    fn from(value: NlAbstractTy) -> Self {
        Self { abstr: Some(value), concrete: None }
    }
}

impl From<ConcreteTy> for MirTy {
    fn from(value: ConcreteTy) -> Self {
        Self { abstr: None, concrete: Some(value) }
    }
}

impl std::fmt::Debug for MirTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let MirTy { abstr, concrete } = self;
        match (abstr, concrete) {
            (Some(abstr), Some(concrete)) => write!(f, "{:?} ({:?})", abstr, concrete),
            (Some(abstr), None) => write!(f, "{:?}", abstr),
            (None, Some(concrete)) => write!(f, "{:?}", concrete),
            (None, None) => write!(f, "unknown"),
        }
    }
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
            Self::Unit => <()>::CONCRETE_TY,
            Self::Top => PackedAny::CONCRETE_TY,
            Self::Bottom => unimplemented!("bottom type has no concrete representation"),
            Self::Numeric => NlFloat::CONCRETE_TY,
            Self::Color => Color::CONCRETE_TY,
            Self::Float => NlFloat::CONCRETE_TY,
            Self::Boolean => NlBool::CONCRETE_TY,
            Self::String => todo!(),
            Self::Point => Point::CONCRETE_TY,
            Self::Agent => PackedAny::CONCRETE_TY,
            Self::Patch => OptionPatchId::CONCRETE_TY,
            Self::Turtle => TurtleId::CONCRETE_TY,
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
            Self::List { element_ty } if **element_ty == Self::Top => <NlBox<NlList>>::CONCRETE_TY,
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

// TODO this doesn't need to be a visitor; just take a closure instead
pub trait MirVisitor {
    fn visit_node(&mut self, program: &Program, fn_id: FunctionId, node_id: NodeId) {
        let _ = program;
        let _ = fn_id;
        let _ = node_id;
    }
}

pub fn visit_mir_function<V: MirVisitor>(visitor: &mut V, program: &Program, fn_id: FunctionId) {
    visit_node_recursive(visitor, program, fn_id, program.functions[fn_id].root_node);
}

fn visit_node_recursive<V: MirVisitor>(
    visitor: &mut V,
    program: &Program,
    fn_id: FunctionId,
    node_id: NodeId,
) {
    visitor.visit_node(program, fn_id, node_id);

    let dependencies = program.nodes[node_id].dependencies();
    for (_, dependency) in dependencies {
        visit_node_recursive(visitor, program, fn_id, dependency);
    }
}
