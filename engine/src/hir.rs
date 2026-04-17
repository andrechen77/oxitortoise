// TODO(doc) all of HIR

use std::{collections::BTreeMap, sync::Arc};

use derive_more::derive::Debug;

use crate::sim::turtle::{TurtleBreed, TurtleBreedId};

mod build_mir;
pub mod expr;
mod format;
mod ty;
mod type_inference;

pub use expr::{Expr, ExprKind};
pub use ty::{ClosureType, NlAbstractTy, NlAbstractTyAtom};

pub use build_mir::{HirToMirFnBuilder, TypeMapping, hir_to_mir, make_type_mapping};
pub use type_inference::narrow_types;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[debug("F{_0}")]
pub struct FunctionId(pub u32);

#[derive(Debug)]
pub struct Program {
    pub global_vars: Box<[CustomVarDecl]>,
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
    pub debug_name: Arc<str>,
    pub ty: NlAbstractTy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[debug("L{_0}")]
pub struct Label(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[debug("V{_0}")]
pub struct LocalId(pub u32);

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
