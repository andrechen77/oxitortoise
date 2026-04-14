use std::collections::{BTreeMap, BTreeSet};

use crate::{
    hir::{self, Expr, ExprKind, NlAbstractTy, expr},
    mir::{MirType, MirTypeInfo},
    sim::{
        observer::GlobalsSchema,
        patch::{
            OptionPatchId, PatchFieldGroup, PatchFieldGroupElement, PatchSchema, PatchVarDesc,
        },
        topology::Point,
        turtle::{TurtleId, TurtleSchema},
        value::{NlBox, NlFloat, NlList, PackedAny},
    },
    util::{reflection::Reflect, rng::CanonRng},
    workspace::Workspace,
};

#[derive(Debug)]
pub struct TypeMapping {
    globals_schema: GlobalsSchema,
    turtle_schema: TurtleSchema,
    patch_schema: PatchSchema,
    workspace_ptr_ty: MirType,
    local_var_tys: BTreeMap<hir::LocalId, LocalVarMapping>,
    function_return_tys: BTreeMap<hir::FunctionId, MirType>,
}

#[derive(Debug)]
pub struct LocalVarMapping {
    ty: MirType,
    // TODO instead of a separate boolean field, we could encode the heap
    // allocation requirement in the type itself by having the type be a
    // rc-pointer to the type of the value
    /// Whether the local variable needs to be stored in the heap. This is true
    /// if the local variable is  captured by a closure that could outlive the
    /// stack frame while also being modified.
    heap: bool,
    /// Whether the local variable is captured by a closure.
    ///
    /// Used to determine if a variable must be stored on the stack.
    captured: bool,
}

impl TypeMapping {
    pub fn globals_schema(&self) -> &GlobalsSchema {
        &self.globals_schema
    }

    pub fn turtle_schema(&self) -> &TurtleSchema {
        &self.turtle_schema
    }

    pub fn patch_schema(&self) -> &PatchSchema {
        &self.patch_schema
    }

    pub fn workspace_ptr_ty(&self) -> MirType {
        self.workspace_ptr_ty.clone()
    }

    pub fn local_var_ty(&self, local_id: hir::LocalId) -> MirType {
        self.local_var_tys[&local_id].ty.clone()
    }

    pub fn function_return_ty(&self, fn_id: hir::FunctionId) -> MirType {
        self.function_return_tys[&fn_id].clone()
    }
}

