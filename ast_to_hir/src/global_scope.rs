use std::{collections::HashMap, sync::Arc};

use engine::{
    hir::{ExprKind, FunctionId, expr},
    sim::{
        patch::PatchVarDesc,
        turtle::{TurtleBreedId, TurtleVarDesc},
        value::UnpackedAny,
    },
};

pub const DEFAULT_TURTLE_BREED_NAME: &str = "TURTLES";
pub const DEFAULT_TURTLE_BREED_SINGULAR_NAME: &str = "TURTLE";

#[derive(Default)]
pub struct GlobalScope {
    pub constants: HashMap<&'static str, ExprKind>,
    pub global_vars: HashMap<Arc<str>, usize>,
    pub patch_vars: HashMap<Arc<str>, PatchVarDesc>,
    pub turtle_vars: HashMap<Arc<str>, TurtleVarDesc>,
    pub turtle_breeds: HashMap<Arc<str>, TurtleBreedId>,
    pub functions: HashMap<Arc<str>, FunctionId>,
}

impl GlobalScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_builtins(&mut self, default_turtle_breed: TurtleBreedId) {
        self.constants.extend([
            ("RED", (ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(15.0)) }))),
            ("ORANGE", ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(25.0)) })),
            ("GREEN", ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(55.0)) })),
            ("CYAN", ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(85.0)) })),
            ("SKY", ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(95.0)) })),
            ("BLUE", ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(105.0)) })),
            ("VIOLET", ExprKind::from(expr::Constant { value: Some(UnpackedAny::Float(115.0)) })),
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
