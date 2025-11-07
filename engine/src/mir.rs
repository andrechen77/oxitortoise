// TODO add documentation about MIR

use std::rc::Rc;

use ambassador::{Delegate, delegatable_trait};
use derive_more::derive::{Display, From};
use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::{
    mir::build_lir::LirInsnBuilder,
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        turtle::BreedId,
        value::NetlogoMachineType,
    },
    util::cell::RefCell,
};

mod build_lir;
mod graphviz;
pub mod node;
pub mod transforms;

pub use build_lir::{HostFunctionIds, LirProgramBuilder, mir_to_lir};

new_key_type! {
    #[derive(Display)]
    #[display("{_0:?}")]
    pub struct GlobalId;
}

new_key_type! {
    #[derive(Display)]
    #[display("{_0:?}")]
    pub struct FunctionId;
}

#[derive(Default, Debug)]
pub struct Program {
    pub globals: SlotMap<GlobalId, ()>,
    pub turtle_breeds: SlotMap<BreedId, ()>,
    pub custom_turtle_vars: Vec<CustomVarDecl>,
    /// None if the turtle schema has not been calculated yet.
    pub turtle_schema: Option<TurtleSchema>,
    pub custom_patch_vars: Vec<CustomVarDecl>,
    /// None if the patch schema has not been calculated yet.
    pub patch_schema: Option<PatchSchema>,
    pub functions: SecondaryMap<FunctionId, RefCell<Function>>,
}

#[derive(Debug)]
pub struct CustomVarDecl {
    pub name: Rc<str>,
    pub ty: NetlogoAbstractType,
}

pub type Nodes = SlotMap<NodeId, EffectfulNodeKind>;

#[derive(derive_more::Debug)]
pub struct Function {
    pub debug_name: Option<Rc<str>>,
    /// A list of local variables which are parameters to the function. This
    /// includes implicit parameters such as the closure environment, the
    /// context pointer, and the executing agent.
    pub parameters: Vec<LocalId>,
    pub return_ty: NetlogoAbstractType,
    /// The set of all local variables used by the function.
    pub locals: SlotMap<LocalId, LocalDeclaration>,
    /// The structured control flow of the function
    #[debug(skip)]
    pub cfg: StatementBlock,
    /// The set of all nodes in the function, where a "node" is some kind of
    /// computation, as in sea-of-nodes.
    #[debug(skip)]
    pub nodes: RefCell<Nodes>,
}

new_key_type! {
    pub struct LocalId;
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Debug)]
pub struct LocalDeclaration {
    pub debug_name: Option<Rc<str>>,
    pub ty: MirType,
    pub storage: LocalStorage,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LocalStorage {
    /// The local variable is stored on the stack, so its address can be taken.
    Stack,
    /// The local variable is stored in a virtual register.
    Register,
}

#[derive(Debug, Default)]
pub struct StatementBlock {
    pub statements: Vec<StatementKind>,
}

#[derive(Debug)]
pub enum StatementKind {
    Node(NodeId),
    IfElse { condition: NodeId, then_block: StatementBlock, else_block: StatementBlock },
    Repeat { num_repetitions: NodeId, block: StatementBlock },
    Return { value: NodeId },
    Stop,
}

#[derive(Debug)]
pub struct WriteLirError;

/// A local transformation that can be applied to a node. The function
/// returns `true` if the transformation was successfully applied, `false`
/// otherwise.
pub type NodeTransform = Box<dyn Fn(&Program, FunctionId, NodeId) -> bool>;

/// Some kind of computation that takes inputs and produces outputs. The output
/// of a node is immutable, though may change between instances if the node is
/// part of a loop. The output of a node can be referenced by its node id.
/// The execution of a node may have side effects; if it does, then it is
/// incorrect to deduplicate nodes; if it doesn't, deduplication is correct.
#[delegatable_trait]
pub trait EffectfulNode {
    fn has_side_effects(&self) -> bool;

    fn dependencies(&self) -> Vec<NodeId>;

    /// For certain low level nodes it doesn't make sense to have an abstract
    /// output type; those should return `None`.
    fn output_type(&self, program: &Program, function: &Function, nodes: &Nodes) -> MirType;

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
    /// Functionally identical to [`EffectfulNode::transform`], but can assume
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
    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let _ = my_node_id;
        let _ = lir_builder;
        let _ = nodes;
        Err(WriteLirError)
    }
}

use node::*;

