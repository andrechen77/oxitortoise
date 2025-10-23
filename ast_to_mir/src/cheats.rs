use std::collections::HashMap;

use engine::{
    mir::{self, FunctionId, MirType, NetlogoAbstractType},
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        patch::PatchVarDesc,
        value::NetlogoMachineType,
    },
    slotmap::SecondaryMap,
};
use serde::Deserialize;

use crate::{FnInfo, GlobalNames};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheatVarType {
    Float,
    Boolean,
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
    // TODO add ctor type
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
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
    patch_var_types: Option<HashMap<String, CheatVarType>>,
    patch_schema: Option<CheatPatchSchema>,
    turtle_schema: Option<CheatTurtleSchema>,
    functions: Option<HashMap<String, CheatFunctionInfo>>,
}

pub fn add_cheats(
    cheats: &Cheats,
    mir: &mut mir::Program,
    global_names: &GlobalNames,
    fn_info: &SecondaryMap<FunctionId, FnInfo>,
) {
    fn translate_var_type_name(var_type: &CheatVarType) -> NetlogoAbstractType {
        match var_type {
            CheatVarType::Float => NetlogoAbstractType::Float,
            CheatVarType::Boolean => NetlogoAbstractType::Boolean,
        }
    }

    if let Some(patch_var_types) = &cheats.patch_var_types {
        for (var_name, var_type) in patch_var_types {
            let PatchVarDesc::Custom(var_id) =
                *global_names.patch_vars.get(var_name.as_str()).unwrap()
            else {
                panic!("variable {} is not a custom patch variable", var_name);
            };

            let var_type = translate_var_type_name(var_type);

            mir.custom_patch_vars[var_id].ty = var_type;
        }
    }

    // TODO add turtle variable types

    if let Some(patch_schema) = &cheats.patch_schema {
        let patch_schema = match patch_schema {
            CheatPatchSchema::Default => PatchSchema::default(),
            CheatPatchSchema::Ctor(ctor_args) => {
                let custom_fields: Vec<_> = ctor_args
                    .custom_fields
                    .iter()
                    .map(|&buffer_idx| {
                        (mir.custom_patch_vars[usize::from(buffer_idx)].ty.repr(), buffer_idx)
                    })
                    .collect();
                PatchSchema::new(
                    ctor_args.pcolor_buffer_idx,
                    &custom_fields,
                    &ctor_args.avoid_occupancy_bitfield,
                )
            }
        };
        mir.patch_schema = Some(patch_schema);
    };

    if let Some(turtle_schema) = &cheats.turtle_schema {
        let turtle_schema = match turtle_schema {
            CheatTurtleSchema::Default => TurtleSchema::default(),
            // TODO add ctor type
        };
        mir.turtle_schema = Some(turtle_schema);
    }

    if let Some(fn_cheats) = &cheats.functions {
        for (fn_name, fn_cheats) in fn_cheats {
            let fn_id = global_names.functions.get(fn_name.as_str()).unwrap();
            if let Some(self_param_type) = &fn_cheats.self_param_type {
                let fn_info = &fn_info[*fn_id];
                let ty =
                    &mut mir.functions[*fn_id].borrow_mut().locals[fn_info.self_param.unwrap()].ty;
                *ty = match self_param_type {
                    CheatSelfParamType::Patch => MirType::Machine(NetlogoMachineType::PATCH_ID),
                    CheatSelfParamType::Turtle => MirType::Machine(NetlogoMachineType::TURTLE_ID),
                }
            }
        }
    }

    // TODO add types of variables
}
