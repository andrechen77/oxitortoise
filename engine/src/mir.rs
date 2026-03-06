use std::{collections::HashMap, sync::Arc};

use crate::{
    mir::reflection::{MemDesc, Type},
    sim::value::BoxedAny,
};

pub mod builder;
pub mod reflection;

#[derive(Debug)]
pub struct HostFunctionInfo {
    pub debug_name: &'static str,
    pub parameter_types: &'static [Type],
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct FunctionId(u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct LocalId(u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Label(u32);

#[derive(Debug, Default)]
pub struct Program {
    pub functions: HashMap<FunctionId, Function>,
}

#[derive(Debug)]
pub struct Function {
    // TODO specify the parameters
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
    pub ty: MemDesc,
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
    pub label: Option<Label>,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct IfElse {
    pub condition: Place,
    pub then: Box<Statement>,
    pub r#else: Box<Statement>,
}

#[derive(Debug)]
pub struct Loop {
    pub num_repetitions: Place,
    pub body: Box<Statement>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Place {
    /// The local variable that the place is in.
    local: LocalId,
    /// The projections that are applied to the local variable to get to the
    /// place. An empty list refers to the entire local variable.
    projections: Vec<Projection>,
}

impl Place {
    pub fn proj(mut self, projection: Projection) -> Self {
        self.projections.push(projection);
        self
    }
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
    /// at the index of the given local variable. This never moves from the
    /// source place since a value used for indexing is always Copy.
    Index(LocalId),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    Break { target: Label },
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
    Const { value: BoxedAny }, // use BoxedAny because it can represent non-NetLogo types too
    /// A binary arithmetic between two scalars.
    ///
    /// Since the operands are scalars, we can directly use the LIR opcode
    /// representation without having to define yet another opcode enum.
    BinaryOp { opcode: lir::BinaryOpcode, lhs: PlaceOperand, rhs: PlaceOperand },
    /// A unary arithmetic operation on a scalar.
    ///
    /// Since the operand is a scalar, we can directly use the LIR opcode
    /// representation without having to define yet another opcode enum.
    UnaryOp { opcode: lir::UnaryOpcode, operand: PlaceOperand },
    /// Calls a user function, taking the function's arguments from the given
    /// places and producing the return value.
    CallUserFunction { function: FunctionId, args: Vec<PlaceOperand> },
    /// Calls a host function, taking the function's arguments from the given
    /// places and producing the return value.
    CallHostFunction { function: &'static HostFunctionInfo, args: Vec<PlaceOperand> },
}

impl LocalId {
    pub fn place(self) -> Place {
        self.into()
    }
}

impl From<LocalId> for Place {
    fn from(local: LocalId) -> Self {
        Place { local, projections: Vec::new() }
    }
}

impl Place {
    pub fn move_out(self) -> PlaceOperand {
        PlaceOperand::Move(self)
    }

    pub fn borrow(self) -> PlaceOperand {
        PlaceOperand::Borrow(self)
    }
}

/// Consolidates a sequence of statements into a single statement. If there is
/// only one statement, it is returned as is. If there are multiple statements,
/// a block is created with the statements.
pub fn consolidate_statements(statements: Vec<Statement>) -> Statement {
    if statements.len() == 1 {
        let mut statements = statements;
        statements.pop().expect("we checked that the length is 1")
    } else {
        Statement::CtrlFlow(CtrlFlowConstruct::Block(Block { label: None, statements }))
    }
}
