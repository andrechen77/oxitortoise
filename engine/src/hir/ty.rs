use std::{fmt, sync::Arc};

use derive_more::Debug;
use smallvec::{SmallVec, smallvec};

/// A representation of an element of the lattice making up all NetLogo types.
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum NlAbstractTy {
    // An independent type used to model a reference to the workspace itself.
    Workspace,
    // An independent type used to model a reference to the random number
    // generator itself.
    Rng,
    #[debug("{_0:?}")]
    Union(Union),
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct Union {
    /// None if literally any type is possible.
    variants: Option<SmallVec<[NlAbstractTyAtom; 1]>>,
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum NlAbstractTyAtom {
    Unit,
    Float,
    Boolean,
    String,
    Point,
    Patch,
    Turtle,
    Link,
    Agentset { agent_type: Arc<NlAbstractTy> },
    Nobody,
    Closure(ClosureType),
    List { element_ty: Arc<NlAbstractTy> },
}

impl Default for NlAbstractTy {
    /// The bottom type
    fn default() -> Self {
        Self::bottom()
    }
}

impl From<NlAbstractTyAtom> for NlAbstractTy {
    fn from(atom: NlAbstractTyAtom) -> Self {
        Self::Union(Union { variants: Some(smallvec![atom]) })
    }
}

impl FromIterator<NlAbstractTyAtom> for NlAbstractTy {
    fn from_iter<T: IntoIterator<Item = NlAbstractTyAtom>>(iter: T) -> Self {
        Self::Union(Union { variants: Some(iter.into_iter().collect()) })
    }
}

impl NlAbstractTy {
    pub fn top() -> Self {
        Self::Union(Union { variants: None })
    }

    pub fn bottom() -> Self {
        Self::Union(Union { variants: Some(smallvec![]) })
    }

    pub fn is_only(&self, atom: &NlAbstractTyAtom) -> bool {
        match self {
            NlAbstractTy::Union(un) => un.is_only(atom),
            _ => false,
        }
    }

    pub fn get_atom(&self) -> Option<&NlAbstractTyAtom> {
        match self {
            NlAbstractTy::Union(un) => un.get_atom(),
            _ => None,
        }
    }

    pub fn get_union(&self) -> Option<&Union> {
        if let NlAbstractTy::Union(un) = self { Some(un) } else { None }
    }

    /// Calculates the least upper bound of two types. Returns whether any
    /// modification was made.
    pub fn join(&mut self, other: NlAbstractTy) -> bool {
        match (&mut *self, other) {
            (NlAbstractTy::Union(self_un), NlAbstractTy::Union(other_un)) => self_un.join(other_un),
            (NlAbstractTy::Union(self_un), other) if self_un.is_empty() => {
                *self = other;
                true
            }
            (_, NlAbstractTy::Union(other_un)) if other_un.is_empty() => false,
            (NlAbstractTy::Rng, NlAbstractTy::Rng) => false,
            (NlAbstractTy::Workspace, NlAbstractTy::Workspace) => false,
            (s, o) => panic!("Cannot join {:?} with {:?}", s, o),
        }
    }

    /// Calculates the greatest lower bound of two types. Returns whether any
    /// modification was made.
    pub fn meet(&mut self, other: NlAbstractTy) -> bool {
        match (self, other) {
            (NlAbstractTy::Union(self_un), NlAbstractTy::Union(other_un)) => self_un.meet(other_un),
            (NlAbstractTy::Workspace, NlAbstractTy::Workspace) => {
                // already is the meet
                false
            }
            (NlAbstractTy::Rng, NlAbstractTy::Rng) => {
                // already is the meet
                false
            }
            (s, o) => panic!("Cannot meet {:?} with {:?}", s, o),
        }
    }
}

impl Union {
    /// Calculates the least upper bound of two unions. Returns whether any
    /// modification was made.
    pub fn join(&mut self, other: Union) -> bool {
        match (&mut self.variants, other.variants) {
            (Some(self_variants), Some(other_variants)) => {
                let mut changed = false;
                for variant in other_variants {
                    if !self_variants.contains(&variant) {
                        self_variants.push(variant);
                        changed = true;
                    }
                }
                changed
            }
            (Some(_), None) => {
                self.variants = None;
                true
            }
            (None, _) => false,
        }
    }

    /// Calculates the greatest lower bound of two unions. Returns whether any
    /// modification was made.
    pub fn meet(&mut self, other: Union) -> bool {
        match (&mut self.variants, other.variants) {
            (Some(self_variants), Some(other_variants)) => {
                let mut changed = false;
                self_variants.retain(|variant| {
                    let keep = other_variants.contains(variant);
                    if !keep {
                        changed = true;
                    }
                    keep
                });
                changed
            }
            (None, Some(other_variants)) => {
                self.variants = Some(other_variants);
                true
            }
            (_, None) => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        let Some(variants) = &self.variants else {
            return false;
        };
        variants.is_empty()
    }

    pub fn is_only(&self, atom: &NlAbstractTyAtom) -> bool {
        let Some(variants) = &self.variants else {
            return false;
        };
        variants.len() == 1 && variants[0] == *atom
    }

    pub fn is_only_2(&self, atom_a: &NlAbstractTyAtom, atom_b: &NlAbstractTyAtom) -> bool {
        let Some(variants) = &self.variants else {
            return false;
        };
        variants.len() == 2
            && (variants[0] == *atom_a || variants[0] == *atom_b)
            && (variants[1] == *atom_a || variants[1] == *atom_b)
    }

    pub fn get_atom(&self) -> Option<&NlAbstractTyAtom> {
        let variants = self.variants.as_ref()?;
        (variants.len() == 1).then(|| &variants[0])
    }

    pub fn get_closure(&self) -> Option<&ClosureType> {
        let Some(variants) = &self.variants else {
            return None;
        };
        if !variants.len() == 1 {
            return None;
        }
        let NlAbstractTyAtom::Closure(closure) = &variants[0] else {
            return None;
        };
        Some(closure)
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct ClosureType {
    pub arg_tys: Vec<NlAbstractTy>,
    pub return_ty: Arc<NlAbstractTy>,
}

impl fmt::Debug for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(variants) = &self.variants {
            if variants.is_empty() {
                write!(f, "bottom")
            } else {
                for (i, ty) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{:?}", ty)?;
                }
                Ok(())
            }
        } else {
            write!(f, "top")
        }
    }
}

impl fmt::Debug for ClosureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, ty) in self.arg_tys.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", ty)?;
        }
        write!(f, ") -> {:?}", self.return_ty)
    }
}
