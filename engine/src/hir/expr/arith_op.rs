//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use std::{alloc::Layout, sync::Arc};

use derive_more::derive::TryFrom;

use crate::{
    hir::{Expr, ExprKind, NlAbstractTy, Program, build_mir::HirToMirFnBuilder},
    mir::{self, prelude::*},
    sim::{
        patch::OptionPatchId,
        value::{BoxedAny, NlFloat, PackedAny},
    },
    util::reflection::{CloneKind, Reflect, Type, TypeInfo},
};

#[derive(Debug, Clone, Copy, TryFrom, PartialEq, Eq)]
#[try_from(repr)]
#[repr(u8)]
pub enum BinaryOpcode {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    And,
    Or,
}

static BINARY_OPCODE_TYPE_INFO: TypeInfo = TypeInfo {
    debug_name: "BinaryOpcode",
    layout: Some(Layout::new::<BinaryOpcode>()),
    is_zeroable: false,
    clone: CloneKind::Copy,
    drop_fn: None,
    make_mir_type: || {
        Arc::new(MirTypeInfo {
            static_ty: Some(&BINARY_OPCODE_TYPE_INFO),
            contents: MirTypeContents::IsPrimitive(lir::ValType::I8),
        })
    },
};

unsafe impl Reflect for BinaryOpcode {
    const TYPE: Type = &BINARY_OPCODE_TYPE_INFO;
}

#[derive(Debug)]
pub struct BinaryOperation {
    /// The context to use for the operation. This is necessary for certain
    /// operations such as checking for equality with nobody.
    pub op: BinaryOpcode,
    pub lhs: Box<ExprKind>,
    pub rhs: Box<ExprKind>,
}

impl Expr for BinaryOperation {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        match self.op {
            BinaryOpcode::Add => NlAbstractTy::Numeric,
            BinaryOpcode::Sub => NlAbstractTy::Numeric,
            BinaryOpcode::Mul => NlAbstractTy::Numeric,
            BinaryOpcode::Div => NlAbstractTy::Numeric,
            BinaryOpcode::Lt => NlAbstractTy::Boolean,
            BinaryOpcode::Lte => NlAbstractTy::Boolean,
            BinaryOpcode::Gt => NlAbstractTy::Boolean,
            BinaryOpcode::Gte => NlAbstractTy::Boolean,
            BinaryOpcode::Eq => NlAbstractTy::Boolean,
            BinaryOpcode::Neq => NlAbstractTy::Boolean,
            BinaryOpcode::And => NlAbstractTy::Boolean,
            BinaryOpcode::Or => NlAbstractTy::Boolean,
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.lhs);
        visitor(&self.rhs);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: LocalId) {
        let lhs_ty = self.lhs.output_type(builder.hir);
        let rhs_ty = self.rhs.output_type(builder.hir);

        // special case comparisons against nobody (both as the sole
        // inhabitant of the nobody type and as the inhabitant of agent id types).
        if (lhs_ty == NlAbstractTy::Nobody || rhs_ty == NlAbstractTy::Nobody)
            && (self.op == BinaryOpcode::Eq || self.op == BinaryOpcode::Neq)
        {
            let negate = match self.op {
                BinaryOpcode::Eq => false,
                BinaryOpcode::Neq => true,
                _ => unreachable!(),
            };

            // short circuit on nobody vs nobody comparison
            if lhs_ty == NlAbstractTy::Nobody && rhs_ty == NlAbstractTy::Nobody {
                let result = !negate;
                builder.mir.add_operation_with_dst(
                    local_out.into(),
                    Operation::Const { value: BoxedAny::new(result) },
                );
                return;
            }

            // find the operand that is not statically known to be nobody
            let (operand, operand_ty) = if lhs_ty == NlAbstractTy::Nobody {
                (&self.rhs, rhs_ty.repr())
            } else {
                (&self.lhs, lhs_ty.repr())
            };
            if operand_ty.is::<OptionPatchId>() {
                let operand_pl = builder.translate_expr(operand);
                OptionPatchId::write_check_nobody(builder, negate, local_out, operand_pl);
            } else {
                todo!("TODO(mvp) handle nobody check for other operand types: {:?}", operand_ty);
            }

            return;
        }

        let lhs_ty = lhs_ty.repr();
        let rhs_ty = rhs_ty.repr();

        let lhs_pl = builder.translate_expr(&self.lhs);
        let rhs_pl = builder.translate_expr(&self.rhs);

        use BinaryOpcode as Op;

        // match on known combinations of input types and opcodes
        // TODO(mvp) so far, these additional conditions on color only exist to
        // get ants to compile. we will want to make a full decision on how to
        // treat colors in the engine later
        let final_operation = if (lhs_ty.is::<NlFloat>() || rhs_ty.is::<NlFloat>())
            && (rhs_ty.is::<NlFloat>() || rhs_ty.is::<NlFloat>())
        {
            let opcode = match self.op {
                Op::Add => lir::BinaryOpcode::FAdd,
                Op::Sub => lir::BinaryOpcode::FSub,
                Op::Mul => lir::BinaryOpcode::FMul,
                Op::Div => lir::BinaryOpcode::FDiv,
                Op::Lt => lir::BinaryOpcode::FLt,
                Op::Lte => lir::BinaryOpcode::FLte,
                Op::Gt => lir::BinaryOpcode::FGt,
                Op::Gte => lir::BinaryOpcode::FGte,
                Op::Eq => lir::BinaryOpcode::FEq,
                _ => unimplemented!("unsupported operation"),
            };
            Operation::BinaryOp {
                opcode,
                lhs: PlaceOperand::Move(lhs_pl.place()),
                rhs: PlaceOperand::Move(rhs_pl.place()),
            }
        } else if lhs_ty.is::<bool>() && rhs_ty.is::<bool>() {
            let opcode = match self.op {
                Op::And => lir::BinaryOpcode::And,
                Op::Or => lir::BinaryOpcode::Or,
                _ => unimplemented!("unsupported operation"),
            };
            Operation::BinaryOp {
                opcode,
                lhs: PlaceOperand::Move(lhs_pl.place()),
                rhs: PlaceOperand::Move(rhs_pl.place()),
            }
        } else if lhs_ty.is::<PackedAny>() && rhs_ty.is::<PackedAny>() {
            let opcode_pl = builder.mir.add_operation(
                None,
                Operation::Const { value: BoxedAny::new::<BinaryOpcode>(self.op) },
            );
            match self.op {
                Op::And | Op::Or | Op::Eq | Op::Neq | Op::Lt | Op::Lte | Op::Gt | Op::Gte => {
                    Operation::CallHostFunction {
                        function: &binary_op_any_bool::FN_INFO,
                        args: vec![
                            PlaceOperand::Move(lhs_pl.place()),
                            PlaceOperand::Move(rhs_pl.place()),
                            PlaceOperand::Move(opcode_pl.place()),
                        ],
                    }
                }
                Op::Add | Op::Sub | Op::Mul | Op::Div => mir::Operation::CallHostFunction {
                    function: &binary_op_any::FN_INFO,
                    args: vec![
                        PlaceOperand::Move(lhs_pl.place()),
                        PlaceOperand::Move(rhs_pl.place()),
                        PlaceOperand::Move(opcode_pl.place()),
                    ],
                },
            }
        } else {
            todo!("TODO(mvp) handle other operand types: {:?} and {:?}", lhs_ty, rhs_ty);
        };
        builder.mir.add_operation_with_dst(local_out.into(), final_operation);
    }
}

