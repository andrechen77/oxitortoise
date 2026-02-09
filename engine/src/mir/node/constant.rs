//! Nodes that represent constant/literal values.

use std::fmt::{self, Write};

use lir::smallvec::smallvec;
use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::value::UnpackedDynBox,
};

#[derive(Debug)]
pub struct Constant {
    pub value: UnpackedDynBox,
}

impl Node for Constant {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        match self.value {
            UnpackedDynBox::Float(_) => NlAbstractTy::Float,
            UnpackedDynBox::Bool(_) => NlAbstractTy::Boolean,
            UnpackedDynBox::Nobody => NlAbstractTy::Nobody,
            _ => todo!("TODO(mvp) include all other variants (doesn't handle {:?})", self.value),
        }
        .into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        _program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let _ = my_node_id;
        match self.value {
            UnpackedDynBox::Float(value) => {
                let pc = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Value::F64(value)));
                lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
                Ok(())
            }
            _ => todo!("TODO(mvp) include other variants"),
        }
    }

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out)
            .add_struct("Constant", |p| p.add_field("value", |p| write!(p, "{:?}", self.value)))
    }
}

#[derive(Debug)]
pub struct ListLiteral {
    pub items: Vec<NodeId>,
}

impl Node for ListLiteral {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        self.items.iter().map(|&id| ("item", id)).collect()
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::List { element_ty: Box::new(NlAbstractTy::Top) }.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // keep track of which instruction outputs the most recent version of the list
        let mut current_list;

        // insert an instruction to create an empty list
        let pc_new_list = lir_builder.push_lir_insn(lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.list_new,
            Box::from([]),
        ));
        current_list = lir::ValRef(pc_new_list, 0);

        // insert instructions to push elements to the list
        for item in &self.items {
            let &[item] = lir_builder.get_node_results::<I>(program, *item) else {
                todo!("TODO(mvp) handle multi-register values");
            };
            let pc_push = lir_builder.push_lir_insn(lir::generate_host_function_call(
                I::HOST_FUNCTION_TABLE.list_push,
                Box::from([current_list, item]),
            ));
            current_list = lir::ValRef(pc_push, 0);
        }

        // make the finished list available as a LIR value
        lir_builder.node_to_lir.insert(my_node_id, smallvec![current_list]);

        Ok(())
    }

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("ListLiteral", |_| Ok(()))
    }
}
