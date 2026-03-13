//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use std::sync::Arc;

use derive_more::derive::TryFrom;

use crate::{
    hir::{Expr, ExprKind, NlAbstractTy, Program, build_mir::HirToMirFnBuilder},
    mir::{
        self,
        reflection::{MirReflect, MirType, MirTypeContents, MirTypeInfo},
    },
    sim::{
        patch::{OptionPatchId, check_patch_nobody},
        value::{Nobody, PackedAny},
    },
    util::reflection::{Reflect, TypeInfo},
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

unsafe impl Reflect for BinaryOpcode {
    const TYPE_INFO: TypeInfo = TypeInfo::new_copy::<BinaryOpcode>("BinaryOpcode", false);
}

unsafe impl MirReflect for BinaryOpcode {
    fn mir_type() -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: Some(&<BinaryOpcode>::TYPE_INFO),
            contents: MirTypeContents::IsPrimitive(lir::ValType::I8),
        })
    }
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

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        todo!()
    }
}

macro_mir_dsl::mir_intrinsic! {
    fn binary_op(op: BinaryOpcode)(lhs: any, rhs: any) -> (out: bool) {
        let lhs_ty = type_of!(lhs);
        let rhs_ty = type_of!(rhs);

        let lhs_is_nobody = lhs_ty.is::<Nobody>();
        let rhs_is_nobody = rhs_ty.is::<Nobody>();

        // special case comparisons againt nobody
        if (lhs_is_nobody || rhs_is_nobody)
            && (op == BinaryOpcode::Eq || op == BinaryOpcode::Neq)
        {
            let negate = match op {
                BinaryOpcode::Eq => false,
                BinaryOpcode::Neq => true,
                _ => unreachable!(),
            };

            // short circuit on nobody vs nobody comparison
            if lhs_is_nobody && rhs_is_nobody {
                // TODO assign the out type to bool
                mir! {
                    out = const { if negate { false } else { true } };
                }
                return;
            };

            // find the operand that is not known to be nobody
            let (operand, operand_ty) = if lhs_is_nobody {
                (place_ref!(rhs), rhs_ty)
            } else {
                (place_ref!(lhs), lhs_ty)
            };
            if operand_ty.is::<OptionPatchId>() {
                mir! {
                    let negate_rt: bool = const { negate };
                    out = const { check_patch_nobody }(negate_rt, &(place_use!(operand).cast::<OptionPatchId>()));
                }
            } else {
                unimplemented!("TODO(mvp) handle nobody check for other operand types: {:?}", operand_ty);
            }
        } else if lhs_ty.is::<bool>() && rhs_ty.is::<bool>() {
            match op {
                BinaryOpcode::And => mir! {
                    out = const { logical_and }(lhs.cast::<bool>(), rhs.cast::<bool>());
                },
                BinaryOpcode::Or => mir! {
                    out = const { logical_or }(lhs.cast::<bool>(), rhs.cast::<bool>());
                },
                _ => unimplemented!("unsupported operation"),
            }
        } else {
            // TODO
            todo!("TODO(mvp) handle other operand types: {:?} and {:?}", lhs_ty, rhs_ty);
        };
    }
}

mod logical_and {
    use super::*;
    use crate::mir::HostFunctionInfo;

    pub fn call(lhs: bool, rhs: bool) -> bool {
        todo!()
    }

    pub const FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "logical_and",
        parameter_types: &[&bool::TYPE_INFO, &bool::TYPE_INFO],
        return_type: &bool::TYPE_INFO,
    };
}

mod logical_or {
    use super::*;
    use crate::mir::HostFunctionInfo;

    pub fn call(lhs: bool, rhs: bool) -> bool {
        todo!()
    }

    pub const FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "logical_or",
        parameter_types: &[&bool::TYPE_INFO, &bool::TYPE_INFO],
        return_type: &bool::TYPE_INFO,
    };
}

fn binary_op_any_bool(_lhs: PackedAny, _rhs: PackedAny, _op: BinaryOpcode) -> bool {
    todo!("TODO(mvp)");
}
mod binary_op_any_bool {
    use super::*;
    use crate::mir::HostFunctionInfo;

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "binary_op_any_bool",
        parameter_types: &[&PackedAny::TYPE_INFO, &PackedAny::TYPE_INFO, &BinaryOpcode::TYPE_INFO],
        return_type: &bool::TYPE_INFO,
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
        parameter_types: &[&PackedAny::TYPE_INFO, &PackedAny::TYPE_INFO, &BinaryOpcode::TYPE_INFO],
        return_type: &PackedAny::TYPE_INFO,
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
        todo!(
            "
            let final_operation = mir::Operation::ScalarUnaryOp {{ opcode, operand: operand_pl.place().move_out() }};
            builder.mir.add_operation_with_dst(local_out.into(), final_operation);
            "
        );
    }
}