fn binary_op_any_bool(_lhs: PackedAny, _rhs: PackedAny, _op: BinaryOpcode) -> bool {
    todo!("TODO(mvp)");
}
mod binary_op_any_bool {
    use super::*;
    use crate::mir::HostFunctionInfo;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "binary_op_any_bool",
        parameter_types: &[PackedAny::TYPE, PackedAny::TYPE, BinaryOpcode::TYPE],
        return_type: bool::TYPE,
    };
}
fn binary_op_any(_lhs: PackedAny, _rhs: PackedAny, _op: BinaryOpcode) -> PackedAny {
    todo!("TODO(mvp)");
}
mod binary_op_any {
    use super::*;
    use crate::mir::HostFunctionInfo;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "binary_op_any",
        parameter_types: &[PackedAny::TYPE, PackedAny::TYPE, BinaryOpcode::TYPE],
        return_type: PackedAny::TYPE,
    };
}

#[derive(Debug)]
pub enum UnaryOpcode {
    Neg,
    Not,
}

#[derive(Debug)]
pub struct UnaryOp {
    pub op: UnaryOpcode,
    pub operand: Box<ExprKind>,
}

impl Expr for UnaryOp {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        match self.op {
            UnaryOpcode::Neg => NlAbstractTy::Numeric,
            UnaryOpcode::Not => NlAbstractTy::Boolean,
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.operand);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        let operand_pl = builder.translate_expr(&self.operand);
        let opcode = match self.op {
            UnaryOpcode::Neg => lir::UnaryOpcode::FNeg,
            UnaryOpcode::Not => lir::UnaryOpcode::Not,
        };
        let final_operation =
            mir::Operation::UnaryOp { opcode, operand: PlaceOperand::Move(operand_pl.place()) };
        builder.mir.add_operation_with_dst(local_out.into(), final_operation);
    }
}
