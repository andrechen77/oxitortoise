//! Nodes for primitives that operate on lists and agentsets.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, NlAbstractTyAtom,
        build_mir::translate_expr,
    },
    sim::value::{NlBox, NlList, PackedAny},
    util::rng::{CanonRng, Rng as _},
};
use reflection::{Reflect, mir};

#[derive(Debug, Clone)]
pub struct OneOf {
    pub rng: Box<ExprKind>,
    pub operand: Box<ExprKind>,
}

impl Expr for OneOf {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let operand_ty = self.operand.output_type(names);
        match operand_ty.get_atom() {
            Some(NlAbstractTyAtom::Agentset { agent_type }) => agent_type.as_ref().clone(),
            Some(NlAbstractTyAtom::List { element_ty }) => element_ty.as_ref().clone(),
            None => todo!("TODO(mvp) OneOf unsupported operand type: {:?}", operand_ty),
            x => todo!("TODO(mvp) OneOf unsupported operand type: {:?}", x),
        }
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.rng);
        visitor(&self.operand);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.rng.as_mut());
        visitor(self.operand.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let OneOf { rng, operand } = self;
        p.add_fn_call("one_of", |p| {
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| operand.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl OneOf {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let OneOf { rng, operand } = self;
        let rng = translate_expr(builder, rng)?;
        let operand = translate_expr(builder, operand)?;

        let output_type = self.operand.output_type(builder.hir_names);
        let operation = match output_type.get_atom() {
            Some(NlAbstractTyAtom::List { .. }) => mir::Operation::CallHostFunction {
                function: &one_of_list::FN_INFO,
                args: vec![
                    mir::PlaceOperand::Direct(rng.place()),
                    mir::PlaceOperand::Direct(operand.place()),
                ],
            },
            _ => todo!("TODO(mvp) OneOf unsupported operand type: {:?}", output_type),
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod one_of_list {
    use reflection::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "one_of_list",
        parameter_types: &[<&mut CanonRng>::STATIC_TYPE, NlBox::<NlList>::STATIC_TYPE],
        return_type: PackedAny::STATIC_TYPE,
        link_name: "one_of_list",
        link_addr: call as *const u8,
    };

    pub fn call(rng: &mut CanonRng, mut list: NlBox<NlList>) -> PackedAny {
        let index = rng.next_int(list.len() as i64) as usize; // TODO casts okay?
        list.swap_remove(index)
    }
}
