//! Primitives relating purely to the topology of the world.

use std::mem::offset_of;

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::{CanonExecutionContext, jit::InstallLir},
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform, Program,
        WriteLirError, build_lir::LirInsnBuilder, node,
    },
    sim::{
        topology::{OFFSET_TOPOLOGY_TO_MAX_PXCOR, OFFSET_TOPOLOGY_TO_MAX_PYCOR},
        value::NlFloat,
        world::World,
    },
    util::reflection::Reflect,
    workspace::Workspace,
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

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        todo!("TODO(mvp) return Point type")
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        _program: &Program,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to call a host function")
    }
}

#[derive(Debug, Display)]
#[display("PatchAt {x:?} {y:?}")]
pub struct PatchAt {
    // The execution context to use.
    pub context: NodeId,
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
        vec![self.context, self.x, self.y]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Patch.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results::<I>(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[x] = lir_builder.get_node_results::<I>(program, self.x) else {
            panic!("expected node outputting x coordinate to be a single LIR value")
        };
        let &[y] = lir_builder.get_node_results::<I>(program, self.y) else {
            panic!("expected node outputting y coordinate to be a single LIR value")
        };
        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.patch_at,
            Box::new([ctx_ptr, x, y]),
        ));
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
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

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Float.into()
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_max_pxcor(
            program: &mut Program,
            _fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let &NodeKind::MaxPxcor(MaxPxcor { context }) = &program.nodes[my_node_id] else {
                return false;
            };

            // insert a node that gets the workspace pointer
            let workspace_ptr = program.nodes.insert(NodeKind::from(node::MemLoad {
                ptr: context,
                offset: offset_of!(CanonExecutionContext, workspace),
                ty: <*mut u8 as Reflect>::CONCRETE_TY,
            }));

            // insert a node that gets the the desired field
            program.nodes[my_node_id] = NodeKind::from(node::MemLoad {
                ptr: workspace_ptr,
                offset: offset_of!(Workspace, world)
                    + offset_of!(World, topology)
                    + OFFSET_TOPOLOGY_TO_MAX_PXCOR,
                ty: NlFloat::CONCRETE_TY,
            });

            true
        }
        Some(Box::new(lower_get_max_pxcor))
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

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Float.into()
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_max_pycor(
            program: &mut Program,
            _fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let &NodeKind::MaxPycor(MaxPycor { context }) = &program.nodes[my_node_id] else {
                return false;
            };

            // insert a node that gets the workspace pointer
            let workspace_ptr = program.nodes.insert(NodeKind::from(node::MemLoad {
                ptr: context,
                offset: offset_of!(CanonExecutionContext, workspace),
                ty: <*mut u8 as Reflect>::CONCRETE_TY,
            }));

            // insert a node that gets the the desired field
            program.nodes[my_node_id] = NodeKind::from(node::MemLoad {
                ptr: workspace_ptr,
                offset: offset_of!(Workspace, world)
                    + offset_of!(World, topology)
                    + OFFSET_TOPOLOGY_TO_MAX_PYCOR,
                ty: NlFloat::CONCRETE_TY,
            });

            true
        }
        Some(Box::new(lower_get_max_pycor))
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

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Float.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let mut args = Vec::new();
        args.extend(lir_builder.get_node_results::<I>(program, self.a));
        args.extend(lir_builder.get_node_results::<I>(program, self.b));

        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.euclidean_distance_no_wrap,
            args.into_boxed_slice(),
        ));
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
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

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Point.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // simply pass through the LIR values
        let Self { x, y } = self;
        let &[x] = lir_builder.get_node_results::<I>(program, *x) else {
            panic!("expected x value to be a single LIR value");
        };
        let &[y] = lir_builder.get_node_results::<I>(program, *y) else {
            panic!("expected y value to be a single LIR value");
        };
        lir_builder.node_to_lir.insert(my_node_id, smallvec![x, y]);
        Ok(())
    }
}
