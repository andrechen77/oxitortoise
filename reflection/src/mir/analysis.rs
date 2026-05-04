use std::collections::BTreeSet;

use crate::mir::{
    Block, CtrlFlowConstruct, ElementaryStatement, Function, IfElse, LocalId, Loop, PlaceOperand,
    Projection, Statement,
};

/// Obtains all the local variables of a function that have their address taken.
pub fn address_taken_locals(function: &Function) -> BTreeSet<LocalId> {
    // a local has its address taken if a place projected without a deref
    // projection is used as a borrow operand
    let mut stack_locals = BTreeSet::new();
    visit_operands(&function.body, &mut |operand| {
        // if the operand is not a borrow, then this does not require a stack
        // allocation
        let PlaceOperand::Borrow(place) = operand else {
            return;
        };

        // if the operand has any deref projections, then this does not require
        // a stack allocation
        for projection in &place.projections {
            if projection == &Projection::Deref {
                return;
            }
        }

        // if we get here, that means this operand requires taking the address
        // of a local variable
        stack_locals.insert(place.local);
    });

    stack_locals
}

fn visit_operands(statement: &Statement, f: &mut impl FnMut(&PlaceOperand)) {
    match statement {
        Statement::CtrlFlow(CtrlFlowConstruct::Block(Block { label: _, statements })) => {
            for statement in statements {
                visit_operands(statement, f);
            }
        }
        Statement::CtrlFlow(CtrlFlowConstruct::IfElse(IfElse { condition: _, then, r#else })) => {
            visit_operands(then, f);
            visit_operands(r#else, f);
        }
        Statement::CtrlFlow(CtrlFlowConstruct::Loop(Loop { num_repetitions: _, body })) => {
            visit_operands(body, f);
        }
        Statement::Elementary(ElementaryStatement::Assign { dst: _, op }) => {
            for operand in op.operands() {
                f(operand);
            }
        }
        Statement::Elementary(ElementaryStatement::Break { target: _ }) => {}
        Statement::Elementary(ElementaryStatement::Drop { src: _ }) => {}
    }
}
