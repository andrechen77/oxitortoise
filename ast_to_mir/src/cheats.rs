use std::collections::{HashMap, HashSet};

use engine::mir::StatementKind::{IfElse, Node as NodeStatement, Repeat, Return, Stop};
use engine::mir::node::SetLocalVar;

use engine::{
    mir::{
        FunctionId, LocalId, NlAbstractTy, Node, NodeId, NodeKind, Program, StatementBlock,
        StatementKind,
    },
    sim::{
        agent_schema::GlobalsSchema,
        patch::{PatchSchema, PatchVarDesc},
        turtle::TurtleSchema,
        turtle::TurtleVarDesc,
    },
    slotmap::SecondaryMap,
};
use serde::Deserialize;

use crate::{FnInfo, GlobalScope};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheatVarType {
    Float,
    Boolean,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CheatGlobalsSchema {
    Default,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CheatPatchSchema {
    Default,
    Ctor(CheatPatchSchemaCtor),
}

#[derive(Deserialize, Debug)]
pub struct CheatPatchSchemaCtor {
    pcolor_buffer_idx: u8,
    custom_fields: Vec<u8>,
    avoid_occupancy_bitfield: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CheatTurtleSchema {
    Default,
    Ctor(CheatTurtleSchemaCtor),
}

#[derive(Deserialize, Debug)]
pub struct CheatTurtleSchemaCtor {
    heading_buffer_idx: u8,
    position_buffer_idx: u8,
    custom_fields: Vec<u8>,
    avoid_occupancy_bitfield: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CheatSelfParamType {
    Patch,
    Turtle,
}

#[derive(Deserialize, Debug)]
pub struct CheatFunctionInfo {
    self_param_type: Option<CheatSelfParamType>,
}

#[derive(Deserialize)]
pub struct Cheats {
    globals_var_types: Option<HashMap<String, CheatVarType>>,
    globals_schema: Option<CheatGlobalsSchema>,
    patch_var_types: Option<HashMap<String, CheatVarType>>,
    patch_schema: Option<CheatPatchSchema>,
    turtle_schema: Option<CheatTurtleSchema>,
    turtle_var_types: Option<HashMap<String, CheatVarType>>,
    functions: Option<HashMap<String, CheatFunctionInfo>>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct LocalVarTypeBinding(LocalId, NlAbstractTy);

fn extract_type_bindings(program: &mut Program, id: FunctionId) -> HashSet<LocalVarTypeBinding> {
    struct BC<'a> {
        // BindingContext
        program: &'a mut Program,
        id: FunctionId,
    }

    fn node_helper(nid: &NodeId, bc: &BC) -> Option<LocalVarTypeBinding> {
        let nodes = &bc.program.nodes;
        match nodes[*nid] {
            NodeKind::SetLocalVar(SetLocalVar { local_id, value }) => {
                let output_type = nodes[value].output_type(bc.program, bc.id);
                if let Some(abs_type) = output_type.abstr {
                    Some(LocalVarTypeBinding(local_id, abs_type))
                } else {
                    panic!("Unrecognized output type: {:?}", output_type)
                }
            }
            _ => None,
        }
    }

    #[rustfmt::skip]
    fn structure_helper(statement: &StatementKind, bc: &BC) -> HashSet<LocalVarTypeBinding> {
        match statement {
            NodeStatement(node_id) =>
                node_helper(node_id, bc).into_iter().collect(),
            IfElse { then_block, else_block, .. } =>
                block_helper(then_block, bc).union(&block_helper(else_block, bc)).cloned().collect(),
            Repeat { block, .. } =>
                block_helper(block, bc),
            Return { .. } =>
                HashSet::new(),
            Stop =>
                HashSet::new(),
        }
    }

    fn block_helper(b: &StatementBlock, bc: &BC) -> HashSet<LocalVarTypeBinding> {
        b.statements.iter().flat_map(|x| structure_helper(x, bc)).collect()
    }

    let bc = BC { program, id };

    block_helper(&bc.program.functions[id].cfg, &bc)
}

fn extract_report_types(program: &mut Program, id: FunctionId) -> HashSet<NlAbstractTy> {
    struct BC<'a> {
        // BindingContext
        program: &'a mut Program,
        id: FunctionId,
    }

    #[rustfmt::skip]
    fn structure_helper(statement: &StatementKind, bc: &BC) -> HashSet<NlAbstractTy> {
        match statement {
            NodeStatement(..) =>
                HashSet::new(),
            IfElse { then_block, else_block, .. } =>
                block_helper(then_block, bc).union(&block_helper(else_block, bc)).cloned().collect(),
            Repeat { block, .. } =>
                block_helper(block, bc),
            Return { value } => {
                let output_type = bc.program.nodes[*value].output_type(bc.program, bc.id);
                if let Some(typ) = output_type.abstr {
                    HashSet::from([typ])
                } else {
                    panic!("Unrecognized report type: {:?}", output_type)
                }
            },
            Stop =>
                HashSet::new(),
        }
    }

    fn block_helper(b: &StatementBlock, bc: &BC) -> HashSet<NlAbstractTy> {
        b.statements.iter().flat_map(|x| structure_helper(x, bc)).collect()
    }

    let bc = BC { program, id };

    block_helper(&bc.program.functions[id].cfg, &bc)
}

fn lub(types: &[NlAbstractTy]) -> NlAbstractTy {
    if types.is_empty() || types.iter().any(|t| *t == NlAbstractTy::Unit) {
        NlAbstractTy::Unit
    } else if types.contains(&NlAbstractTy::Top) {
        NlAbstractTy::Top
    } else {
        todo!("TODO Implement least upper bound for other types")
    }
}

/// `avoid_occupancy_bitfield` specifies index numbers of variables that will have fully-dense
/// sequences of values.  Said sequences can be sparse when agents in model can be of mixed or dynamic
/// breeds. --Jason B. (11/18/25)
pub fn add_cheats(
    cheats: &Cheats,
    program: &mut Program,
    global_names: &GlobalScope,
    fn_info: &SecondaryMap<FunctionId, FnInfo>,
) {
    fn translate_var_type_name(var_type: &CheatVarType) -> NlAbstractTy {
        match var_type {
            CheatVarType::Float => NlAbstractTy::Float,
            CheatVarType::Boolean => NlAbstractTy::Boolean,
        }
    }

    {
        let mut types = Vec::new();

        if let Some(globals_var_types) = &cheats.globals_var_types {
            for (var_name, var_type) in globals_var_types {
                let Some(var_id) = global_names.global_vars.get(var_name.as_str()).copied() else {
                    panic!("variable {} is not a custom global variable", var_name);
                };
                let typ = translate_var_type_name(var_type);
                types.push(typ.repr());
                program.globals[var_id].ty = typ.into();
            }
        }

        if let Some(globals_cheaty_schema) = &cheats.globals_schema {
            let globals_schema = match globals_cheaty_schema {
                CheatGlobalsSchema::Default => GlobalsSchema::new(&types),
            };
            program.globals_schema = Some(globals_schema);
        };
    }

    if let Some(patch_var_types) = &cheats.patch_var_types {
        for (var_name, var_type) in patch_var_types {
            let PatchVarDesc::Custom(var_id) =
                *global_names.patch_vars.get(var_name.as_str()).unwrap()
            else {
                panic!("variable {} is not a custom patch variable", var_name);
            };

            let var_type = translate_var_type_name(var_type);

            program.custom_patch_vars[var_id].ty = var_type.into();
        }
    }

    if let Some(patch_schema) = &cheats.patch_schema {
        let patch_schema = match patch_schema {
            CheatPatchSchema::Default => PatchSchema::default(),
            CheatPatchSchema::Ctor(ctor_args) => {
                let custom_fields: Vec<_> = ctor_args
                    .custom_fields
                    .iter()
                    .map(|&buffer_idx| {
                        (program.custom_patch_vars[usize::from(buffer_idx)].ty.repr(), buffer_idx)
                    })
                    .collect();
                PatchSchema::new(
                    ctor_args.pcolor_buffer_idx,
                    &custom_fields,
                    &ctor_args.avoid_occupancy_bitfield,
                )
            }
        };
        program.patch_schema = Some(patch_schema);
    };

    if let Some(turtle_var_types) = &cheats.turtle_var_types {
        for (var_name, var_type) in turtle_var_types {
            let TurtleVarDesc::Custom(var_id) =
                *global_names.turtle_vars.get(var_name.as_str()).unwrap()
            else {
                panic!("variable {} is not a custom turtle variable", var_name);
            };
            let var_type = translate_var_type_name(var_type);
            program.custom_turtle_vars[var_id].ty = var_type.into();
        }
    }

    if let Some(turtle_schema) = &cheats.turtle_schema {
        let turtle_schema = match turtle_schema {
            CheatTurtleSchema::Default => TurtleSchema::default(),
            CheatTurtleSchema::Ctor(ctor_args) => {
                let custom_fields: Vec<_> = ctor_args
                    .custom_fields
                    .iter()
                    .map(|&buffer_idx| {
                        (program.custom_turtle_vars[usize::from(buffer_idx)].ty.repr(), buffer_idx)
                    })
                    .collect();
                TurtleSchema::new(
                    ctor_args.heading_buffer_idx,
                    ctor_args.position_buffer_idx,
                    &custom_fields,
                    &ctor_args.avoid_occupancy_bitfield,
                )
            }
        };
        program.turtle_schema = Some(turtle_schema);
    }

    if let Some(fn_cheats) = &cheats.functions {
        for (fn_name, info) in fn_cheats {
            let fn_id = global_names.functions.get(fn_name.as_str()).unwrap();
            if let Some(self_param_type) = &info.self_param_type {
                let fn_info = &fn_info[*fn_id];
                let ty = &mut program.locals[fn_info.self_param.unwrap()].ty;
                *ty = match self_param_type {
                    CheatSelfParamType::Patch => NlAbstractTy::Patch.into(),
                    CheatSelfParamType::Turtle => NlAbstractTy::Turtle.into(),
                }
            }
        }
    }

    let func_ids: Vec<FunctionId> = program.functions.keys().collect();

    for func_id in func_ids {
        {
            let bindings_set = extract_type_bindings(program, func_id);

            let func = &mut program.functions[func_id];

            let mut lid_to_types: HashMap<LocalId, Vec<NlAbstractTy>> =
                func.locals.clone().into_iter().map(|k| (k, Vec::new())).collect();

            for LocalVarTypeBinding(local_id, typ) in bindings_set {
                lid_to_types.get_mut(&local_id).unwrap().push(typ);
            }

            for local_id in &func.locals {
                let decl = &mut program.locals[*local_id];
                let types = &lid_to_types.get(local_id).as_ref().unwrap()[..];
                if !types.is_empty() {
                    let abs_type = if let [typ] = types {
                        typ.clone()
                    } else {
                        todo!("TODO Maybe find a least upper-bound type, if we're confident in it")
                    };
                    decl.ty = abs_type.into();
                }
            }
        }

        {
            let report_types_set = extract_report_types(program, func_id);
            let report_types = &report_types_set.iter().cloned().collect::<Vec<_>>()[..];

            let out_type = if report_types.is_empty() {
                NlAbstractTy::Unit.into()
            } else if let [typ] = report_types {
                typ.clone().into()
            } else {
                lub(report_types).into()
            };

            let func = &mut program.functions[func_id];
            func.return_ty = out_type;
        }
    }
}
