//! Nodes for commands/reporters that interact with colors.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, build_mir::translate_expr},
    mir,
    sim::{color, value::NlFloat},
    util::reflection::Reflect,
};

/// NetLogo `scale-color`.
#[derive(Debug, Clone)]
pub struct ScaleColor {
    pub color: Box<ExprKind>,
    pub number: Box<ExprKind>,
    pub range1: Box<ExprKind>,
    pub range2: Box<ExprKind>,
}

impl Expr for ScaleColor {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Color
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.color);
        visitor(&self.number);
        visitor(&self.range1);
        visitor(&self.range2);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.color.as_mut());
        visitor(self.number.as_mut());
        visitor(self.range1.as_mut());
        visitor(self.range2.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let ScaleColor { color, number, range1, range2 } = self;
        p.add_fn_call("scale_color", |p| {
            p.add_fn_arg_with(|p| color.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| number.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| range1.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| range2.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl ScaleColor {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Self { color, number, range1, range2 } = self;
        let color = translate_expr(builder, color)?;
        let number = translate_expr(builder, number)?;
        let range1 = translate_expr(builder, range1)?;
        let range2 = translate_expr(builder, range2)?;

        let operation = mir::Operation::CallHostFunction {
            function: &scale_color::FN_INFO,
            args: vec![
                mir::PlaceOperand::Move(color),
                mir::PlaceOperand::Move(number),
                mir::PlaceOperand::Move(range1),
                mir::PlaceOperand::Move(range2),
            ],
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod scale_color {
    use crate::{mir::HostFunctionInfo, sim::color::Color};

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "scale_color",
        parameter_types: &[Color::TYPE, NlFloat::TYPE, NlFloat::TYPE, NlFloat::TYPE],
        return_type: Color::TYPE,
        link_name: "scale_color",
        link_addr: call as *const u8,
    };

    pub fn call(
        color: Color,
        number: NlFloat,
        range_start: NlFloat,
        range_end: NlFloat,
    ) -> Color {
        color::scale_color(color, number, range_start, range_end)
    }
}
