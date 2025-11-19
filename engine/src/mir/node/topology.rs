//! Primitives relating purely to the topology of the world.

use derive_more::derive::Display;

use crate::{
    exec::jit::host_fn,
    mir::{
        Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

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

impl Node for OffsetDistanceByHeading {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.position, self.amt, self.heading]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        todo!("TODO(mvp) return Point type")
    }

    fn write_lir_execution(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to call a host function")
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

impl Node for PatchAt {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.x, self.y]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Patch)
    }
}

#[derive(Debug, Display)]
#[display("MaxPxcor")]
pub struct MaxPxcor {
    /// The execution context to use.
    pub context: NodeId,
}

impl Node for MaxPxcor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}

#[derive(Debug, Display)]
#[display("MaxPycor")]
pub struct MaxPycor {
    /// The execution context to use.
    pub context: NodeId,
}

impl Node for MaxPycor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}

#[derive(Debug, Display)]
#[display("EuclideanDistanceNoWrap {a:?} {b:?}")]
pub struct EuclideanDistanceNoWrap {
    /// The first point.
    pub a: NodeId,
    /// The second point.
    pub b: NodeId,
}

impl Node for EuclideanDistanceNoWrap {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.a, self.b]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let mut args = Vec::new();
        args.extend(lir_builder.get_node_results(program, function, nodes, self.a));
        args.extend(lir_builder.get_node_results(program, function, nodes, self.b));

        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::EUCLIDEAN_DISTANCE_NO_WRAP,
            args.into_boxed_slice(),
        ));
        Ok(())
    }
}

/// A node that constructs a point from two floating-point values.
#[derive(Debug, Display)]
#[display("PointConstructor")]
pub struct PointConstructor {
    pub x: NodeId,
    pub y: NodeId,
}

impl Node for PointConstructor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.x, self.y]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Point)
    }

    fn write_lir_execution(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO simply pass through the LIR values")
    }
}
