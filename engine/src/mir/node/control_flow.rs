//! Nodes that represent control flow constructs. These are the fundamental
//! building blocks that allow sequential execution of statements in a function.

use derive_more::derive::Display;
use lir::typed_index_collections::TiVec;
use tracing::trace;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

/// A node representing a block of statements that are sequentially executed.
/// For a given execution of this node, statements are executed sequentially
/// until a break is encountered targeting this block; this stops execution of
/// nodes under this block and the break value becomes the output of this node.
///
/// For the purposes of analysis and transformation, the only way to exit the
/// block is with a break statement. If these do not exist in the NetLogo
/// source, they should be automatically inserted when this node is created.
#[derive(Debug, Display)]
#[display("Block")]
pub struct Block {
    pub statements: Vec<NodeId>,
    /// The IDs of all [`Break`] nodes that can break out of this block with a
    /// value. Used to calculate the output type of the block.
    pub come_from: Vec<NodeId>,
}

/// A node representing a branching control flow construct. For a given
/// execution of this node, the condition is always executed; based on the
/// result of the condition, only one of the branch nodes is executed and
/// becomes the output of this node.
#[derive(Debug, Display)]
#[display("IfElse")]
pub struct IfElse {
    pub condition: NodeId,
    pub then_block: NodeId,
    pub else_block: NodeId,
}

/// A node representing a loop control flow construct. For a given execution of
/// this node, the node representing the number of repetition is executed
/// exactly once, and the body can be executed any number of times.
#[derive(Debug, Display)]
#[display("Repeat")]
pub struct Repeat {
    pub num_repetitions: NodeId,
    pub body_block: NodeId,
}

/// A node representing a breaking control flow construct. Executing this node
/// will break out of some control flow construct, returning a certain value.
#[derive(Debug, Display)]
#[display("Break({:?})", target)]
pub struct Break {
    /// The Block node being targeted by this break.
    pub target: NodeId,
    pub value: Option<NodeId>,
}

impl Node for Block {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        self.statements.clone()
    }

    fn output_type(&self, program: &Program, fn_id: FunctionId) -> MirTy {
        let mut output_type = NlAbstractTy::Bottom;
        for &come_from in &self.come_from {
            let NodeKind::Break(Break { target: _, value }) = program.nodes[come_from] else {
                panic!("expected a Break node");
            };
            let break_ty = value.map_or(NlAbstractTy::Unit, |v| {
                program.nodes[v].output_type(program, fn_id).abstr.unwrap()
            });
            output_type = output_type.join(break_ty);
        }
        output_type.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        lir_builder.block_to_insn_seq.insert(my_node_id, *lir_builder.insn_seqs.last().unwrap());
        for &stmt in &self.statements {
            trace!("writing LIR execution for {:?} {:?}", stmt, program.nodes[stmt]);
            program.nodes[stmt].write_lir_execution::<I>(program, stmt, lir_builder)?
        }
        Ok(())
    }
}

impl Node for IfElse {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.condition, self.then_block, self.else_block]
    }

    fn output_type(&self, program: &Program, fn_id: FunctionId) -> MirTy {
        let then_ty = program.nodes[self.then_block].output_type(program, fn_id).abstr.unwrap();
        let else_ty = program.nodes[self.else_block].output_type(program, fn_id).abstr.unwrap();
        then_ty.join(else_ty).into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // create a new instruction sequence for each branch
        let then_body = lir_builder.product.insn_seqs.push_and_get_key(TiVec::new());
        let else_body = lir_builder.product.insn_seqs.push_and_get_key(TiVec::new());
        lir_builder.with_insn_seq(then_body, |lir_builder| {
            program.nodes[self.then_block].write_lir_execution::<I>(
                program,
                self.then_block,
                lir_builder,
            )
        })?;
        lir_builder.with_insn_seq(else_body, |lir_builder| {
            program.nodes[self.else_block].write_lir_execution::<I>(
                program,
                self.else_block,
                lir_builder,
            )
        })?;

        // evaluate the condition and insert the branch instruction
        let &[condition] = lir_builder.get_node_results::<I>(program, self.condition) else {
            panic!("a condition should evaluate to a single LIR value");
        };
        trace!("calculating output type for if-else in function {:?}", lir_builder.fn_id);
        let my_output_type = self.output_type(program, lir_builder.fn_id).repr();
        lir_builder.push_lir_insn(lir::InsnKind::IfElse(lir::IfElse {
            condition,
            output_type: my_output_type
                .info()
                .mem_repr
                .expect("a value being returned should have a known ABI")
                .iter()
                .map(|&(_, r#type)| r#type.loaded_type())
                .collect(),
            then_body,
            else_body,
        }));
        Ok(())
    }
}

impl Node for Repeat {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.num_repetitions, self.body_block]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        _program: &Program,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!()
    }
}

impl Node for Break {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        self.value.iter().copied().collect()
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        // a break diverges, it never returns
        NlAbstractTy::Bottom.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let target = lir_builder.block_to_insn_seq[&self.target];
        let values = Box::from(
            self.value
                .map_or_else(Default::default, |v| lir_builder.get_node_results::<I>(program, v)),
        );
        lir_builder.push_lir_insn(lir::InsnKind::Break { target, values });
        Ok(())
    }
}
