//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::jit::host_fn,
    mir::{
        Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::value::{DynBox, NlBool, NlFloat},
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

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(match self.op {
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
        })
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) be prepared for other possible input types and adjust
        // the implementation accordingly
        // TODO(mvp) assert that the types of the operands are compatible with the operation
        let lhs_type = nodes[self.lhs].output_type(program, function, nodes).repr();
        let rhs_type = nodes[self.rhs].output_type(program, function, nodes).repr();

        let &[lhs] = lir_builder.get_node_results(program, function, nodes, self.lhs) else {
            unimplemented!();
        };
        let &[rhs] = lir_builder.get_node_results(program, function, nodes, self.rhs) else {
            unimplemented!();
        };
        use BinaryOpcode as Op;
        if lhs_type == NlFloat::CONCRETE_TY && rhs_type == NlFloat::CONCRETE_TY {
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
            let opcode = self.op as u8;
            let opcode_val = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
                ty: lir::ValType::I8,
                value: opcode as u64,
            }));
            let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
                host_fn::DYNBOX_BINARY_OP,
                Box::new([lhs, rhs, lir::ValRef(opcode_val, 0)]),
            ));
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

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(match self.op {
            UnaryOpcode::Neg => NlAbstractTy::Numeric,
            UnaryOpcode::Not => NlAbstractTy::Boolean,
        })
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[operand] = lir_builder.get_node_results(program, function, nodes, self.operand)
        else {
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
