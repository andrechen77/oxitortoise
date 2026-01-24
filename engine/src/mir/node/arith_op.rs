//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::jit::host_fn,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform, Program,
        WriteLirError, build_lir::LirInsnBuilder, node,
    },
    sim::{
        color::Color,
        value::{DynBox, NlBool, NlFloat},
    },
    util::reflection::Reflect,
};

#[derive(Debug, Display, Clone, Copy)]
#[display("{_0:?}")]
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

#[derive(Debug, Display)]
#[display("{op:?}")]
pub struct BinaryOperation {
    /// The context to use for the operation. This is necessary for certain
    /// operations such as checking for equality with nobody.
    pub context: NodeId,
    pub op: BinaryOpcode,
    pub lhs: NodeId,
    pub rhs: NodeId,
}

impl Node for BinaryOperation {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.lhs, self.rhs]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
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
        .into()
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn decompose_with_check_nobody(
            program: &mut Program,
            fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let &NodeKind::BinaryOperation(BinaryOperation { context, op, lhs, rhs }) =
                &program.nodes[my_node_id]
            else {
                return false;
            };

            let lhs_type = program.nodes[lhs]
                .output_type(program, fn_id)
                .abstr
                .expect("operand must have an abstract type");
            let rhs_type = program.nodes[rhs]
                .output_type(program, fn_id)
                .abstr
                .expect("operand must have an abstract type");

            // expect that the operation is either Eq or Neq
            let negate = match op {
                BinaryOpcode::Eq => false,
                BinaryOpcode::Neq => true,
                _ => return false,
            };

            // find the operand that is being compared to nobody
            let operand = match (lhs_type, rhs_type) {
                (NlAbstractTy::Nobody, _) => rhs,
                (_, NlAbstractTy::Nobody) => lhs,
                _ => return false,
            };

            program.nodes[my_node_id] =
                NodeKind::from(node::CheckNobody { context, agent: operand, negate });

            true
        }
        Some(Box::new(decompose_with_check_nobody))
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) be prepared for other possible input types and adjust
        // the implementation accordingly
        // TODO(mvp) assert that the types of the operands are compatible with the operation
        let lhs_type = program.nodes[self.lhs].output_type(program, lir_builder.fn_id).repr();
        let rhs_type = program.nodes[self.rhs].output_type(program, lir_builder.fn_id).repr();

        let &[lhs] = lir_builder.get_node_results(program, self.lhs) else {
            unimplemented!();
        };
        let &[rhs] = lir_builder.get_node_results(program, self.rhs) else {
            unimplemented!();
        };
        use BinaryOpcode as Op;
        // TODO(mvp) so far, this additional conditions on color only exist to
        // get ants to compile. we will want to make a full decision on how to
        // treat colors in the engine later
        if (lhs_type == NlFloat::CONCRETE_TY || lhs_type == Color::CONCRETE_TY)
            && (rhs_type == NlFloat::CONCRETE_TY || rhs_type == Color::CONCRETE_TY)
        {
            let op = match self.op {
                Op::Add => lir::BinaryOpcode::FAdd,
                Op::Sub => lir::BinaryOpcode::FSub,
                Op::Mul => lir::BinaryOpcode::FMul,
                Op::Div => lir::BinaryOpcode::FDiv,
                Op::Lt => lir::BinaryOpcode::FLt,
                Op::Lte => lir::BinaryOpcode::FLte,
                Op::Gt => lir::BinaryOpcode::FGt,
                Op::Gte => lir::BinaryOpcode::FGte,
                Op::Eq => lir::BinaryOpcode::FEq,
                _ => unimplemented!(),
            };

            let result = lir_builder.push_lir_insn(lir::InsnKind::BinaryOp { op, lhs, rhs });
            lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(result, 0)]);
            Ok(())
        } else if lhs_type == NlBool::CONCRETE_TY && rhs_type == NlBool::CONCRETE_TY {
            let op = match self.op {
                Op::And => lir::BinaryOpcode::And,
                Op::Or => lir::BinaryOpcode::Or,
                _ => unimplemented!(),
            };
            let result = lir_builder.push_lir_insn(lir::InsnKind::BinaryOp { op, lhs, rhs });
            lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(result, 0)]);
            Ok(())
        } else if lhs_type == DynBox::CONCRETE_TY && rhs_type == DynBox::CONCRETE_TY {
            let opcode = self.op as u32;
            let opcode_val = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
                ty: lir::ValType::I32,
                value: opcode as u64,
            }));
            let pc = match self.op {
                Op::And | Op::Or | Op::Eq | Op::Neq | Op::Lt | Op::Lte | Op::Gt | Op::Gte => {
                    lir_builder.push_lir_insn(lir::generate_host_function_call(
                        host_fn::DYNBOX_BOOL_BINARY_OP,
                        Box::new([lhs, rhs, lir::ValRef(opcode_val, 0)]),
                    ))
                }
                // TODO(mvp) like the bool-returning specializations, make sure
                // that other operations actually return the correct type as
                // predicted by the output_type method. it is incorrect to
                // use a function that returns a type that disagrees, e.g. dynboc
                _ => lir_builder.push_lir_insn(lir::generate_host_function_call(
                    host_fn::DYNBOX_BINARY_OP,
                    Box::new([lhs, rhs, lir::ValRef(opcode_val, 0)]),
                )),
            };

            lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
            Ok(())
        } else {
            todo!("binary op has operands of types {:?} and {:?}", lhs_type, rhs_type);
        }
    }
}

#[derive(Debug, Display)]
#[display("{op:?}")]
pub enum UnaryOpcode {
    Neg,
    Not,
}

#[derive(Debug, Display)]
#[display("{op:?}")]
pub struct UnaryOp {
    pub op: UnaryOpcode,
    pub operand: NodeId,
}

impl Node for UnaryOp {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.operand]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        match self.op {
            UnaryOpcode::Neg => NlAbstractTy::Numeric,
            UnaryOpcode::Not => NlAbstractTy::Boolean,
        }
        .into()
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[operand] = lir_builder.get_node_results(program, self.operand) else {
            todo!("TODO(mvp) are there operands that are multi-register values?");
        };
        let op = match self.op {
            UnaryOpcode::Neg => lir::UnaryOpcode::FNeg,
            UnaryOpcode::Not => lir::UnaryOpcode::Not,
        };
        let result = lir_builder.push_lir_insn(lir::InsnKind::UnaryOp { op, operand });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(result, 0)]);
        Ok(())
    }
}
