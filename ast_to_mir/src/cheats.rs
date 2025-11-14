use std::collections::HashMap;

use engine::{
    mir::{self, FunctionId, MirTy, NlAbstractTy},
    sim::{
        agent_schema::{GlobalsSchema, PatchSchema, TurtleSchema},
        patch::PatchVarDesc,
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
    // TODO(mvp_ants) add ctor type, similar to PatchSchemaCtor
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
    functions: Option<HashMap<String, CheatFunctionInfo>>,
}

/// `avoid_occupancy_bitfield` specifies index numbers of variables that will have fully-dense
/// sequences of values.  Said sequences can be sparse when agents in model can be of mixed or dynamic
/// breeds. --Jason B. (11/18/25)
pub fn add_cheats(
    cheats: &Cheats,
    program: &mut mir::Program,
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
                program.globals[var_id].ty = typ;
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

            program.custom_patch_vars[var_id].ty = var_type;
        }
    }

    // TODO(mvp_ants) add turtle variable types

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

    if let Some(turtle_schema) = &cheats.turtle_schema {
        let turtle_schema = match turtle_schema {
            CheatTurtleSchema::Default => TurtleSchema::default(),
        };
        program.turtle_schema = Some(turtle_schema);
    }

    if let Some(fn_cheats) = &cheats.functions {
        for (fn_name, fn_cheats) in fn_cheats {
            let fn_id = global_names.functions.get(fn_name.as_str()).unwrap();
            if let Some(self_param_type) = &fn_cheats.self_param_type {
                let fn_info = &fn_info[*fn_id];
                let ty =
                    &mut program.functions[*fn_id].borrow_mut().locals[fn_info.self_param.unwrap()].ty;
                *ty = match self_param_type {
                    CheatSelfParamType::Patch => MirTy::Abstract(NlAbstractTy::Patch),
                    CheatSelfParamType::Turtle => MirTy::Abstract(NlAbstractTy::Turtle),
                }
            }
        }
    }

    // TODO(mvp_ants) add types of global and local variables
}