pub fn make_type_mapping(hir: &hir::Program) -> TypeMapping {
    // stores all patch variables that are used in a diffuse command. these will
    // be separated into their own tables to improve cache locality during the
    // diffuse operation
    let mut patch_diffused = BTreeSet::new();

    // stores the types of local variables and whether they need to be stored
    // in the heap (captured by a closure that could outlives the stack frame)
    let mut local_var_tys = BTreeMap::new();

    // stores the types of function returns
    let mut function_return_tys = BTreeMap::new();

    // stores all function parameters, including closure parameters, so they can
    // all be mapped at the end
    let mut function_params = Vec::new();

    for (fn_id, function) in &hir.functions {
        let return_ty = mir_repr_simple(&function.return_ty);
        function_return_tys.insert(*fn_id, return_ty);
        for (param_id, param_decl) in &function.parameters {
            function_params.push((*param_id, param_decl));
        }
    }

    // iterate through the program and collect information about how each
    // variable is used
    for function_body in hir.function_bodies.values() {
        // visit the function body
        visit_expr(function_body, &mut patch_diffused, &mut local_var_tys, &mut function_params);
        fn visit_expr<'a>(
            expr_kind: &'a ExprKind,
            patch_diffused: &mut BTreeSet<PatchVarDesc>,
            local_var_tys: &mut BTreeMap<hir::LocalId, LocalVarMapping>,
            function_params: &mut Vec<(hir::LocalId, &'a hir::LocalDecl)>,
        ) {
            match expr_kind {
                ExprKind::Diffuse(expr::Diffuse { variable, .. }) => {
                    patch_diffused.insert(*variable);
                }
                ExprKind::Scope(expr::Scope { locals, .. }) => {
                    for (local_id, local_decl) in locals {
                        let ty = mir_repr_simple(&local_decl.ty);
                        // default to not being stored in the heap first. then
                        // any closures being visited that capture this local
                        // can set it to true.
                        let mapping = LocalVarMapping { ty, heap: false, captured: false };
                        local_var_tys.insert(*local_id, mapping);
                    }
                }
                ExprKind::Closure(expr::Closure { captures, parameters, .. }) => {
                    for capture in captures {
                        let mapping = local_var_tys.get_mut(capture).expect(
                            "captured variable must have been previously defined by a scope",
                        );
                        mapping.captured = true;
                        // conservatively estimate that if a variable is captured,
                        // it might outlive the stack frame. we could make this
                        // more precise by actually checking how the closure is used
                        mapping.heap = true;
                    }

                    for (param_id, param_decl) in parameters {
                        function_params.push((*param_id, param_decl));
                    }
                }
                _ => {} // do nothing
            }
            expr_kind.visit_children(|child_expr_kind| {
                visit_expr(child_expr_kind, patch_diffused, local_var_tys, function_params)
            });
        }
    }

    // make the globals schema
    let custom_fields = hir
        .global_vars
        .iter()
        .map(|var| {
            let concrete_ty = mir_repr_simple(&var.ty);
            let concrete_ty = concrete_ty
                .static_ty
                .expect("we cannot handled dynamically defined types in globals (yet)");
            (var.name.clone(), concrete_ty)
        })
        .collect();
    let globals_schema = GlobalsSchema::new_with_custom_fields(custom_fields);

    let turtle_schema = TurtleSchema::default();

    // make the patch schema. algorithm: the base data and position always goes
    // in the first field group. for each other variable, if it is diffused, it
    // goes in its own field group without an occupancy bitfield, otherwise it
    // goes in the first field group
    let mut patch_field_groups = vec![PatchFieldGroup {
        avoid_occupancy_bitfield: false,
        fields: vec![PatchFieldGroupElement::BaseData],
    }];
    if patch_diffused.contains(&PatchVarDesc::Pcolor) {
        patch_field_groups.push(PatchFieldGroup {
            avoid_occupancy_bitfield: true,
            fields: vec![PatchFieldGroupElement::Pcolor],
        });
    } else {
        patch_field_groups[0].fields.push(PatchFieldGroupElement::Pcolor);
    }
    for (var_id, var) in hir.custom_patch_vars.iter().enumerate() {
        let var_desc = PatchVarDesc::Custom(var_id);

        let ty = mir_repr_simple(&var.ty);
        let ty =
            ty.static_ty.expect("we cannot handled dynamically defined types in globals (yet)");

        if patch_diffused.contains(&var_desc) {
            patch_field_groups.push(PatchFieldGroup {
                avoid_occupancy_bitfield: true,
                fields: vec![PatchFieldGroupElement::Custom { name: var.name.clone(), ty }],
            });
        } else {
            patch_field_groups[0]
                .fields
                .push(PatchFieldGroupElement::Custom { name: var.name.clone(), ty });
        }
    }
    let patch_schema = PatchSchema::new_with_field_groups(patch_field_groups);

    let workspace_ptr_ty = make_workspace_ptr_type(&globals_schema, &turtle_schema, &patch_schema);

    for (param_id, param_decl) in function_params {
        let ty = if param_decl.ty == NlAbstractTy::Workspace {
            workspace_ptr_ty.clone()
        } else {
            mir_repr_simple(&param_decl.ty)
        };
        let mapping = LocalVarMapping { ty, heap: false, captured: false };
        local_var_tys.insert(param_id, mapping);
    }

    TypeMapping {
        globals_schema,
        turtle_schema,
        patch_schema,
        workspace_ptr_ty,
        local_var_tys,
        function_return_tys,
    }
}

fn make_workspace_ptr_type(
    globals_schema: &GlobalsSchema,
    turtle_schema: &TurtleSchema,
    patch_schema: &PatchSchema,
) -> MirType {
    MirTypeInfo::ptr_to(Workspace::mir_type_from_schemas(
        globals_schema,
        turtle_schema,
        patch_schema,
    ))
}

pub fn mir_repr_simple(abstract_ty: &NlAbstractTy) -> MirType {
    match abstract_ty {
        NlAbstractTy::Workspace => {
            unimplemented!("workspace type cannot be lowered in a simple manner")
        }
        NlAbstractTy::Rng => MirTypeInfo::ptr_to(CanonRng::mir_type()),
        NlAbstractTy::Unit => <()>::mir_type(),
        NlAbstractTy::NlTop => PackedAny::mir_type(),
        NlAbstractTy::Bottom => unimplemented!("bottom type has no concrete representation"),
        NlAbstractTy::Color => NlFloat::mir_type(),
        NlAbstractTy::Float => NlFloat::mir_type(),
        NlAbstractTy::Boolean => bool::mir_type(),
        NlAbstractTy::String => todo!(),
        NlAbstractTy::Point => Point::mir_type(),
        NlAbstractTy::Agent => PackedAny::mir_type(),
        NlAbstractTy::Patch => OptionPatchId::mir_type(),
        NlAbstractTy::Turtle => TurtleId::mir_type(),
        NlAbstractTy::Link => todo!(""),
        NlAbstractTy::Agentset { agent_type: _ } => todo!(""),
        // If a type is just "nobody", then it is inhabited by only one
        // value and therefore holds no data. Operations that take the
        // nobody value as an operand typically see it as an inhabitant of
        // some other type, e.g. nobody as a patch id, or nobody as a turtle
        // id. This is why "nobody" just by itself has no concrete
        // representation.
        NlAbstractTy::Nobody => unimplemented!("nobody type has no concrete representation"),
        NlAbstractTy::Closure(_) => todo!(),
        NlAbstractTy::List { element_ty } if **element_ty == NlAbstractTy::NlTop => {
            <NlBox<NlList>>::mir_type()
        }
        NlAbstractTy::List { element_ty: _ } => todo!(),
    }
}