#[derive(Debug, Display, From, Delegate)]
#[delegate(EffectfulNode)]
pub enum EffectfulNodeKind {
    Agentset(Agentset),
    AdvanceTick(AdvanceTick),
    Ask(Ask),
    AskAllTurtles(AskAllTurtles),
    BinaryOperation(BinaryOperation),
    CallUserFn(CallUserFn),
    CanMove(CanMove),
    ClearAll(ClearAll),
    Closure(Closure),
    Constant(Constant),
    CreateTurtles(CreateTurtles),
    DeriveElement(DeriveElement),
    DeriveField(DeriveField),
    Diffuse(Diffuse),
    Distancexy(Distancexy),
    GetLocalVar(GetLocalVar),
    GetPatchVar(GetPatchVar),
    GetPatchVarAsTurtleOrPatch(GetPatchVarAsTurtleOrPatch),
    GetTick(GetTick),
    GetTurtleVar(GetTurtleVar),
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
    RandomInt(RandomInt),
    ResetTicks(ResetTicks),
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

#[derive(Clone, Debug, PartialEq)]
pub enum MirType {
    Abstract(NetlogoAbstractType),
    Machine(NetlogoMachineType),
    /// The "no one cares what type this is" value
    Other,
}

impl MirType {
    pub fn repr(&self) -> NetlogoMachineType {
        match self {
            MirType::Abstract(ty) => ty.repr(),
            MirType::Machine(ty) => ty.clone(),
            MirType::Other => unimplemented!(),
        }
    }
}

/// A representation of an element of the lattice making up all NetLogo types.
#[derive(PartialEq, derive_more::Debug, Clone)]
pub enum NetlogoAbstractType {
    Unit,
    /// Top doesn't actually include everything
    Top,
    Numeric,
    Color,
    Integer,
    Float,
    Boolean,
    String,
    Agent,
    Patch,
    Turtle,
    Link,
    Agentset {
        agent_type: Box<NetlogoAbstractType>,
    },
    Nobody,
    Closure {
        return_ty: Box<NetlogoAbstractType>,
    },
    List {
        element_ty: Box<NetlogoAbstractType>,
    },
    // TODO add more
}

impl NetlogoAbstractType {
    pub fn join(&self, other: &NetlogoAbstractType) -> NetlogoAbstractType {
        let _ = other;
        todo!()
    }

    pub fn meet(&self, other: &NetlogoAbstractType) -> NetlogoAbstractType {
        let _ = other;
        todo!()
    }

    pub fn repr(&self) -> NetlogoMachineType {
        match self {
            Self::Unit => {
                panic!("No one should be asking about the representation of the unit type")
            }
            Self::Top => NetlogoMachineType::DYN_BOX,
            Self::Numeric => NetlogoMachineType::FLOAT,
            Self::Color => NetlogoMachineType::COLOR,
            Self::Integer => NetlogoMachineType::INTEGER,
            Self::Float => NetlogoMachineType::FLOAT,
            Self::Boolean => NetlogoMachineType::BOOLEAN,
            Self::String => NetlogoMachineType::STRING,
            Self::Agent => NetlogoMachineType::DYN_BOX,
            Self::Patch => NetlogoMachineType::PATCH_ID,
            Self::Turtle => NetlogoMachineType::TURTLE_ID,
            Self::Link => todo!("add link id"),
            Self::Agentset { agent_type: _ } => todo!("add agentset id"),
            Self::Nobody => todo!(),
            Self::Closure { return_ty: _ } => todo!(),
            Self::List { element_ty: _ } => todo!(),
        }
    }
}

pub trait MirVisitor {
    fn visit_statement(&mut self, statement: &StatementKind) {
        let _ = statement;
    }

    fn visit_node(&mut self, node_id: NodeId) {
        let _ = node_id;
    }
}

pub fn visit_mir_function<V: MirVisitor>(visitor: &mut V, function: &Function) {
    visit_statement_block_recursive(visitor, &function.cfg, &function.nodes);
}

fn visit_statement_block_recursive<V: MirVisitor>(
    visitor: &mut V,
    statement_block: &StatementBlock,
    nodes: &RefCell<Nodes>,
) {
    for statement in &statement_block.statements {
        visitor.visit_statement(statement);
        match statement {
            StatementKind::Node(node_id) => visit_node_recursive(visitor, *node_id, nodes),
            StatementKind::IfElse { condition, then_block, else_block } => {
                visit_node_recursive(visitor, *condition, nodes);
                visit_statement_block_recursive(visitor, then_block, nodes);
                visit_statement_block_recursive(visitor, else_block, nodes);
            }
            StatementKind::Repeat { num_repetitions, block } => {
                visit_node_recursive(visitor, *num_repetitions, nodes);
                visit_statement_block_recursive(visitor, block, nodes);
            }
            StatementKind::Return { value } => {
                visit_node_recursive(visitor, *value, nodes);
            }
            StatementKind::Stop => {
                // do nothing
            }
        }
    }
}

fn visit_node_recursive<V: MirVisitor>(visitor: &mut V, node_id: NodeId, nodes: &RefCell<Nodes>) {
    visitor.visit_node(node_id);

    let dependencies = nodes.borrow()[node_id].dependencies();
    for dependency in dependencies {
        visit_node_recursive(visitor, dependency, nodes);
    }
}
