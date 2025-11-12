//! Nodes representing calls to NetLogo primitives which are more complex than
//! add, subtract, etc., but can be modeled as "standard library functions."
//! This does not refer to calls to other NetLogo functions or external
//! functions.

use derive_more::derive::Display;
use lir::{ValRef, smallvec::smallvec};
use slotmap::Key;

use crate::{
    exec::jit::host_fn,
    mir::{
        EffectfulNode, Function, MirTy, NlAbstractTy, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::{patch::PatchVarDesc, turtle::BreedId},
};

#[derive(Debug, Display)]
#[display("ClearAll")]
pub struct ClearAll {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for ClearAll {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::CLEAR_ALL,
            Box::new([ctx_ptr]),
        ));
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.amt]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &Nodes,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to call a host function")
    }
}

#[derive(Debug, Display)]
#[display("ResetTicks {context:?}")]
pub struct ResetTicks {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for ResetTicks {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::RESET_TICKS,
            Box::new([ctx_ptr]),
        ));
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }
}

#[derive(Debug, Display)]
#[display("GetTick")]
pub struct GetTick {
    /// The context whose tick is being gotten.
    pub context: NodeId,
}

impl EffectfulNode for GetTick {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.num_turtles, self.body]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
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
        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::CREATE_TURTLES,
            Box::new([ctx_ptr, ValRef(breed_id, 0), num_turtles, env_ptr, fn_ptr]),
        ));
        lir_builder.node_to_lir.insert(my_node_id, smallvec![ValRef(pc, 0)]);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("Of")]
pub struct Of {
    /// The execution context to use for the ask.
    pub context: NodeId,
    /// The recipients to ask.
    pub recipients: NodeId,
    /// A closure representing the reporter to run for each recipient.
    pub body: NodeId,
}

impl EffectfulNode for Of {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.recipients, self.body]
    }

    fn output_type(
        &self,
        program: &crate::mir::Program,
        function: &crate::mir::Function,
        nodes: &crate::mir::Nodes,
    ) -> MirTy {
        let MirTy::Abstract(NlAbstractTy::Closure(closure)) =
            nodes[self.body].output_type(program, function, nodes)
        else {
            panic!("expected node outputting closure body to be a closure")
        };

        MirTy::Abstract(*closure.return_ty)
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.angle]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }
}

#[derive(Debug, Display)]
#[display("CanMove")]
pub struct CanMove {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to check movement for
    pub turtle: NodeId,
    /// The distance to check
    pub distance: NodeId,
}

impl EffectfulNode for CanMove {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Boolean)
    }

    // TODO(mvp_ants) add transformation to turn the node into "patch ahead != nobody"
}

#[derive(Debug, Display)]
pub enum PatchLocRelation {
    LeftAhead,
    RightAhead,
}

#[derive(Debug, Display)]
#[display("PatchNearby {relative_loc:?}")]
pub struct PatchRelative {
    /// The execution context to use.
    pub context: NodeId,
    /// The location to check relative to the patch
    pub relative_loc: PatchLocRelation,
    /// The turtle to check from
    pub turtle: NodeId,
    /// The distance to check
    pub distance: NodeId,
    /// The heading to check
    pub heading: NodeId,
}

impl EffectfulNode for PatchRelative {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance, self.heading]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Patch)
    }
}

#[derive(Debug, Display)]
#[display("PatchAt {x:?} {y:?}")]
pub struct PatchAt {
    /// The x coordinate.
    pub x: NodeId,
    /// The y coordinate.
    pub y: NodeId,
}

impl EffectfulNode for PatchAt {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.x, self.y]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Patch)
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
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.position, self.amt, self.heading]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        todo!("TODO(mvp) return Point type")
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &Nodes,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to call a host function")
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
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.agent, self.x, self.y]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}

#[derive(Debug, Display)]
#[display("MaxPxcor")]
pub struct MaxPxcor {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for MaxPxcor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}

#[derive(Debug, Display)]
#[display("MaxPycor")]
pub struct MaxPycor {
    /// The execution context to use.
    pub context: NodeId,
}

impl EffectfulNode for MaxPycor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}

#[derive(Debug, Display)]
#[display("OneOf")]
pub struct OneOf {
    pub context: NodeId,
    pub xs: NodeId,
}

impl EffectfulNode for OneOf {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.xs]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        let out_type = match _nodes[self.xs].output_type(_program, _function, _nodes) {
            MirTy::Abstract(NlAbstractTy::Agentset { agent_type }) => agent_type,
            MirTy::Abstract(NlAbstractTy::List { element_ty }) => element_ty,
            x => panic!("Impossible argument type for `one-of`: {:?}", x),
        };

        MirTy::Abstract(*out_type)
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
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.color, self.number, self.range1, self.range2]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Color)
    }
}

/// Returns a random integer between 0 (inclusive) and bound (exclusive)
#[derive(Debug, Display)]
#[display("RandomInt")]
pub struct RandomInt {
    /// The execution context to use.
    pub context: NodeId,
    pub bound: NodeId,
}

impl EffectfulNode for RandomInt {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.bound]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}

#[derive(Debug, Display)]
#[display("SetDefaultShape")]
pub struct SetDefaultShape {
    /// The breed to set the default shape for.
    pub breed: NodeId,
    /// The shape to set.
    pub shape: NodeId,
}

impl EffectfulNode for SetDefaultShape {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.breed, self.shape]
    }

    fn output_type(
        &self,
        _program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &Nodes,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) write LIR code to set the default shape for the breed
        Ok(())
    }
}
