use std::{collections::BTreeMap, fmt, sync::Arc};

use crate::{sim::value::BoxedAny, util::reflection::Type};

mod builder;
mod reflection;

pub use builder::{FunctionBuilder, FunctionStub, ProgramBuilder};
use derive_more::Debug;
pub use reflection::{
    DynPtr, DynPtrMut, HasDynPtr, MirReflect, MirType, MirTypeArray, MirTypeStruct,
};

#[derive(Debug)]
#[debug("{debug_name}")]
pub struct HostFunctionInfo {
    pub debug_name: &'static str,
    pub parameter_types: &'static [Type],
    pub return_type: Type,
    /// Meaningful on Wasm targets. The function is exported by this name.
    pub link_name: &'static str,
    /// Meaningfun on native targets. The function is located at this address.
    pub link_addr: *const u8,
}

// SAFETY: the only thing that prevents the struct from being Sync is the raw
// pointer, which is just a function pointer so it is safe to share.
unsafe impl Sync for HostFunctionInfo {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[debug("F{_0}")]
pub struct FunctionId(u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[debug("V{_0}")]
pub struct LocalId(u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[debug("L{_0}")]
pub struct Label(u32);

#[derive(Debug, Default)]
pub struct Program {
    pub functions: BTreeMap<FunctionId, Function>,
}

#[derive(Debug)]
pub struct Function {
    /// The parameters to the function.
    pub parameters: Vec<LocalId>,
    /// Holds every local variable used in the function body, including
    /// parameters, temporaries, and return values.
    pub local_decls: BTreeMap<LocalId, LocalDecl>,
    /// The local variable where the function's return value is stored.
    pub return_local: LocalId,
    /// The body of the function.
    pub body: Statement,
}

#[derive(Debug)]
pub struct LocalDecl {
    pub debug_name: Option<Arc<str>>,
    pub ty: MirType,
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
    pub label: Label,
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

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Place {
    /// The local variable that the place is in.
    pub local: LocalId,
    /// The projections that are applied to the local variable to get to the
    /// place. An empty list refers to the entire local variable.
    pub projections: Vec<Projection>,
}

impl Place {
    pub fn proj(mut self, projection: Projection) -> Self {
        self.projections.push(projection);
        self
    }

    pub fn proj_deref(self) -> Self {
        self.proj(Projection::Deref)
    }

    pub fn proj_field(self, byte_offset: usize) -> Self {
        self.proj(Projection::Field { byte_offset })
    }

    pub fn proj_dynamic_index(self, index: LocalId) -> Self {
        self.proj(Projection::DynamicIndex(index))
    }

    pub fn proj_static_index(self, index: usize) -> Self {
        self.proj(Projection::StaticIndex(index))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
    DynamicIndex(LocalId),
    /// With the place having an array value, produces the place of the element
    /// at the given index.
    StaticIndex(usize),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PlaceOperand {
    /// Produces an operand by moving out of the place. This is a destructive
    /// move, and the source place is considered deinitialized afterward.
    ///
    /// To simplify things, it is only possible to move out of the entire local
    /// variable, not just a subplace.
    Move(LocalId),
    /// Produces an operand by copying the place. The type must be Copy.
    Copy(Place),
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
    /// Lowers to the address of the specified function.
    FunctionPtr { function: FunctionId },
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
    pub fn unwrap_local(&self) -> LocalId {
        assert!(self.projections.is_empty(), "place must not have projections");
        self.local
    }
}

impl Function {
    fn return_ty(&self) -> &MirType {
        &self.local_decls[&self.return_local].ty
    }
}

impl Operation {
    pub fn operands(&self) -> Vec<&PlaceOperand> {
        match self {
            Operation::Operand(opd) => vec![opd],
            Operation::Const { .. } => vec![],
            Operation::FunctionPtr { .. } => vec![],
            Operation::BinaryOp { lhs, rhs, .. } => vec![lhs, rhs],
            Operation::UnaryOp { operand, .. } => vec![operand],
            Operation::CallUserFunction { args, .. } => args.iter().collect(),
            Operation::CallHostFunction { args, .. } => args.iter().collect(),
        }
    }
}

/// Consolidates a sequence of statements into a single statement. If there is
/// only one statement, it is returned as is. If there are multiple statements,
/// a block is created with the statements.
pub fn consolidate_statements(
    statements: Vec<Statement>,
    generate_label: impl FnOnce() -> Label,
) -> Statement {
    if statements.len() == 1 {
        let mut statements = statements;
        statements.pop().expect("we checked that the length is 1")
    } else {
        let label = generate_label();
        Statement::CtrlFlow(CtrlFlowConstruct::Block(Block { label, statements }))
    }
}

impl fmt::Debug for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Place { local, projections } = self;
        write!(f, "{local:?}")?;
        for projection in projections {
            match projection {
                Projection::Deref => write!(f, ".deref")?,
                Projection::Field { byte_offset } => write!(f, ".({byte_offset})")?,
                Projection::DynamicIndex(index) => write!(f, ".[{index:?}]")?,
                Projection::StaticIndex(index) => write!(f, ".[{index}]")?,
            }
        }
        Ok(())
    }
}
