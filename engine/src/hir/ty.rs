use derive_more::Display;

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
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Display)]
#[display("({}) -> {}", arg_tys.iter().map(|ty| ty.to_string()).collect::<Vec<String>>().join(", "), return_ty)]
pub struct ClosureType {
    pub arg_tys: Vec<NlAbstractTy>,
    pub return_ty: Box<NlAbstractTy>,
}
