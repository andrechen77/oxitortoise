// Foundational expressions such as control flow and scope are defined here.
// More specialized expression kinds are defined in their respective submodules.

use std::{
    collections::BTreeMap,
    fmt::{self, Write},
};

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, Label, LocalDecl, LocalId, NlAbstractTy, Program},
    mir,
};

mod agent_var;
mod agentset;
mod arith_op;
mod ask;
mod clear;
mod closure;
mod color;
mod constant;
mod create_agent;
mod diffuse;
mod distancexy;
mod list_set_ops;
mod local_var;
mod rand;
mod set_default_shape;
mod ticks;
mod topology;
mod turtle_movement;
mod user_fn;

pub use agent_var::*;
pub use agentset::*;
pub use arith_op::*;
pub use ask::*;
pub use clear::*;
pub use closure::*;
pub use color::*;
pub use constant::*;
pub use create_agent::*;
pub use diffuse::*;
pub use distancexy::*;
pub use list_set_ops::*;
pub use local_var::*;
use pretty_print::PrettyPrinter;
pub use rand::*;
pub use set_default_shape::*;
pub use ticks::*;
pub use topology::*;
pub use turtle_movement::*;
pub use user_fn::*;

/// An expression that defines a set of mutable local variables that can be
/// written and read in the evaluation of an inner expression.
#[derive(Debug, Clone)]
pub struct Scope {
    pub locals: BTreeMap<LocalId, LocalDecl>,
    pub inner: Box<ExprKind>,
}

impl Expr for Scope {
    fn output_type(&self, program: &super::Program) -> super::NlAbstractTy {
        self.inner.output_type(program)
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.inner);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        for (local_id, decl) in &self.locals {
            let mir_local_decl =
                mir::LocalDecl { debug_name: decl.debug_name.clone(), ty: decl.ty.repr() };
            let (mir_local_id, _) = builder.mir.create_local(mir_local_decl);
            builder.translator.locals.insert(*local_id, mir_local_id);
        }
        self.inner.write_mir_execution(builder, local_out);

        // the scope does not remove the locals after the inner expression is
        // evaluated. At the time this code was written, this did not seem to be
        // an issue.
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, program: &Program) -> fmt::Result {
        let Scope { locals, inner } = self;
        write!(p, "with (")?;
        p.indented(|p| {
            // add local declarations
            for (local_id, decl) in locals {
                p.line()?;
                write!(
                    p,
                    "{:?} {}: {},",
                    local_id,
                    decl.debug_name.as_ref().map_or("", |n| n.as_ref()),
                    decl.ty
                )?;
            }
            Ok(())
        })?;
        p.line()?;
        write!(p, ") ")?;
        inner.pretty_print(p, program)?;
        Ok(())
    }
}

/// An expression that represents multiple expressions evaluated in order.
/// The only way to exit a block is to break out of it; "falling through" the
/// end of the statement sequence is not allowed.
#[derive(Debug, Clone)]
pub struct Block {
    pub label: Label,
    pub statements: Vec<ExprKind>,
}

impl Expr for Block {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        // This does not really need to be an option but since we move out of it
        // for a split second to do the join, the compiler requires it.
        // Logically it is never None.
        let mut output_type = Some(NlAbstractTy::Bottom);
        self.visit_children(|expr| {
            if let ExprKind::Break(Break { target, value }) = expr
                && *target == self.label
            {
                let break_ty = value.output_type(program);
                output_type = Some(output_type.take().unwrap().join(break_ty));
            }
        });
        output_type.unwrap()
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        for stmt in &self.statements {
            visitor(stmt);
        }
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        let label = builder.mir.create_label();

        // make this label visible to child expressions
        builder.translator.ctrl_flow_constructs.insert(self.label, (label, local_out));

        // translate the statements in the block
        let (statements, _) = builder.with_inner_statement_seq(|builder| {
            for expr in &self.statements {
                builder.translate_expr(expr);
            }
        });

        // create a block statement with the translated statements
        let block = mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::Block(mir::Block {
            label: Some(label),
            statements,
        }));
        builder.mir.add_statement(block);
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, program: &Program) -> fmt::Result {
        let Block { label, statements } = self;
        write!(p, "{}: {{", label)?;
        p.indented(|p| {
            for statement in statements {
                p.line()?;
                statement.pretty_print(p, program)?;
            }
            Ok(())
        })?;
        p.line()?;
        write!(p, "}}")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IfElse {
    pub condition: Box<ExprKind>,
    pub then: Box<ExprKind>,
    pub r#else: Box<ExprKind>,
}

impl Expr for IfElse {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        let then_ty = self.then.output_type(program);
        let else_ty = self.r#else.output_type(program);
        then_ty.join(else_ty)
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.condition);
        visitor(&self.then);
        visitor(&self.r#else);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, local_out: mir::LocalId) {
        let condition = builder.translate_expr(&self.condition);
        let (then_stmts, _) = builder.with_inner_statement_seq(|builder| {
            self.then.write_mir_execution(builder, local_out);
        });
        let then = mir::consolidate_statements(then_stmts);
        let (else_stmts, _) = builder.with_inner_statement_seq(|builder| {
            self.r#else.write_mir_execution(builder, local_out);
        });
        let r#else = mir::consolidate_statements(else_stmts);

        let if_else = mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::IfElse(mir::IfElse {
            condition: condition.place,
            then: Box::new(then),
            r#else: Box::new(r#else),
        }));
        builder.mir.add_statement(if_else);
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, program: &Program) -> fmt::Result {
        let IfElse { condition, then, r#else } = self;
        write!(p, "if ")?;
        condition.pretty_print(p, program)?;
        write!(p, " ")?;
        then.pretty_print(p, program)?;
        write!(p, " else ")?;
        r#else.pretty_print(p, program)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Break {
    pub target: Label,
    pub value: Box<ExprKind>,
}

impl Expr for Break {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        // a break diverges, it never returns
        NlAbstractTy::Bottom
    }

    fn visit_children(&self, visitor: impl FnMut(&ExprKind)) {
        self.value.visit_children(visitor);
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder, break_local_out: mir::LocalId) {
        // ignore this value, a break never returns
        let _ = break_local_out;

        let (target_label, target_local_out) =
            builder.translator.ctrl_flow_constructs[&self.target];

        // assign the break's value to the target local instead
        self.value.write_mir_execution(builder, target_local_out);

        // and add the break statement
        builder.mir.add_statement(mir::Statement::Elementary(mir::ElementaryStatement::Break {
            target: target_label,
        }));
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let Break { target, value } = self;
        write!(p, "break {} ", target)?;
        value.pretty_print(p, program)?;
        Ok(())
    }
}
