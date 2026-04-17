//! Nodes to represent closures.

use std::{
    alloc::Layout,
    collections::BTreeMap,
    fmt::{self, Write},
    sync::Arc,
};

use pretty_print::PrettyPrinter;
use tracing::{trace, warn};

use crate::{
    exec::jit::JitCallback,
    hir::{
        ClosureType, Expr, ExprKind, HirToMirFnBuilder, LocalDecl, LocalId, NameContext,
        NlAbstractTy,
        build_mir::{self, HirToMirFnTranslator, translate_expr},
        ty::NlAbstractTyAtom,
    },
    mir,
    sim::value::BoxedAny,
    util::{rc::create_erased_rc, reflection::Reflect},
};

#[derive(Debug, Clone)]
pub struct Closure {
    /// All the local variables that are captured by the closure.
    pub captures: Vec<LocalId>,
    pub parameters: BTreeMap<LocalId, LocalDecl>,
    /// The body of the closure. This is the part of the closure with deferred
    /// evaluation.
    pub body: Box<ExprKind>,
}

impl Expr for Closure {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let return_ty = self.body.output_type(names);

        NlAbstractTyAtom::Closure(ClosureType {
            arg_tys: self.parameters.values().map(|decl| decl.ty.clone()).collect(),
            return_ty: Arc::new(return_ty),
        })
        .into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.body);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.body.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Closure { captures, parameters, body } = self;
        p.add_struct("closure", |p| {
            p.add_field_with("captures", |p| {
                p.add_list(captures.iter(), |p, capture| {
                    let name = names
                        .lookup_local_var(*capture)
                        .map(|decl| decl.debug_name.as_ref())
                        .unwrap_or("?");
                    write!(p, "{}#{}", capture.0, name)
                })
            })?;
            p.add_field_with("parameters", |p| {
                p.add_list(parameters.iter(), |p, (local_id, decl)| {
                    write!(p, "{}#{}: {:?}", local_id.0, decl.debug_name, decl.ty)
                })
            })?;
            p.add_field_with("body", |p| body.pretty_print(p, names.with_locals(parameters)))?;
            Ok(())
        })
    }
}

impl Closure {
    pub fn write_mir_execution_with_static_types<Arg, Ret>(
        &self,
        builder: &mut HirToMirFnBuilder,
    ) -> mir::LocalId
    where
        JitCallback<'static, Arg, Ret>: Reflect,
    {
        let mut inner_translator = HirToMirFnTranslator::default();

        let (env, env_ty) = if self.captures.is_empty() {
            let env_ty = <*mut u8>::mir_type();
            let env = builder.mir.add_operation_with_decl(
                mir::LocalDecl { debug_name: None, ty: env_ty.clone() },
                mir::Operation::Const { value: BoxedAny::new(std::ptr::null_mut::<u8>()) },
            );
            (env, env_ty)
        } else {
            // create an anonymous struct with the captures, and update
            let captured_places: Vec<mir::Place> = self
                .captures
                .iter()
                .map(|cap| builder.local_translator.locals[cap].clone())
                .collect();
            let (env, env_ty) = mir_create_anon_struct(builder, &captured_places);
            // TODO update the translator to map the captures to the new place
            (env, env_ty)
        };

        let call_fn = {
            let mut mir_fn_builder = builder.mir.create_another_function();
            build_mir::translate_function(
                builder.hir_names,
                builder.type_mapping,
                &mut mir_fn_builder,
                &mut inner_translator,
                builder.user_fn_translator,
                &self.parameters,
                &self.body,
            );
            mir_fn_builder.finish()
        };

        let drop_fn = {
            let mut mir_fn_builder = builder.mir.create_another_function();
            let ptr_to_val =
                mir_fn_builder.create_parameter(mir::LocalDecl { debug_name: None, ty: env_ty });
            if !self.captures.is_empty() {
                let _ = ptr_to_val;
                todo!("add statements to the function to drop the value")
            }
            mir_fn_builder.set_return(mir_fn_builder.unit_local());
            mir_fn_builder.finish()
        };

        // create and initialize a local variable to hold the closure
        let result = <JitCallback<'static, Arg, Ret>>::mir_initialize(
            builder,
            env.place(),
            call_fn,
            drop_fn,
        );
        result
    }

    /// Calls the closure in place with the provided arguments.
    pub fn write_mir_inline_call(
        &self,
        builder: &mut HirToMirFnBuilder,
        args: &[mir::Place],
    ) -> Option<mir::LocalId> {
        let Self { captures, parameters, body } = self;

        let mut inner_translator = HirToMirFnTranslator::default();
        for &captured_local_id in captures {
            let outer_fn_place = builder.local_translator.locals[&captured_local_id].clone();
            inner_translator.locals.insert(captured_local_id, outer_fn_place);
        }
        assert!(parameters.len() == args.len(), "number of parameters and arguments must match");
        for ((param_id, param_decl), arg_place) in parameters.iter().zip(args.iter()) {
            // assert that the parameter type matches
            let arg_ty = builder.mir.type_of_place(arg_place);
            let param_ty = builder.type_mapping.local_var_ty(*param_id);
            if !param_ty.is_supertype_of(arg_ty) {
                warn!(
                    "parameter {:?} has type {:?} but argument has type {:?}",
                    *param_id, param_ty, arg_ty
                );
            }

            trace!(
                "inserting parameter {:?} with type {:?} into inner translator",
                *param_id, param_decl.ty
            );
            inner_translator.locals.insert(*param_id, arg_place.clone());
        }

        trace!("inner translator: {:?}", inner_translator);

        let result = builder
            .with_local_translator(&mut inner_translator, |builder| translate_expr(builder, body));

        // TODO how to drop the arguments

        result
    }
}

