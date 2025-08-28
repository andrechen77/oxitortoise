// TODO add documentationa bout MIR. some points to include:
// - in MIR, a "local variable" doesn't necessarily need to correspond to a
// local variable in the NetLogo source code. it simply means a value that
// the function needs for later. it can refer to a temporary value as well. for
// example, when evaluating (a * b) + c, (a * b) will be computed and stored
// into a local variable, and then the variable will be used as an operand to
// add with c.

use slotmap::{SlotMap, new_key_type};

use crate::sim::{
    agent_schema::AgentFieldDescriptor,
    turtle::BreedId,
    value::{NetlogoInternalType, UnpackedDynBox},
};

mod primitives;
mod to_lir;

new_key_type! {
    pub struct FunctionId;
}

#[derive(Default)]
pub struct Program {
    pub functions: SlotMap<FunctionId, Function>,
}

pub struct Function {
    pub debug_name: Option<String>,
    /// Whether the function takes a pointer to an environment struct as a
    /// parameter. True for all closures.
    pub takes_env: bool,
    /// Whether the function takes a pointer to the current context as a
    /// parameter. True for all procedures/functions that interact with the
    /// NetLogo workspace.
    pub takes_context: bool,
    /// A list of local variables which are parameters to the function. This
    /// does not include special parameters such as the context or environment
    /// parameter.
    pub parameters: Vec<LocalId>,
    /// The local variable that will store the return value of the function.
    pub return_value: Option<LocalId>,
    /// The set of all local variables used by the function.
    pub locals: SlotMap<LocalId, LocalDeclaration>,
    pub statements: StatementBlock,
}

new_key_type! {
    pub struct LocalId;
}

pub struct LocalDeclaration {
    pub debug_name: Option<String>,
    pub mutable: bool,
    pub ty: NetlogoInternalType,
}

pub struct StatementBlock {
    pub statements: Vec<StatementKind>,
}

pub enum StatementKind {
    Op(Operation),
    IfElse { condition: Operand, then_block: StatementBlock, else_block: StatementBlock },
    Loop { block: StatementBlock },
    Stop,
}

/// The execution of a command or reporter which may create a new value. If it
/// does create a new value, the value is stored in the specified local
/// variable.
pub struct Operation {
    pub local_id: Option<LocalId>,
    pub operator: Operator,
    pub args: Vec<Operand>,
}

pub enum Operand {
    Constant(Constant),
    LocalVar(LocalId),
    GlobalVar(GlobalVar),
    ImmClosure(ImmClosure),
}

pub struct Constant {
    pub value: UnpackedDynBox,
}

pub struct ImmClosure {
    pub captures: Vec<LocalId>,
    pub body: FunctionId,
}

pub struct GlobalVar; // TODO implement

pub enum Operator {
    /// A call to another function defined in the NetLogo program.
    UserFunctionCall {
        target: FunctionId,
    },
    /// A call to a function defined by the host NetLogo engine. This is used
    /// for many NetLogo primitives which act like "standard library" functions.
    HostFunctionCall {
        // TODO add some kind of id or pointer to a data structure defining the
        // host function. the name here is just a placeholder for a true
        // identifier
        name: &'static str,
    },
    CreateTurtles {
        breed: BreedId,
    },
    /// A "call" to an operation that can be implemented directly in target code
    /// instead of having to go through a host function. This is used for
    /// very simple operations such as arithmetic and comparisons.
    DirectOperator(DirectOperator),
    SetTurtleField {
        field: AgentField,
    },
}

/// Differs from `AgentFieldDescriptor` in that while that references a memory
/// location in the row-buffer, which could be a custom field, the entire base
/// data struct, or something else entirely, this references an actual agent
/// variable that's visible to NetLogo code.
pub enum AgentField {
    Size,
    Color,
    Custom(AgentFieldDescriptor),
}

pub enum DirectOperator {
    Identity,
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
}
