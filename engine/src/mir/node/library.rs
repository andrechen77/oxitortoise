//! Nodes representing calls to NetLogo primitives which are more complex than
//! add, subtract, etc., but can be modeled as "standard library functions."
//! This does not refer to calls to other NetLogo functions or external
//! functions.

use derive_more::derive::Display;
use lir::{ValRef, smallvec::smallvec};
use slotmap::{Key, SlotMap};

use crate::{
    mir::{EffectfulNode, LocalDeclaration, LocalId, NodeId, build_lir::LirInsnBuilder},
    sim::{patch::PatchVarDesc, turtle::BreedId, value::NetlogoInternalType},
};

#[derive(Debug, Display)]
#[display("ClearAll")]
pub struct ClearAll {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for ClearAll {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::InsnKind::CallHostFunction {
            function: lir_builder.program_builder.host_function_ids.clear_all,
            output_type: smallvec![],
            args: Box::new([ctx_ptr]),
        });
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("Diffuse {variable:?}")]
pub struct Diffuse {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch variable to diffuse.
    pub variable: PatchVarDesc,
    /// The amount of the variable to diffuse.
    pub amt: NodeId,
}

impl EffectfulNode for Diffuse {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.amt]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
    }
}

#[derive(Debug, Display)]
#[display("ResetTicks {context:?}")]
pub struct ResetTicks {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for ResetTicks {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::InsnKind::CallHostFunction {
            function: lir_builder.program_builder.host_function_ids.reset_ticks,
            output_type: smallvec![],
            args: Box::new([ctx_ptr]),
        });
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("AdvanceTick")]
pub struct AdvanceTick {
    /// The context whose tick is being advanced.
    pub context: NodeId,
}

impl EffectfulNode for AdvanceTick {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }
}

#[derive(Debug, Display)]
#[display("GetTick")]
pub struct GetTick {
    /// The context whose tick is being gotten.
    pub context: NodeId,
}

impl EffectfulNode for GetTick {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(NetlogoInternalType::FLOAT)
    }
}

#[derive(Debug, Display)]
#[display("CreateTurtles {breed:?}")]
pub struct CreateTurtles {
    /// The execution context to use.
    pub context: NodeId,
    /// The breed of turtles to create.
    pub breed: BreedId,
    /// The number of turtles to create.
    pub num_turtles: NodeId,
    /// A closure representing the commands to run for each created turtle.
    pub body: NodeId,
}

impl EffectfulNode for CreateTurtles {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.num_turtles, self.body]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let breed_id = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
            ty: lir::ValType::I64,
            value: self.breed.data().as_ffi(),
        }));
        let &[num_turtles] = lir_builder.get_node_results(nodes, self.num_turtles) else {
            panic!("expected node outputting number of turtles to be a single LIR value")
        };
        let &[env_ptr, fn_ptr] = lir_builder.get_node_results(nodes, self.body) else {
            panic!("expected node outputting closure body to be two LIR values");
        };
        let pc = lir_builder.push_lir_insn(lir::InsnKind::CallHostFunction {
            function: lir_builder.program_builder.host_function_ids.create_turtles,
            output_type: smallvec![],
            args: Box::new([ctx_ptr, ValRef(breed_id, 0), num_turtles, env_ptr, fn_ptr]),
        });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![ValRef(pc, 0)]);
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AskRecipient {
    AllTurtles,
    AllPatches,
    TurtleAgentset(NodeId),
    PatchAgentset(NodeId),
    SingleTurtle(NodeId),
    SinglePatch(NodeId),
    Dynamic(NodeId),
    // TODO add links
}

/// A node representing an "ask" construct.
#[derive(Debug, Display)]
#[display("Ask")]
pub struct Ask {
    /// The execution context to use for the ask.
    pub context: NodeId,
    /// The agents being asked.
    pub recipients: NodeId,
    /// A closure representing the commands to run for each recipient.
    pub body: NodeId,
}

impl EffectfulNode for Ask {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.recipients, self.body]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    // TODO if possible this node should be optimized into AskAllTurtles, etc.
    // maybe we can hijack lowering expand for this? or repurpose that function
    // as one that checks for all optimizations?

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
    }
}

#[derive(Debug, Display)]
#[display("AskAllTurtles")]
pub struct AskAllTurtles {
    /// The execution context to use for the ask.
    pub context: NodeId,
    /// A closure representing the commands to run for each recipient.
    pub body: NodeId,
}

impl EffectfulNode for AskAllTurtles {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.body]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[env_ptr, fn_ptr] = lir_builder.get_node_results(nodes, self.body) else {
            panic!("expected node outputting closure body to be two LIR values");
        };
        let pc = lir_builder.push_lir_insn(lir::InsnKind::CallHostFunction {
            function: lir_builder.program_builder.host_function_ids.ask_all_turtles,
            output_type: smallvec![], // TODO this should be inferred from the host function declaration
            args: Box::new([ctx_ptr, env_ptr, fn_ptr]),
        });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("TurtleRotate")]
pub struct TurtleRotate {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to rotate.
    pub turtle: NodeId,
    /// The amount to rotate.
    pub angle: NodeId,
}

impl EffectfulNode for TurtleRotate {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.angle]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        workspace: &crate::workspace::Workspace,
        nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        todo!()
    }
}

#[derive(Debug, Display)]
#[display("TurtleForward")]
pub struct TurtleForward {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to move.
    pub turtle: NodeId,
    /// The distance to move.
    pub distance: NodeId,
}

impl EffectfulNode for TurtleForward {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.distance]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }

    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        workspace: &crate::workspace::Workspace,
        nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        todo!()
    }
}

#[derive(Debug, Display)]
#[display("OffsetDistanceByHeading")]
pub struct OffsetDistanceByHeading {
    /// The position to offset.
    pub position: NodeId,
    /// The distance to offset.
    pub amt: NodeId,
    /// The heading to offset by.
    pub heading: NodeId,
}

impl EffectfulNode for OffsetDistanceByHeading {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.position, self.amt, self.heading]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(NetlogoInternalType::POINT)
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
    }
}

#[derive(Debug, Display)]
#[display("Distancexy {x:?} {y:?}")]
pub struct Distancexy {
    /// The agent to get the distance from.
    pub agent: NodeId,
    /// The x coordinate.
    pub x: NodeId,
    /// The y coordinate.
    pub y: NodeId,
}

impl EffectfulNode for Distancexy {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.agent, self.x, self.y]
    }

    fn output_type(
        &self,
        workspace: &crate::workspace::Workspace,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        Some(NetlogoInternalType::FLOAT)
    }
}

#[derive(Debug, Display)]
#[display("MaxPxcor")]
pub struct MaxPxcor {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for MaxPxcor {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(NetlogoInternalType::FLOAT)
    }
}

#[derive(Debug, Display)]
#[display("MaxPycor")]
pub struct MaxPycor {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for MaxPycor {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(NetlogoInternalType::FLOAT)
    }
}

/// https://docs.netlogo.org/dict/scale-color.html
#[derive(Debug, Display)]
#[display("ScaleColor")]
pub struct ScaleColor {
    pub color: NodeId,
    pub number: NodeId,
    pub range1: NodeId,
    pub range2: NodeId,
}

impl EffectfulNode for ScaleColor {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.color, self.number, self.range1, self.range2]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(NetlogoInternalType::FLOAT)
    }
}
