//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use std::{alloc::Layout, fmt, sync::Arc};

use derive_more::derive::TryFrom;
use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, NameContext, NlAbstractTy, build_mir::HirToMirFnBuilder},
    mir,
    sim::{
        patch::OptionPatchId,
        value::{BoxedAny, NlFloat, PackedAny},
    },
    util::reflection::{CloneKind, Reflect, Type, TypeInfo},
};

#[derive(Debug, Clone, Copy, TryFrom, PartialEq, Eq)]
#[try_from(repr)]
#[repr(u8)]
pub enum BinaryArithOpcode {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, TryFrom, PartialEq, Eq)]
#[try_from(repr)]
#[repr(u8)]
pub enum BinaryCmpOpcode {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
}

#[derive(Debug, Clone, Copy, TryFrom, PartialEq, Eq)]
#[try_from(repr)]
#[repr(u8)]
pub enum BinaryBoolOpcode {
    And,
    Or,
}

static BINARY_ARITH_OPCODE_TYPE_INFO: TypeInfo = TypeInfo {
    debug_name: "BinaryArithOpcode",
    layout: Some(Layout::new::<BinaryArithOpcode>()),
    is_zeroable: false,
    clone: CloneKind::Copy,
    drop_fn: None,
    make_mir_type: || {
        Arc::new(mir::MirTypeInfo {
            static_ty: Some(&BINARY_ARITH_OPCODE_TYPE_INFO),
            contents: mir::MirTypeContents::IsPrimitive(lir::ValType::I8),
        })
    },
};

unsafe impl Reflect for BinaryArithOpcode {
    const TYPE: Type = &BINARY_ARITH_OPCODE_TYPE_INFO;
}

static BINARY_CMP_OPCODE_TYPE_INFO: TypeInfo = TypeInfo {
    debug_name: "BinaryCmpOpcode",
    layout: Some(Layout::new::<BinaryCmpOpcode>()),
    is_zeroable: false,
    clone: CloneKind::Copy,
    drop_fn: None,
    make_mir_type: || {
        Arc::new(mir::MirTypeInfo {
            static_ty: Some(&BINARY_CMP_OPCODE_TYPE_INFO),
            contents: mir::MirTypeContents::IsPrimitive(lir::ValType::I8),
        })
    },
};

unsafe impl Reflect for BinaryCmpOpcode {
    const TYPE: Type = &BINARY_CMP_OPCODE_TYPE_INFO;
}

static BINARY_BOOL_OPCODE_TYPE_INFO: TypeInfo = TypeInfo {
    debug_name: "BinaryBoolOpcode",
    layout: Some(Layout::new::<BinaryBoolOpcode>()),
    is_zeroable: false,
    clone: CloneKind::Copy,
    drop_fn: None,
    make_mir_type: || {
        Arc::new(mir::MirTypeInfo {
            static_ty: Some(&BINARY_BOOL_OPCODE_TYPE_INFO),
            contents: mir::MirTypeContents::IsPrimitive(lir::ValType::I8),
        })
    },
};

unsafe impl Reflect for BinaryBoolOpcode {
    const TYPE: Type = &BINARY_BOOL_OPCODE_TYPE_INFO;
}

#[derive(Debug, Clone)]
pub struct BinaryArith {
    /// The context to use for the operation. This is necessary for certain
    /// operations such as checking for equality with nobody.
    pub op: BinaryArithOpcode,
    pub lhs: Box<ExprKind>,
    pub rhs: Box<ExprKind>,
}

