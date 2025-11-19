//! The `distancexy` operation.

use derive_more::derive::Display;

use crate::{
    mir::{
        Function, FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform, Nodes,
        Program, node,
    },
    sim::{patch::PatchVarDesc, turtle::TurtleVarDesc},
};

#[derive(Debug, Display)]
#[display("Distancexy {x:?} {y:?}")]
pub struct Distancexy {
    pub context: NodeId,
    /// The agent to get the distance from.
    pub agent: NodeId,
    /// The x coordinate.
    pub x: NodeId,
    /// The y coordinate.
    pub y: NodeId,
}

impl Node for Distancexy {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.agent, self.x, self.y]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }

    fn peephole_transform(
        &self,
        program: &Program,
        fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        // if the agent is a turtle
        let function = program.functions[fn_id].borrow();
        let nodes = function.nodes.borrow();
        let agent_type = nodes[self.agent].output_type(program, &function, &nodes);

        Some(Box::new(move |program, fn_id, my_node_id| {
            decompose_distancexy(program, fn_id, my_node_id, agent_type)
        }))
    }
}

fn decompose_distancexy(
    program: &Program,
    fn_id: FunctionId,
    my_node_id: NodeId,
    agent_type: MirTy,
) -> bool {
    let function = program.functions[fn_id].borrow();
    let mut nodes = function.nodes.borrow_mut();

    let &Distancexy { context, agent, x, y } = (&nodes[my_node_id]).try_into().unwrap();

    // add a node to get the location of the turtle
    let agent_pos = match agent_type {
        MirTy::Abstract(NlAbstractTy::Turtle) => nodes.insert(NodeKind::from(node::GetTurtleVar {
            context,
            turtle: agent,
            var: TurtleVarDesc::Pos,
        })),
        MirTy::Abstract(NlAbstractTy::Patch) => nodes.insert(NodeKind::from(node::GetPatchVar {
            context,
            patch: agent,
            var: PatchVarDesc::Pos,
        })),
        _ => todo!("TODO(mvp) decompose in case of link or any"),
    };

    // add a node to construct a point from the x and y coordinates
    let reference_pos = nodes.insert(NodeKind::from(node::PointConstructor { x, y }));

    // calculate the distance between the two points
    nodes[my_node_id] =
        NodeKind::from(node::EuclideanDistanceNoWrap { a: agent_pos, b: reference_pos });

    true
}
