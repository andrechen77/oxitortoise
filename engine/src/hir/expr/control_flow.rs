// Foundational expressions such as control flow and scope are defined here.

use std::{
    collections::BTreeMap,
    fmt::{self, Write},
};

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, Label, LocalDecl, LocalId, NameContext, NlAbstractTy,
    },
    mir,
};

use pretty_print::PrettyPrinter;

/// An expression that defines a set of mutable local variables that can be
/// written and read in the evaluation of an inner expression.
#[derive(Debug, Clone)]
pub struct Scope {
    pub locals: BTreeMap<LocalId, LocalDecl>,
    pub inner: Box<ExprKind>,
}

impl Expr for Scope {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        self.inner.output_type(names)
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.inner);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.inner.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        for (local_id, decl) in &self.locals {
            let ty = builder.type_mapping.local_var_ty(*local_id);
            let mir_local_id = builder
                .mir
                .create_local(mir::LocalDecl { debug_name: Some(decl.debug_name.clone()), ty });
            builder.translator.locals.insert(*local_id, mir_local_id);
        }
        // the scope does not remove the locals from the translator after the inner expression is
        // evaluated. At the time this code was written, this did not seem to be
        // an issue.

        let value =
            builder.with_locals(&self.locals, |builder| self.inner.write_mir_execution(builder));

        for local_id in self.locals.keys() {
            let mir_local_id = builder.translator.locals[local_id];
            let ty = builder.mir.type_of_place(&mir_local_id.place());

            // only insert a drop instruction if the local has an associated
            // statically known type, because it if doesn't have a statically
            // known type then it probably doesn't have a destructor
            if let Some(static_ty) = ty.static_ty
                && static_ty.drop_fn.is_some()
            {
                builder.mir.add_statement(mir::Statement::Elementary(
                    mir::ElementaryStatement::Drop { src: mir_local_id.place() },
                ));
            }
        }

        value
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, names: NameContext) -> fmt::Result {
        let Scope { locals, inner } = self;
        write!(p, "with ")?;
        p.add_list(locals.iter(), |p, (local_id, decl)| {
            write!(p, "{}#{}: {}", local_id.0, decl.debug_name, decl.ty)
        })?;
        write!(p, " do ")?;
        inner.pretty_print(p, names.with_locals(&self.locals))?;
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
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        // This does not really need to be an option but since we move out of it
        // for a split second to do the join, the compiler requires it.
        // Logically it is never None.
        let mut output_type = Some(NlAbstractTy::Bottom);
        self.visit_children(|expr| {
            if let ExprKind::Break(Break { target, value }) = expr
                && *target == self.label
            {
                let break_ty = value.output_type(names);
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

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        for stmt in &mut self.statements {
            visitor(stmt);
        }
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let label = builder.mir.create_label();

        // make this label visible to child expressions
        builder.translator.ctrl_flow_constructs.insert(self.label, (label, None));

        // translate the statements in the block
        let (statements, _) = builder.with_inner_statement_seq(|builder| {
            for expr in &self.statements {
                let result = expr.write_mir_execution(builder)?;
                let ty = builder.mir.type_of_place(&result.place());
                if let Some(static_ty) = ty.static_ty
                    && static_ty.drop_fn.is_some()
                {
                    builder.mir.add_statement(mir::Statement::Elementary(
                        mir::ElementaryStatement::Drop { src: result.place() },
                    ));
                }
            }
            Some(())
        });

        // create a block statement with the translated statements
        let block = mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::Block(mir::Block {
            label: Some(label),
            statements,
        }));
        builder.mir.add_statement(block);

        // get the return place of the statement
        let (_, return_place) = builder.translator.ctrl_flow_constructs[&self.label];
        return_place
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, names: NameContext) -> fmt::Result {
        let Block { label, statements } = self;
        write!(p, "{}: {{", label)?;
        p.indented(|p| {
            for statement in statements {
                p.line()?;
                statement.pretty_print(p, names)?;
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
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let then_ty = self.then.output_type(names);
        let else_ty = self.r#else.output_type(names);
        then_ty.join(else_ty)
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.condition);
        visitor(&self.then);
        visitor(&self.r#else);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.condition.as_mut());
        visitor(self.then.as_mut());
        visitor(self.r#else.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let condition = self.condition.write_mir_execution(builder)?.place();

        let (then_stmts, then_out) =
            builder.with_inner_statement_seq(|builder| self.then.write_mir_execution(builder));
        let then_ty = then_out.as_ref().map(|t| builder.mir.type_of_place(&t.place()));
        let then_stmt = Box::new(mir::consolidate_statements(then_stmts));

        let (else_stmts, total_out) = builder.with_inner_statement_seq(|builder| {
            let else_out = self.r#else.write_mir_execution(builder);
            let else_ty = else_out.as_ref().map(|t| builder.mir.type_of_place(&t.place()));
            assert!(
                then_ty.is_none() || else_ty.is_none() || then_ty == else_ty,
                "then and else branches must have compatible types"
            );

            match (then_out, else_out) {
                (Some(then_out), Some(else_out)) => {
                    if then_out != else_out {
                        // move the else value into the then place
                        builder.mir.add_operation_with_dst(
                            then_out.place(),
                            mir::Operation::Operand(mir::PlaceOperand::Move(else_out)),
                        )
                    }
                    Some(then_out)
                }
                (Some(then_out), None) => Some(then_out),
                (None, Some(else_out)) => Some(else_out),
                (None, None) => None,
            }
        });
        let else_stmt = Box::new(mir::consolidate_statements(else_stmts));

        let if_else = mir::Statement::CtrlFlow(mir::CtrlFlowConstruct::IfElse(mir::IfElse {
            condition,
            then: then_stmt,
            r#else: else_stmt,
        }));
        builder.mir.add_statement(if_else);

        total_out
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, names: NameContext) -> fmt::Result {
        let IfElse { condition, then, r#else } = self;
        write!(p, "if ")?;
        condition.pretty_print(p, names)?;
        write!(p, " ")?;
        then.pretty_print(p, names)?;
        write!(p, " else ")?;
        r#else.pretty_print(p, names)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Break {
    pub target: Label,
    pub value: Box<ExprKind>,
}

impl Expr for Break {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        // a break diverges, it never returns
        NlAbstractTy::Bottom
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(self.value.as_ref());
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.value.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let value = self.value.write_mir_execution(builder)?;

        let (target_label, target_local_out) =
            builder.translator.ctrl_flow_constructs.get_mut(&self.target).unwrap();
        let target_local_out = *target_local_out.get_or_insert_with(|| {
            let ty = builder.mir.type_of_place(&value.place());
            builder.mir.create_local(mir::LocalDecl { debug_name: None, ty })
        });

        // assign the break's value to the target local instead
        builder.mir.add_operation_with_dst(
            target_local_out.place(),
            mir::Operation::Operand(mir::PlaceOperand::Move(value)),
        );

        // and add the break statement
        builder.mir.add_statement(mir::Statement::Elementary(mir::ElementaryStatement::Break {
            target: *target_label,
        }));

        // a break never returns
        None
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Break { target, value } = self;
        write!(p, "break {} ", target)?;
        value.pretty_print(p, names)?;
        Ok(())
    }
}
