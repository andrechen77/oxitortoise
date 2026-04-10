use std::{collections::BTreeMap, sync::Arc};

use engine::{
    hir::{ExprKind, FunctionId, expr},
    sim::{
        patch::PatchVarDesc,
        turtle::{TurtleBreedId, TurtleVarDesc},
    },
};

pub const DEFAULT_TURTLE_BREED_NAME: &str = "TURTLES";
pub const DEFAULT_TURTLE_BREED_SINGULAR_NAME: &str = "TURTLE";

#[derive(Default)]
pub struct GlobalScope {
    pub constants: BTreeMap<&'static str, ExprKind>,
    pub global_vars: BTreeMap<Arc<str>, usize>,
    pub patch_vars: BTreeMap<Arc<str>, PatchVarDesc>,
    pub turtle_vars: BTreeMap<Arc<str>, TurtleVarDesc>,
    pub turtle_breeds: BTreeMap<Arc<str>, TurtleBreedId>,
    pub functions: BTreeMap<Arc<str>, FunctionId>,
}

impl GlobalScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_builtins(&mut self, default_turtle_breed: TurtleBreedId) {
        self.constants.extend([
            ("RED", (ExprKind::from(expr::NumberLiteral { value: 15.0 }))),
            ("ORANGE", ExprKind::from(expr::NumberLiteral { value: 25.0 })),
            ("GREEN", ExprKind::from(expr::NumberLiteral { value: 55.0 })),
            ("CYAN", ExprKind::from(expr::NumberLiteral { value: 85.0 })),
            ("SKY", ExprKind::from(expr::NumberLiteral { value: 95.0 })),
            ("BLUE", ExprKind::from(expr::NumberLiteral { value: 105.0 })),
            ("VIOLET", ExprKind::from(expr::NumberLiteral { value: 115.0 })),
        ]);
        self.patch_vars.extend([(Arc::from("PCOLOR"), PatchVarDesc::Pcolor)]);
        self.turtle_vars.extend([
            (Arc::from("WHO"), TurtleVarDesc::Who),
            (Arc::from("COLOR"), TurtleVarDesc::Color),
            (Arc::from("SIZE"), TurtleVarDesc::Size),
        ]);
        self.turtle_breeds.extend([(DEFAULT_TURTLE_BREED_NAME.into(), default_turtle_breed)]);
    }
}
