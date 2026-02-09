//! The `distancexy` operation.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    mir::{FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform, Program, node},
    sim::{patch::PatchVarDesc, turtle::TurtleVarDesc},
};

#[derive(Debug)]
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

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![("agent", self.agent), ("x", self.x), ("y", self.y)]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Float.into()
    }

    fn peephole_transform(
        &self,
        program: &Program,
        fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        // if the agent is a turtle
        let agent_type = program.nodes[self.agent].output_type(program, fn_id);

        Some(Box::new(move |program, fn_id, my_node_id| {
            decompose_distancexy(program, fn_id, my_node_id, agent_type)
        }))
    }

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("Distancexy", |_| Ok(()))
    }
}

fn decompose_distancexy(
    program: &mut Program,
    _fn_id: FunctionId,
    my_node_id: NodeId,
    agent_type: MirTy,
) -> bool {
    let &NodeKind::Distancexy(Distancexy { context, agent, x, y }) = &program.nodes[my_node_id]
    else {
        return false;
    };

    // add a node to get the location of the turtle
    let agent_pos = match agent_type.abstr.expect("agent type must have an abstract type") {
        NlAbstractTy::Turtle => program.nodes.insert(NodeKind::from(node::GetTurtleVar {
            context,
            turtle: agent,
            var: TurtleVarDesc::Pos,
        })),
        NlAbstractTy::Patch => program.nodes.insert(NodeKind::from(node::GetPatchVar {
            context,
            patch: agent,
            var: PatchVarDesc::Pos,
        })),
        _ => todo!("TODO(mvp) decompose in case of link or any"),
    };

    // add a node to construct a point from the x and y coordinates
    let reference_pos = program.nodes.insert(NodeKind::from(node::PointConstructor { x, y }));

    // calculate the distance between the two points
    program.nodes[my_node_id] =
        NodeKind::from(node::EuclideanDistanceNoWrap { a: agent_pos, b: reference_pos });

    true
}
