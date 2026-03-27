use derive_more::Display;

use crate::mir;

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
