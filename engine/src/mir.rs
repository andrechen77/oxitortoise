// TODO add documentationa bout MIR. some points to include:
// - in MIR, a "local variable" doesn't necessarily need to correspond to a
// local variable in the NetLogo source code. it simply means a value that the
// function needs for later. it can refer to a temporary value as well. for
// example, when evaluating (a * b) + c, (a * b) will be computed and stored
// into a local variable, and then the variable will be used as an operand to
// add with c.

// Nodes do not necessarily need to appear in the cfg. Some nodes may be
// "floating", and they will be scheduled for execution when they need to be.

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use derive_more::derive::Display;
use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::{
    mir::build_lir::LirInsnBuilder,
    sim::{agent_schema::TurtleSchema, turtle::BreedId, value::NetlogoMachineType},
    util::cell::RefCell,
};

mod build_lir;
mod graphviz;
pub mod lowering;
pub mod node;

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
    pub functions: SecondaryMap<FunctionId, RefCell<Function>>,
}

#[derive(Debug)]
pub struct CustomVarDecl {
    pub name: Rc<str>,
    pub ty: NetlogoAbstractType,
}

pub type Nodes = SlotMap<NodeId, Box<dyn EffectfulNode>>;

#[derive(derive_more::Debug)]
pub struct Function {
    pub debug_name: Option<String>,
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
    pub debug_name: Option<String>,
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

#[derive(Debug)]
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

/// Some kind of computation that takes inputs and produces outputs. The output
/// of a node is immutable, though may change between instances if the node is
/// part of a loop. The output of a node can be referenced by its node id.
/// The execution of a node may have side effects; if it does, then it is
/// incorrect to deduplicate nodes; if it doesn't, deduplication is correct.
pub trait EffectfulNode: Debug + Display {
    fn has_side_effects(&self) -> bool;

    fn dependencies(&self) -> Vec<NodeId>;

    /// For certain low level nodes it doesn't make sense to have an abstract
    /// output type; those should return `None`.
    fn output_type(&self, program: &Program, function: &Function, nodes: &Nodes) -> MirType;

    /// Attempt to expand the node into a lower level representation, and
    /// performs the replacement in the nodes arena. Incoming connections to the
    /// rewritten area are preserved by reusing old `NodeId`. Returns whether
    /// any modification was performed
    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        program: &Program,
        function: &Function,
        nodes: &RefCell<Nodes>,
    ) -> bool {
        let _ = my_node_id;
        let _ = program;
        let _ = function;
        let _ = nodes;
        false
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
    ) -> Result<(), ()> {
        let _ = my_node_id;
        let _ = lir_builder;
        let _ = nodes;
        Err(())
    }
}

#[derive(Debug, Clone)]
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
#[derive(derive_more::Debug)]
pub enum NetlogoAbstractType {
    Unit,
    /// Top doesn't actually include everything
    Top,
    Bottom,
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
    Agentset,
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
    fn join(&self, other: &NetlogoAbstractType) -> NetlogoAbstractType {
        let _ = other;
        todo!()
    }

    fn meet(&self, other: &NetlogoAbstractType) -> NetlogoAbstractType {
        let _ = other;
        todo!()
    }

    fn repr(&self) -> NetlogoMachineType {
        todo!()
    }
}

impl Clone for NetlogoAbstractType {
    fn clone(&self) -> Self {
        todo!()
    }
}
