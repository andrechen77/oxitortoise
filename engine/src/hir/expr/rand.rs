//! Nodes for commands/reporters that interact with the RNG.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, build_mir::translate_expr,
    },
    mir,
    sim::value::NlFloat,
    util::rng::{CanonRng, Rng as _},
};

#[derive(Debug, Clone)]
pub struct RandomInt {
    pub rng: Box<ExprKind>,
    pub bound: Box<ExprKind>,
}

impl Expr for RandomInt {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        // Despite the name, the current HIR uses `Float` as the abstract output type.
        NlAbstractTy::Float
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.rng);
        visitor(&self.bound);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.rng.as_mut());
        visitor(self.bound.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let RandomInt { rng, bound } = self;
        p.add_fn_call("random_int", |p| {
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| bound.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl RandomInt {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let rng = translate_expr(builder, &self.rng)?;
        let bound = translate_expr(builder, &self.bound)?;

        let operation = mir::Operation::CallHostFunction {
            function: &random_int::FN_INFO,
            args: vec![mir::PlaceOperand::Move(rng), mir::PlaceOperand::Move(bound)],
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod random_int {
    use crate::{mir::HostFunctionInfo, util::reflection::Reflect};

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "random_int",
        parameter_types: &[<&mut CanonRng>::TYPE, NlFloat::TYPE],
        return_type: NlFloat::TYPE,
        link_addr: call as *const u8,
        link_name: "random_int",
    };

    pub fn call(rng: &mut CanonRng, max: f64) -> f64 {
        rng.next_int(max as i64) as f64
    }
}
