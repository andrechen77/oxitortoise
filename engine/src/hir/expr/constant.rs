//! Nodes that represent constant/literal values.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, build_mir::translate_expr,
    },
    mir,
    sim::value::{NlBox, NlFloat, NlList, NlString, PackedAny},
    util::reflection::Reflect,
};

#[derive(Debug, Clone)]
pub struct UnitLiteral;

impl Expr for UnitLiteral {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children<'a>(&'a self, _visitor: impl FnMut(&'a ExprKind)) {
        // no children
    }

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {}

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        _names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("unit_literal", |_p| Ok(()))
    }
}

impl UnitLiteral {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> mir::LocalId {
        builder.mir.unit_local()
    }
}

#[derive(Debug, Clone)]
pub struct NumberLiteral {
    pub value: f64,
}

impl Expr for NumberLiteral {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children<'a>(&'a self, _visitor: impl FnMut(&'a ExprKind)) {}

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {}

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        _names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("number_literal", |p| {
            p.add_fn_arg_with(|p| write!(p, "{:?}", self.value))?;
            Ok(())
        })
    }
}

impl NumberLiteral {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        Some(builder.mir.add_operation(
            None,
            mir::Operation::Const {
                value: crate::sim::value::BoxedAny::new(NlFloat::new(self.value)),
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: NlString,
}

impl Expr for StringLiteral {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::String
    }

    fn visit_children<'a>(&'a self, _visitor: impl FnMut(&'a ExprKind)) {}

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {}

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        _names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("string_literal", |p| {
            p.add_fn_arg_with(|p| write!(p, "{:?}", self.value))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct NobodyLiteral;

impl Expr for NobodyLiteral {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Nobody
    }

    fn visit_children<'a>(&'a self, _visitor: impl FnMut(&'a ExprKind)) {}

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {}

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        _names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("nobody_literal", |_p| Ok(()))
    }
}

#[derive(Debug, Clone)]
pub struct ListLiteral {
    pub items: Vec<Box<ExprKind>>,
}

impl Expr for ListLiteral {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let mut ty = NlAbstractTy::Bottom;
        for item in &self.items {
            ty = ty.join(item.output_type(names));
        }
        NlAbstractTy::List { element_ty: Box::new(ty) }
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        for item in &self.items {
            visitor(item);
        }
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        for item in &mut self.items {
            visitor(item.as_mut());
        }
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("list_literal", |p| {
            for item in &self.items {
                p.add_fn_arg_with(|p| item.pretty_print(p, names))?;
            }
            Ok(())
        })
    }
}

impl ListLiteral {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let list = builder.mir.add_operation(
            None,
            mir::Operation::CallHostFunction { function: &list_new::FN_INFO, args: vec![] },
        );

        // push each item onto the list
        for item in &self.items {
            let item = translate_expr(builder, item)?;
            builder.mir.add_operation_with_dst(
                list.place(),
                mir::Operation::CallHostFunction {
                    function: &list_push::FN_INFO,
                    args: vec![mir::PlaceOperand::Move(list), mir::PlaceOperand::Move(item)],
                },
            );
        }

        Some(list)
    }
}

mod list_new {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "list_new",
        parameter_types: &[],
        return_type: NlBox::<NlList>::TYPE,
        link_name: "list_new",
        link_addr: call as *const u8,
    };

    pub fn call() -> NlBox<NlList> {
        NlBox::new(NlList::new())
    }
}

mod list_push {
    use crate::mir::HostFunctionInfo;

    use super::*;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "list_push",
        parameter_types: &[NlBox::<NlList>::TYPE, PackedAny::TYPE],
        return_type: NlBox::<NlList>::TYPE,
        link_name: "list_push",
        link_addr: call as *const u8,
    };

    pub fn call(mut list: NlBox<NlList>, element: PackedAny) -> NlBox<NlList> {
        list.push(element);
        list
    }
}
