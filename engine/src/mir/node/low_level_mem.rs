//! Nodes representing low-level memory operations.

use derive_more::derive::Display;
use lir::smallvec::{SmallVec, smallvec};

use crate::{
    exec::jit::InstallLir,
    mir::{FunctionId, MirTy, Node, NodeId, Program, WriteLirError, build_lir::LirInsnBuilder},
    util::reflection::{ConcreteTy, Reflect},
};

#[derive(Debug, Display)]
#[display("MemLoad offset={offset:?}")]
pub struct MemLoad {
    /// The pointer to the memory to load from.
    pub ptr: NodeId,
    /// The byte offset of the field to load.
    pub offset: usize,
    /// The type of the value to load.
    pub ty: ConcreteTy,
}

impl Node for MemLoad {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        self.ty.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let mem_repr = self.ty.info().mem_repr.expect("mem load type must have known ABI");

        let &[ptr] = lir_builder.get_node_results::<I>(program, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };
        let mut val_refs = SmallVec::new();
        for &(additional_offset, r#type) in mem_repr {
            let pc = lir_builder.push_lir_insn(lir::InsnKind::MemLoad {
                r#type,
                ptr,
                offset: self.offset + additional_offset,
            });
            val_refs.push(lir::ValRef(pc, 0));
        }
        lir_builder.node_to_lir.insert(my_node_id, val_refs);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("MemStore offset={offset:?}")]
pub struct MemStore {
    /// The pointer to the memory to store to.
    pub ptr: NodeId,
    /// The byte offset of the field to store.
    pub offset: usize,
    /// The value to store.
    pub value: NodeId,
}

impl Node for MemStore {
    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr, self.value]
    }

    fn is_pure(&self) -> bool {
        true
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        MirTy::default()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ptr] = lir_builder.get_node_results::<I>(program, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };

        let ty = program.nodes[self.value].output_type(program, lir_builder.fn_id).repr();
        let mem_repr = ty.info().mem_repr.expect("mem store value type must have known ABI");
        let values = lir_builder.get_node_results::<I>(program, self.value).to_owned();
        assert_eq!(
            values.len(),
            mem_repr.len(),
            "mem store value must have same number of registers as the type"
        );
        for (&(additional_offset, r#type), &value) in mem_repr.iter().zip(values.iter()) {
            lir_builder.push_lir_insn(lir::InsnKind::MemStore {
                r#type,
                offset: self.offset + additional_offset,
                ptr,
                value,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("DeriveField offset={offset:?}")]
pub struct DeriveField {
    /// The pointer to the memory to derive.
    pub ptr: NodeId,
    /// The byte offset of the field.
    pub offset: usize,
}

impl Node for DeriveField {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        <*mut u8 as Reflect>::CONCRETE_TY.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ptr] = lir_builder.get_node_results::<I>(program, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };
        let pc = lir_builder.push_lir_insn(lir::InsnKind::DeriveField { offset: self.offset, ptr });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("DeriveElement stride={stride:?}")]
pub struct DeriveElement {
    /// The pointer to the memory to derive.
    pub ptr: NodeId,
    /// The index of the element.
    pub index: NodeId,
    /// The stride of the element.
    pub stride: usize,
}

impl Node for DeriveElement {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr, self.index]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        <*mut u8 as Reflect>::CONCRETE_TY.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ptr] = lir_builder.get_node_results::<I>(program, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };
        let &[index] = lir_builder.get_node_results::<I>(program, self.index) else {
            panic!("expected a node that outputs an index to be a single LIR value");
        };
        let pc = lir_builder.push_lir_insn(lir::InsnKind::DeriveElement {
            element_size: self.stride,
            ptr,
            index,
        });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}