/// Creates a heap-allocated anonymous struct with the provided values. The
/// values are cloned into the struct and a pointer to the heap allocation is
/// returned.
fn mir_create_anon_struct(
    builder: &mut HirToMirFnBuilder,
    values: &[mir::Place],
) -> (mir::LocalId, mir::MirType) {
    // aggregate the fields together to find the total layout as well as offsets
    // and types of each field
    let mut total_layout = Layout::new::<()>();
    let mut fields = Vec::new();
    for value in values {
        let field_ty = builder.mir.type_of_place(&value).clone();
        let field_layout = field_ty.layout();
        let (new_layout, offset) = total_layout
            .extend(field_layout)
            .expect("if the layout overflows we have bigger problems");
        total_layout = new_layout;
        fields.push((offset, field_ty));
    }

    // now put it all together to create a definition of the struct type
    let struct_ty = mir::MirType::new_struct(total_layout, fields.clone());

    // define a function that will drop all the values in the struct
    let drop_fn = {
        let mut drop_fn_builder = builder.mir.create_another_function();

        // add the parameter: a pointer to the struct being dropped
        let param_ty = mir::MirType::ref_to(struct_ty.clone());
        let param =
            drop_fn_builder.create_parameter(mir::LocalDecl { debug_name: None, ty: param_ty });

        // add drop statements for each field
        for (offset, _) in &fields {
            let field_place = param.place().proj_field(*offset);
            drop_fn_builder.add_statement(mir::Statement::Elementary(
                mir::ElementaryStatement::Drop { src: field_place },
            ));
        }

        drop_fn_builder.finish()
    };

    // call a host function to allocate the struct on the heap
    let size: u32 = total_layout.size().try_into().unwrap();
    let align: u32 = total_layout.align().try_into().unwrap();
    let size =
        builder.mir.add_operation(None, mir::Operation::Const { value: BoxedAny::new(size) });
    let align =
        builder.mir.add_operation(None, mir::Operation::Const { value: BoxedAny::new(align) });
    let drop_fn =
        builder.mir.add_operation(None, mir::Operation::FunctionPtr { function: drop_fn });
    let erased_rc = builder.mir.add_operation(
        None,
        mir::Operation::CallHostFunction {
            function: &create_erased_rc::FN_INFO,
            args: vec![
                mir::PlaceOperand::Move(size),
                mir::PlaceOperand::Move(align),
                mir::PlaceOperand::Move(drop_fn),
            ],
        },
    );

    // initialize the fields of the struct with clones of the values
    for (src_place, (offset, _ty)) in values.iter().zip(fields.iter()) {
        let dst = erased_rc.place().proj_deref().proj_field(*offset);
        builder.mir.clone_to_uninit(src_place.clone(), dst);
    }

    (erased_rc, struct_ty)
}