impl Expr for BinaryArith {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.lhs);
        visitor(&self.rhs);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.lhs.as_mut());
        visitor(self.rhs.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp)");
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let BinaryArith { op, lhs, rhs } = self;
        p.add_fn_call("binary_arith", |p| {
            p.add_fn_arg(*op)?;
            p.add_fn_arg_with(|p| lhs.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rhs.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct BinaryCmp {
    /// The context to use for the operation. This is necessary for certain
    /// operations such as checking for equality with nobody.
    pub op: BinaryCmpOpcode,
    pub lhs: Box<ExprKind>,
    pub rhs: Box<ExprKind>,
}

impl Expr for BinaryCmp {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Boolean
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.lhs);
        visitor(&self.rhs);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.lhs.as_mut());
        visitor(self.rhs.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        let lhs_ty = self.lhs.output_type(builder.hir_names);
        let rhs_ty = self.rhs.output_type(builder.hir_names);

        // special case comparisons against nobody (both as the sole
        // inhabitant of the nobody type and as the inhabitant of agent id types).
        if (lhs_ty == NlAbstractTy::Nobody || rhs_ty == NlAbstractTy::Nobody)
            && (self.op == BinaryCmpOpcode::Eq || self.op == BinaryCmpOpcode::Neq)
        {
            let negate = match self.op {
                BinaryCmpOpcode::Eq => false,
                BinaryCmpOpcode::Neq => true,
                _ => unreachable!(),
            };

            // short circuit on nobody vs nobody comparison
            if lhs_ty == NlAbstractTy::Nobody && rhs_ty == NlAbstractTy::Nobody {
                let result = !negate;
                builder.mir.add_operation_with_dst(
                    local_out.into(),
                    mir::Operation::Const { value: BoxedAny::new(result) },
                );
                return;
            }

            // find the operand that is not statically known to be nobody
            let operand = if lhs_ty == NlAbstractTy::Nobody { &self.rhs } else { &self.lhs };
            let operand_pl = builder.translate_expr(operand);
            let operand_ty = builder.mir.type_of_place(&operand_pl.place);
            if operand_ty.is::<OptionPatchId>() {
                OptionPatchId::write_check_nobody(builder, negate, local_out, operand_pl);
            } else {
                todo!("TODO(mvp) handle nobody check for other operand types: {:?}", operand_ty);
            }

            return;
        }

        let lhs_pl = builder.translate_expr(&self.lhs);
        let lhs_ty = builder.mir.type_of_place(&lhs_pl.place);
        let rhs_pl = builder.translate_expr(&self.rhs);
        let rhs_ty = builder.mir.type_of_place(&rhs_pl.place);

        use BinaryCmpOpcode as Op;

        // match on known combinations of input types and opcodes
        // TODO(mvp) so far, these additional conditions on color only exist to
        // get ants to compile. we will want to make a full decision on how to
        // treat colors in the engine later
        let final_operation = if (lhs_ty.is::<NlFloat>() || rhs_ty.is::<NlFloat>())
            && (rhs_ty.is::<NlFloat>() || rhs_ty.is::<NlFloat>())
        {
            let opcode = match self.op {
                Op::Lt => lir::BinaryOpcode::FLt,
                Op::Lte => lir::BinaryOpcode::FLte,
                Op::Gt => lir::BinaryOpcode::FGt,
                Op::Gte => lir::BinaryOpcode::FGte,
                Op::Eq => lir::BinaryOpcode::FEq,
                _ => unimplemented!("unsupported operation"),
            };
            mir::Operation::BinaryOp {
                opcode,
                lhs: mir::PlaceOperand::Move(lhs_pl.place),
                rhs: mir::PlaceOperand::Move(rhs_pl.place),
            }
        } else if lhs_ty.is::<PackedAny>() && rhs_ty.is::<PackedAny>() {
            let opcode_pl = builder.mir.add_operation(
                None,
                mir::Operation::Const { value: BoxedAny::new::<BinaryCmpOpcode>(self.op) },
            );
            mir::Operation::CallHostFunction {
                function: &binary_cmp_any_bool::FN_INFO,
                args: vec![
                    mir::PlaceOperand::Move(lhs_pl.place),
                    mir::PlaceOperand::Move(rhs_pl.place),
                    mir::PlaceOperand::Move(opcode_pl.place()),
                ],
            }
        } else {
            todo!("TODO(mvp) handle other operand types: {:?} and {:?}", lhs_ty, rhs_ty);
        };
        builder.mir.add_operation_with_dst(local_out.into(), final_operation);
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let BinaryCmp { op, lhs, rhs } = self;
        p.add_fn_call("binary_cmp", |p| {
            p.add_fn_arg(*op)?;
            p.add_fn_arg_with(|p| lhs.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rhs.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[allow(dead_code)]
fn binary_cmp_any_bool(_lhs: PackedAny, _rhs: PackedAny, _op: BinaryCmpOpcode) -> bool {
    todo!("TODO(mvp)");
}
mod binary_cmp_any_bool {
    use super::*;
    use crate::mir::HostFunctionInfo;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "binary_cmp_any_bool",
        parameter_types: &[PackedAny::TYPE, PackedAny::TYPE, BinaryCmpOpcode::TYPE],
        return_type: bool::TYPE,
    };
}

#[derive(Debug, Clone)]
pub struct BinaryBool {
    pub op: BinaryBoolOpcode,
    pub lhs: Box<ExprKind>,
    pub rhs: Box<ExprKind>,
}

impl Expr for BinaryBool {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Boolean
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.lhs);
        visitor(&self.rhs);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.lhs.as_mut());
        visitor(self.rhs.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp)");
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let BinaryBool { op, lhs, rhs } = self;
        p.add_fn_call("binary_bool", |p| {
            p.add_fn_arg(*op)?;
            p.add_fn_arg_with(|p| lhs.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rhs.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct LogicalNot {
    pub operand: Box<ExprKind>,
}

impl Expr for LogicalNot {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Boolean
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.operand);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.operand.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        let operand_pl = builder.translate_expr(&self.operand);
        let final_operation = mir::Operation::UnaryOp {
            opcode: lir::UnaryOpcode::Not,
            operand: mir::PlaceOperand::Move(operand_pl.place),
        };
        builder.mir.add_operation_with_dst(local_out.into(), final_operation);
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let LogicalNot { operand } = self;
        p.add_fn_call("logical_not", |p| {
            p.add_fn_arg_with(|p| operand.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct Negate {
    pub operand: Box<ExprKind>,
}

impl Expr for Negate {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.operand);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.operand.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        let operand_pl = builder.translate_expr(&self.operand);
        let final_operation = mir::Operation::UnaryOp {
            opcode: lir::UnaryOpcode::FNeg,
            operand: mir::PlaceOperand::Move(operand_pl.place),
        };
        builder.mir.add_operation_with_dst(local_out.into(), final_operation);
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Negate { operand } = self;
        p.add_fn_call("negate", |p| {
            p.add_fn_arg_with(|p| operand.pretty_print(p, names))?;
            Ok(())
        })
    }
}
