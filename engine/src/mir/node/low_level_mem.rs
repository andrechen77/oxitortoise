//! Nodes representing the derivation of some "included" value from a larger
//! "including" value. For example, deriving a pointer to the workspace from a
//! context pointer.

use derive_more::derive::Display;
use lir::smallvec::smallvec;
use slotmap::SlotMap;

use crate::{
    mir::{EffectfulNode, NodeId},
    sim::value::NetlogoInternalType,
};

#[derive(Debug, Display)]
#[display("MemLoad offset={offset:?}")]
pub struct MemLoad {
    /// The pointer to the memory to load from.
    pub ptr: NodeId,
    /// The byte offset of the field to load.
    pub offset: usize,
    /// The type of the value to load.
    pub ty: NetlogoInternalType,
}

impl EffectfulNode for MemLoad {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &slotmap::SlotMap<crate::mir::LocalId, crate::mir::LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        // this returns None not because it doesn't have an output, but
        // because we don't know the type
        None
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[lir_type] = self.ty.to_lir_type().as_slice() else {
            // this panic is purely a limitation of this implementation; there's
            // no inherent limitation that makes insertion of multiple mem load
            // instructions impossible
            panic!("don't know how to load a value that takes up multiple LIR registers");
        };
        let &[ptr] = lir_builder.get_node_results(nodes, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };
        let pc = lir_builder.push_lir_insn(lir::InsnKind::MemLoad {
            r#type: lir_type,
            ptr,
            offset: self.offset,
        });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
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

impl EffectfulNode for MemStore {
    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr, self.value]
    }

    fn has_side_effects(&self) -> bool {
        false
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &slotmap::SlotMap<crate::mir::LocalId, crate::mir::LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ptr] = lir_builder.get_node_results(nodes, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };
        let &[value] = lir_builder.get_node_results(nodes, self.value) else {
            panic!(
                "expected a node that outputs a about-to-be-stored value to be a single LIR value"
            );
        };
        lir_builder.push_lir_insn(lir::InsnKind::MemStore { ptr, value, offset: self.offset });
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

impl EffectfulNode for DeriveField {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &slotmap::SlotMap<crate::mir::LocalId, crate::mir::LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(NetlogoInternalType::UNTYPED_PTR)
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ptr] = lir_builder.get_node_results(nodes, self.ptr) else {
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

impl EffectfulNode for DeriveElement {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.ptr, self.index]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &slotmap::SlotMap<crate::mir::LocalId, crate::mir::LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        Some(NetlogoInternalType::UNTYPED_PTR)
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ptr] = lir_builder.get_node_results(nodes, self.ptr) else {
            panic!("expected a node that outputs a pointer to be a single LIR value");
        };
        let &[index] = lir_builder.get_node_results(nodes, self.index) else {
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
