use std::{collections::HashMap, sync::Arc};

use crate::{sim::value::UnpackedAny, util::reflection::ConcreteTy};

pub mod from_hir;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct FunctionId(u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct LocalId(u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ControlFlowConstructId(u32);

#[derive(Debug, Default)]
pub struct Program {
    pub functions: HashMap<FunctionId, Function>,
}

#[derive(Debug)]
pub struct Function {
    /// Holds every local variable used in the function body, including
    /// parameters, temporaries, and return values.
    pub local_decls: HashMap<LocalId, LocalDecl>,
    /// The local variable where the function's return value is stored.
    pub return_local: LocalId,
    /// The body of the function.
    pub body: Statement,
}

#[derive(Debug)]
pub struct LocalDecl {
    pub debug_name: Option<Arc<str>>,
    pub ty: ConcreteTy,
}

#[derive(Debug)]
pub enum Statement {
    CtrlFlow(CtrlFlowConstruct),
    Elementary(ElementaryStatement),
}

#[derive(Debug)]
pub enum CtrlFlowConstruct {
    Block(Block),
    IfElse(IfElse),
    Loop(Loop),
}

// TODO maybe add a structure called a "scope" which is sequence of statements
// as well as a set of locals that are statically only valid in that scope

#[derive(Debug)]
pub struct Block {
    pub id: ControlFlowConstructId,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct IfElse {
    pub id: ControlFlowConstructId,
    pub condition: Place,
    pub then_block: Vec<Statement>,
    pub else_block: Vec<Statement>,
}

#[derive(Debug)]
pub struct Loop {
    pub id: ControlFlowConstructId,
    pub num_repetitions: Place,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Place {
    /// The local variable that the place is in.
    local: LocalId,
    /// The projections that are applied to the local variable to get to the
    /// place. An empty list refers to the entire local variable.
    projections: Vec<Projection>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Projection {
    /// With the place having a pointer value, dereferences the pointer and
    /// produces the place of the dereferenced value.
    Deref,
    /// With the place having a struct value, produces the place of the field
    /// at the given byte offset.
    Field { byte_offset: usize },
    /// With the place having an array value, produces the place of the element
    /// at the index of the given local variable.
    Index(LocalId),
}

#[derive(Debug)]
pub enum PlaceOperand {
    /// Produces an operand by moving out of the place. This is a destructive
    /// move unless the type is Copy.
    Move(Place),
    /// Produces an operand by creating a reference to the place.
    Borrow(Place),
}

#[derive(Debug)]
pub enum ElementaryStatement {
    /// Breaks out the specified control flow construct. Not yet sure what
    /// breaking from a loop should do.
    Break { target: ControlFlowConstructId },
    /// Drops the value in the source place, making it dead.
    Drop { src: Place },
    /// Performs some operation and assigns the result to the destination place.
    Assign { dst: Place, op: Operation },
}

#[derive(Debug)]
pub enum Operation {
    /// Directly produces the operand.
    Operand(PlaceOperand),
    /// Produces a new instance of the specified value.
    Const { value: UnpackedAny },
    /// Calls a user function, taking the function's arguments from the given
    /// places and producing the return value.
    CallUserFunction { function: FunctionId, args: Vec<PlaceOperand> },
    /// Calls a host function, taking the function's arguments from the given
    /// places and producing the return value.
    CallHostFunction { /* TODO refer to the host function */ args: Vec<PlaceOperand> },
}

impl From<LocalId> for Place {
    fn from(local: LocalId) -> Self {
        Place { local, projections: Vec::new() }
    }
}
